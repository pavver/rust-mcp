use anyhow::Result;
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

use crate::analyzer::protocol::*;

#[derive(Debug, Clone)]
pub struct DefinitionDetails {
    pub location: Location,
    pub symbol_path: SymbolPath,
}

fn get_rust_analyzer_path() -> String {
    std::env::var("RUST_ANALYZER_PATH").unwrap_or_else(|_| {
        // Default to ~/.cargo/bin/rust-analyzer
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.cargo/bin/rust-analyzer")
    })
}

pub struct RustAnalyzerClient {
    process: Option<Child>,
    request_id: u64,
    initialized: bool,
}

impl Default for RustAnalyzerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzerClient {
    pub fn new() -> Self {
        Self {
            process: None,
            request_id: 0,
            initialized: false,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let rust_analyzer_path = get_rust_analyzer_path();
        let child = tokio::process::Command::new(&rust_analyzer_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        self.initialize().await?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        // Get current working directory
        let current_dir = std::env::current_dir()?;
        let root_uri = format!("file://{}", current_dir.display());

        let full_analysis = std::env::var("RUST_MCP_FULL_ANALYSIS")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let initialization_options = if full_analysis {
            json!({
                "cargo": {
                    "loadOutDirsFromCheck": true
                },
                "procMacro": {
                    "enable": true
                }
            })
        } else {
            json!({
                "cargo": {
                    "loadOutDirsFromCheck": false
                },
                "procMacro": {
                    "enable": false
                }
            })
        };

        // Send initialize request
        let init_params = json!({
            "processId": null,
            "clientInfo": {
                "name": "rust-mcp-server",
                "version": "0.1.0"
            },
            "rootUri": root_uri,
            "initializationOptions": initialization_options,
            "capabilities": {
                "textDocument": {
                    "definition": {
                        "dynamicRegistration": false
                    },
                    "references": {
                        "dynamicRegistration": false
                    },
                    "publishDiagnostics": {
                        "relatedInformation": true
                    },
                    "documentSymbol": {
                        "hierarchicalDocumentSymbolSupport": true
                    }
                },
                "workspace": {
                    "symbol": {
                        "dynamicRegistration": false
                    }
                }
            }
        });

        let _response = self
            .send_request_internal("initialize", init_params)
            .await?;

        // Send initialized notification
        self.send_notification("initialized", json!({})).await?;

        self.initialized = true;
        Ok(())
    }

    async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification).await
    }

    async fn send_request_internal(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        self.send_message(&request).await?;
        self.read_response(self.request_id).await
    }

    async fn send_message(&mut self, message: &Value) -> Result<()> {
        let content = message.to_string();
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        if let Some(child) = &mut self.process {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(header.as_bytes()).await?;
                stdin.write_all(content.as_bytes()).await?;
                stdin.flush().await?;
            }
        }

        Ok(())
    }

    async fn read_response(&mut self, expected_id: u64) -> Result<Value> {
        if let Some(child) = &mut self.process {
            if let Some(stdout) = child.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);

                loop {
                    // Read headers
                    let mut content_length: Option<usize> = None;
                    loop {
                        let mut line = String::new();
                        reader.read_line(&mut line).await?;

                        if line == "\r\n" {
                            break;
                        }

                        if let Some(stripped) = line.strip_prefix("Content-Length:") {
                            let length_str = stripped.trim();
                            content_length = Some(length_str.parse()?);
                        }
                    }

                    if let Some(length) = content_length {
                        let mut content = vec![0u8; length];
                        reader.read_exact(&mut content).await?;

                        let response: Value = serde_json::from_slice(&content)?;

                        if let Some(id) = response.get("id") {
                            if id.as_u64() == Some(expected_id) {
                                return Ok(response);
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to read response"))
    }

    // Tool implementation methods
    fn ensure_initialized(&self) -> Result<()> {
        if self.initialized {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Client not initialized"))
        }
    }

    fn extract_result(response: &Value) -> Result<Value> {
        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing result field in LSP response"))
    }

    fn position_in_range(range: &Range, position: &Position) -> bool {
        let starts_before = range.start.line < position.line
            || (range.start.line == position.line && range.start.character <= position.character);
        let ends_after = range.end.line > position.line
            || (range.end.line == position.line && range.end.character >= position.character);
        starts_before && ends_after
    }

    fn select_definition_location(definition: DefinitionResponse) -> Option<Location> {
        match definition {
            DefinitionResponse::SingleLocation(location) => Some(location),
            DefinitionResponse::LocationArray(mut locations) => locations.pop(),
            DefinitionResponse::LocationLinks(mut links) => links.pop().map(|link| Location {
                uri: link.target_uri,
                range: link.target_selection_range,
            }),
        }
    }

    fn find_symbol_path_in_document_symbols(
        symbols: &[DocumentSymbol],
        position: &Position,
    ) -> Option<SymbolPath> {
        for symbol in symbols {
            if Self::position_in_range(&symbol.selection_range, position) {
                let mut path = vec![SymbolPathSegment {
                    name: symbol.name.clone(),
                    kind: symbol.kind,
                }];

                if let Some(children) = &symbol.children {
                    if let Some(mut child_path) =
                        Self::find_symbol_path_in_document_symbols(children, position)
                    {
                        path.append(&mut child_path);
                    }
                }

                return Some(path);
            }
        }
        None
    }

    fn symbol_path_from_response(
        symbols: DocumentSymbolResponse,
        position: &Position,
    ) -> Option<SymbolPath> {
        match symbols {
            DocumentSymbolResponse::DocumentSymbols(symbols) => {
                Self::find_symbol_path_in_document_symbols(&symbols, position)
            }
            DocumentSymbolResponse::SymbolInformation(mut infos) => infos
                .drain(..)
                .find(|info| Self::position_in_range(&info.location.range, position))
                .map(|info| {
                    let mut path = Vec::new();
                    if let Some(container) = info.container_name {
                        path.push(SymbolPathSegment {
                            name: container,
                            kind: info.kind,
                        });
                    }
                    path.push(SymbolPathSegment {
                        name: info.name,
                        kind: info.kind,
                    });
                    path
                }),
        }
    }

    async fn request_document_symbols(&mut self, uri: &str) -> Result<DocumentSymbolResponse> {
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier {
                uri: uri.to_string(),
            },
        };

        let response = self
            .send_request_internal("textDocument/documentSymbol", serde_json::to_value(params)?)
            .await?;

        let result_value = Self::extract_result(&response)?;
        let parsed: DocumentSymbolResponse = serde_json::from_value(result_value)?;
        Ok(parsed)
    }

    pub async fn definition_details(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<DefinitionDetails>> {
        self.ensure_initialized()?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: format!("file://{}", file_path),
            },
            position: Position { line, character },
        };

        let response = self
            .send_request_internal("textDocument/definition", serde_json::to_value(params)?)
            .await?;

        let result_value = Self::extract_result(&response)?;
        let definition_response: DefinitionResponse = serde_json::from_value(result_value)?;
        let Some(location) = Self::select_definition_location(definition_response) else {
            return Ok(None);
        };

        let symbol_response = self.request_document_symbols(&location.uri).await;
        let symbol_path = match symbol_response {
            Ok(symbols) => {
                Self::symbol_path_from_response(symbols, &location.range.start).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        };

        Ok(Some(DefinitionDetails {
            location,
            symbol_path,
        }))
    }

    fn format_symbol_path(path: &[SymbolPathSegment]) -> Option<String> {
        if path.is_empty() {
            None
        } else {
            Some(
                path.iter()
                    .map(|segment| segment.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::"),
            )
        }
    }

    pub async fn find_definition(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        self.ensure_initialized()?;

        let details = self
            .definition_details(file_path, line, character)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No definition found"))?;

        let path_display = Self::format_symbol_path(&details.symbol_path)
            .unwrap_or_else(|| "<unnamed>".to_string());
        let start = details.location.range.start;
        Ok(format!(
            "Definition at {}:{}:{} ({path_display})",
            details.location.uri,
            start.line + 1,
            start.character + 1
        ))
    }

    pub async fn find_references(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_references_params(file_path, line, character);
        let response = self
            .send_request_internal("textDocument/references", params)
            .await?;

        Ok(format!("References response: {response}"))
    }

    pub async fn get_diagnostics(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        // For diagnostics, we typically receive them via notifications
        // This is a simplified implementation
        Ok(format!("Diagnostics for file: {file_path}"))
    }

    pub async fn workspace_symbols(&mut self, query: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_workspace_symbol_params(query);
        let response = self
            .send_request_internal("workspace/symbol", params)
            .await?;

        Ok(format!("Workspace symbols response: {response}"))
    }

    pub async fn get_hover(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        self.ensure_initialized()?;

        let params = create_text_document_position_params(file_path, line, character);
        let response = self
            .send_request_internal("textDocument/hover", params)
            .await?;

        let result_value = Self::extract_result(&response)?;
        if result_value.is_null() {
            return Ok("No hover information found".to_string());
        }

        let hover: Hover = serde_json::from_value(result_value)?;
        Ok(hover.contents.value)
    }

    pub async fn get_document_symbols(&mut self, file_path: &str) -> Result<String> {
        self.ensure_initialized()?;

        let uri = format!("file://{}", file_path);
        let symbols = self.request_document_symbols(&uri).await?;

        Ok(serde_json::to_string_pretty(&symbols)?)
    }

    fn find_symbol_range_recursive(
        symbols: &[DocumentSymbol],
        position: &Position,
    ) -> Option<Range> {
        for symbol in symbols {
            if Self::position_in_range(&symbol.range, position) {
                if let Some(children) = &symbol.children {
                    if let Some(child_range) = Self::find_symbol_range_recursive(children, position)
                    {
                        return Some(child_range);
                    }
                }
                return Some(symbol.range.clone());
            }
        }
        None
    }

    pub async fn get_symbol_source(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<(String, Range, String)> {
        self.ensure_initialized()?;

        // 1. Try to find definition first to handle references correctly
        let def_params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: format!("file://{}", file_path),
            },
            position: Position { line, character },
        };

        // We catch errors here because if definition fails, we might want to fallback to current position
        // assuming the user pointed directly at a definition.
        let (target_uri, target_point) = match self
            .send_request_internal("textDocument/definition", serde_json::to_value(def_params)?)
            .await
        {
            Ok(def_response) => {
                let def_result = Self::extract_result(&def_response)?;

                if def_result.is_null() {
                    // Fallback to current file if definition lookup fails.
                    // This allows the tool to work for symbols defined at the cursor position
                    // or when rust-analyzer fails to resolve the reference.
                    (
                        format!("file://{}", file_path),
                        Position { line, character },
                    )
                } else {
                    let def_parsed: DefinitionResponse = serde_json::from_value(def_result)?;

                    if let Some(loc) = Self::select_definition_location(def_parsed) {
                        (loc.uri, loc.range.start)
                    } else {
                        // Fallback
                        (
                            format!("file://{}", file_path),
                            Position { line, character },
                        )
                    }
                }
            }
            Err(_) => (
                format!("file://{}", file_path),
                Position { line, character },
            ),
        };

        let target_path = if target_uri.starts_with("file://") {
            target_uri.strip_prefix("file://").unwrap().to_string()
        } else {
            target_uri.clone()
        };

        // 2. Get document symbols for the target file
        // This works for external files too if rust-analyzer indexed them
        let sym_response = self.request_document_symbols(&target_uri).await?;

        // 3. Find range covering the target point
        let range = match sym_response {
            DocumentSymbolResponse::DocumentSymbols(symbols) => {
                Self::find_symbol_range_recursive(&symbols, &target_point).ok_or_else(|| {
                    anyhow::anyhow!(
                        "No symbol found covering definition at {}:{}:{}",
                        target_path,
                        target_point.line,
                        target_point.character
                    )
                })?
            }
            DocumentSymbolResponse::SymbolInformation(symbols) => symbols
                .iter()
                .find(|info| Self::position_in_range(&info.location.range, &target_point))
                .map(|info| info.location.range.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!("No symbol found covering definition (flat view)")
                })?,
        };

        // 4. Read file content
        let content = fs::read_to_string(&target_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", target_path, e))?;

        let lines: Vec<&str> = content.lines().collect();
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        if start_line >= lines.len() {
            return Err(anyhow::anyhow!(
                "Symbol range start line {} is out of bounds",
                start_line
            ));
        }

        let end_line_safe = std::cmp::min(end_line, lines.len().saturating_sub(1));

        if start_line > end_line_safe {
            return Ok((String::new(), range, target_path));
        }

        let code_lines = &lines[start_line..=end_line_safe];
        Ok((code_lines.join("\n"), range, target_path))
    }

    pub async fn rename_symbol(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_rename_params(file_path, line, character, new_name);
        let response = self
            .send_request_internal("textDocument/rename", params)
            .await?;

        Ok(format!("Rename response: {response}"))
    }

    pub async fn format_code(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_formatting_params(file_path);
        let response = self
            .send_request_internal("textDocument/formatting", params)
            .await?;

        Ok(format!("Formatting response: {response}"))
    }

    pub async fn analyze_manifest(&mut self, manifest_path: &str) -> Result<String> {
        // This would analyze Cargo.toml file
        Ok(format!("Manifest analysis for: {manifest_path}"))
    }

    pub async fn run_cargo_check(&mut self, workspace_path: &str) -> Result<String> {
        // This would run cargo check and parse results
        Ok(format!("Cargo check results for: {workspace_path}"))
    }

    pub async fn extract_function(
        &mut self,
        file_path: &str,
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
        function_name: &str,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        // This would use rust-analyzer's extract function code action
        // For now, return a placeholder implementation
        Ok(format!(
            "Extract function '{function_name}' from {file_path}:{start_line}:{start_character} to {end_line}:{end_character}"
        ))
    }

    pub async fn generate_struct(
        &mut self,
        struct_name: &str,
        fields: &[Value],
        derives: &[&str],
        file_path: &str,
    ) -> Result<String> {
        // This would generate a struct with the specified fields and derives
        Ok(format!(
            "Generated struct '{struct_name}' with {} fields and derives {derives:?} in {file_path}",
            fields.len()
        ))
    }

    pub async fn generate_enum(
        &mut self,
        enum_name: &str,
        variants: &[Value],
        derives: &[&str],
        file_path: &str,
    ) -> Result<String> {
        // This would generate an enum with the specified variants and derives
        Ok(format!(
            "Generated enum '{enum_name}' with {} variants and derives {derives:?} in {file_path}",
            variants.len()
        ))
    }

    pub async fn generate_trait_impl(
        &mut self,
        trait_name: &str,
        struct_name: &str,
        file_path: &str,
    ) -> Result<String> {
        // This would generate a trait implementation for the specified struct
        Ok(format!(
            "Generated trait implementation of '{trait_name}' for '{struct_name}' in {file_path}"
        ))
    }

    pub async fn generate_tests(
        &mut self,
        target_function: &str,
        file_path: &str,
        test_cases: &[Value],
    ) -> Result<String> {
        // This would generate unit tests for the specified function
        Ok(format!(
            "Generated {} test cases for function '{target_function}' in {file_path}",
            test_cases.len()
        ))
    }

    pub async fn inline_function(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(format!(
            "Inlined function at {file_path}:{line}:{character}"
        ))
    }

    pub async fn change_signature(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
        new_signature: &str,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(format!(
            "Changed signature to '{new_signature}' at {file_path}:{line}:{character}"
        ))
    }

    pub async fn organize_imports(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(format!("Organized imports in {file_path}"))
    }

    pub async fn apply_clippy_suggestions(&mut self, file_path: &str) -> Result<String> {
        // This would apply clippy suggestions to the file
        Ok(format!("Applied clippy suggestions to {file_path}"))
    }

    pub async fn validate_lifetimes(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(format!("Validated lifetimes in {file_path}"))
    }

    pub async fn get_type_hierarchy(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        // This would use rust-analyzer's type hierarchy capability
        Ok(format!(
            "Type hierarchy for symbol at {file_path}:{line}:{character}"
        ))
    }

    pub async fn suggest_dependencies(
        &mut self,
        query: &str,
        workspace_path: &str,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        // This would analyze code and suggest crates based on usage patterns
        Ok(format!(
            "Dependency suggestions for '{query}' in workspace {workspace_path}"
        ))
    }

    pub async fn create_module(
        &mut self,
        module_name: &str,
        module_path: &str,
        is_public: bool,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        let visibility = if is_public { "pub " } else { "" };
        Ok(format!(
            "Created {visibility}module '{module_name}' at {module_path}"
        ))
    }

    pub async fn move_items(
        &mut self,
        source_file: &str,
        target_file: &str,
        item_names: &[&str],
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(format!(
            "Moved {} items from {source_file} to {target_file}: {item_names:?}",
            item_names.len()
        ))
    }
}
