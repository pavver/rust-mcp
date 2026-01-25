use anyhow::Result;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

use crate::analyzer::RustAnalyzerClient;

pub struct ToolDefinition {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub input_schema: Arc<serde_json::Map<String, Value>>,
}

impl ToolDefinition {
    pub fn new(name: &'static str, description: &'static str, schema: Value) -> Self {
        let schema_map = match schema {
            Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        };

        Self {
            name: Cow::Borrowed(name),
            description: Cow::Borrowed(description),
            input_schema: schema_map,
        }
    }
}

pub struct ToolResult {
    pub content: Vec<serde_json::Map<String, Value>>,
}

fn not_implemented_tool_result(tool_name: &str) -> ToolResult {
    ToolResult {
        content: vec![
            json!({
                "type": "text",
                "text": format!("{tool_name} not implemented")
            })
            .as_object()
            .unwrap()
            .clone(),
        ],
    }
}

pub async fn execute_tool(
    name: &str,
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    match name {
        "find_definition" => crate::tools::analysis::find_definition_impl(args, analyzer).await,
        "find_references" => crate::tools::analysis::find_references_impl(args, analyzer).await,
        "get_diagnostics" => crate::tools::analysis::get_diagnostics_impl(args, analyzer).await,
        "workspace_symbols" => {
            crate::tools::navigation::workspace_symbols_impl(args, analyzer).await
        }
        "document_symbols" => crate::tools::navigation::document_symbols_impl(args, analyzer).await,
        "get_hover" => crate::tools::analysis::get_hover_impl(args, analyzer).await,
        "get_symbol_source" => crate::tools::analysis::get_symbol_source_impl(args, analyzer).await,
        "rename_symbol" => crate::tools::refactoring::rename_symbol_impl(args, analyzer).await,
        "extract_function" => {
            crate::tools::refactoring::extract_function_impl(args, analyzer).await
        }
        "run_cargo_check" => crate::tools::cargo::run_cargo_check_impl(args, analyzer).await,
        "inline_function" => crate::tools::refactoring::inline_function_impl(args, analyzer).await,
        "apply_clippy_suggestions" => {
            crate::tools::quality::apply_clippy_suggestions_impl(args, analyzer).await
        }
        "get_type_hierarchy" => {
            crate::tools::advanced::get_type_hierarchy_impl(args, analyzer).await
        }
        "suggest_dependencies" => {
            crate::tools::advanced::suggest_dependencies_impl(args, analyzer).await
        }
        "move_items" => crate::tools::advanced::move_items_impl(args, analyzer).await,
        "inspect_mir" => Ok(not_implemented_tool_result("inspect_mir")),
        "inspect_llvm_ir" => Ok(not_implemented_tool_result("inspect_llvm_ir")),
        "inspect_asm" => Ok(not_implemented_tool_result("inspect_asm")),
        "inspect" => Ok(not_implemented_tool_result("inspect")),
        "capabilities" => Ok(not_implemented_tool_result("capabilities")),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}

pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        // Code Analysis
        ToolDefinition::new(
            "find_definition",
            "Locates the definition of a specific symbol by searching within a provided code block. More reliable than using raw coordinates.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Absolute path to the file"},
                    "symbol": {"type": "string", "description": "The exact symbol name to find"},
                    "code_block": {"type": "string", "description": "A unique multi-line code snippet containing the target symbol"},
                    "occurrence": {"type": "integer", "description": "The 1-based index of the symbol's occurrence within the code_block", "default": 1}
                },
                "required": ["file_path", "symbol", "code_block"]
            }),
        ),
        ToolDefinition::new(
            "find_references",
            "Finds all references to a specific symbol by searching within a provided code block. Useful for refactoring and understanding usage patterns.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Absolute path to the file"},
                    "symbol": {"type": "string", "description": "The exact symbol name to find references for"},
                    "code_block": {"type": "string", "description": "A unique multi-line code snippet containing the target symbol"},
                    "occurrence": {"type": "integer", "description": "The 1-based index of the symbol's occurrence within the code_block", "default": 1}
                },
                "required": ["file_path", "symbol", "code_block"]
            }),
        ),
        ToolDefinition::new(
            "get_diagnostics",
            "Get compiler diagnostics for a file",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "workspace_symbols",
            "Search for symbols in the workspace",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
        ),
        ToolDefinition::new(
            "document_symbols",
            "Retrieves the hierarchical structure (symbols) of a file. PREFERRED over `read_file` for large files to understand code organization without consuming massive context tokens. Returns an outline of functions, structs, and impls.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "get_hover",
            "Retrieves hover information (signature, documentation) for a specific symbol by locating it within a provided code block. This method is more robust than using line/character coordinates.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Absolute path to the file"},
                    "symbol": {"type": "string", "description": "The exact symbol name to hover over"},
                    "code_block": {"type": "string", "description": "A unique multi-line code snippet (3-5 lines) containing the target symbol to ensure correct context"},
                    "occurrence": {"type": "integer", "description": "The 1-based index of the symbol's occurrence within the provided code_block. Defaults to 1.", "default": 1}
                },
                "required": ["file_path", "symbol", "code_block"]
            }),
        ),
        ToolDefinition::new(
            "get_symbol_source",
            "Retrieves the source code of a symbol by locating it within a provided code block. Useful for reading implementations.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Absolute path to the file"},
                    "symbol": {"type": "string", "description": "The exact symbol name"},
                    "code_block": {"type": "string", "description": "A unique multi-line code snippet containing the symbol"},
                    "occurrence": {"type": "integer", "description": "The 1-based index of the symbol's occurrence within the code_block", "default": 1}
                },
                "required": ["file_path", "symbol", "code_block"]
            }),
        ),
        ToolDefinition::new(
            "rename_symbol",
            "Rename a symbol with scope awareness",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "number"},
                    "character": {"type": "number"},
                    "new_name": {"type": "string"}
                },
                "required": ["file_path", "line", "character", "new_name"]
            }),
        ),
        ToolDefinition::new(
            "run_cargo_check",
            "Execute cargo check and parse errors",
            json!({
                "type": "object",
                "properties": {
                    "workspace_path": {"type": "string"}
                },
                "required": ["workspace_path"]
            }),
        ),
        ToolDefinition::new(
            "extract_function",
            "Extract selected code into a new function",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "start_line": {"type": "number"},
                    "start_character": {"type": "number"},
                    "end_line": {"type": "number"},
                    "end_character": {"type": "number"},
                    "function_name": {"type": "string"}
                },
                "required": ["file_path", "start_line", "start_character", "end_line", "end_character", "function_name"]
            }),
        ),
        ToolDefinition::new(
            "inline_function",
            "Inline a function call at specified position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "number"},
                    "character": {"type": "number"}
                },
                "required": ["file_path", "line", "character"]
            }),
        ),
        ToolDefinition::new(
            "apply_clippy_suggestions",
            "Apply clippy lint suggestions to improve code quality",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "validate_lifetimes",
            "Validate and suggest lifetime annotations",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "get_type_hierarchy",
            "Retrieves the type hierarchy (supertypes/traits implemented, subtypes/implementations) for a symbol. Useful for understanding trait relationships and implementations.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Absolute path to the file"},
                    "symbol": {"type": "string", "description": "The exact symbol name"},
                    "code_block": {"type": "string", "description": "A unique multi-line code snippet containing the symbol"},
                    "occurrence": {"type": "integer", "description": "The 1-based index of the symbol's occurrence within the code_block", "default": 1}
                },
                "required": ["file_path", "symbol", "code_block"]
            }),
        ),
        ToolDefinition::new(
            "suggest_dependencies",
            "Suggest crate dependencies based on code patterns",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "workspace_path": {"type": "string"}
                },
                "required": ["query", "workspace_path"]
            }),
        ),
        ToolDefinition::new(
            "move_items",
            "Move code items from one file to another",
            json!({
                "type": "object",
                "properties": {
                    "source_file": {"type": "string"},
                    "target_file": {"type": "string"},
                    "item_names": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                },
                "required": ["source_file", "target_file", "item_names"]
            }),
        ),
        ToolDefinition::new(
            "inspect_mir",
            "Inspect MIR for a symbol or source position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "integer", "minimum": 0},
                    "character": {"type": "integer", "minimum": 0},
                    "symbol_name": {"type": "string"},
                    "opt_level": {"type": "string"},
                    "target": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "inspect_llvm_ir",
            "Inspect LLVM IR for a symbol or source position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "integer", "minimum": 0},
                    "character": {"type": "integer", "minimum": 0},
                    "symbol_name": {"type": "string"},
                    "opt_level": {"type": "string"},
                    "target": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "inspect_asm",
            "Inspect assembly for a symbol or source position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "integer", "minimum": 0},
                    "character": {"type": "integer", "minimum": 0},
                    "symbol_name": {"type": "string"},
                    "opt_level": {"type": "string"},
                    "target": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "inspect",
            "Inspect compiler artifacts using curated presets",
            json!({
                "type": "object",
                "properties": {
                    "view": {"type": "string"},
                    "file_path": {"type": "string"},
                    "line": {"type": "integer", "minimum": 0},
                    "character": {"type": "integer", "minimum": 0},
                    "symbol_name": {"type": "string"},
                    "opt_level": {"type": "string"},
                    "target": {"type": "string"},
                    "gating_mode": {"type": "string"}
                },
                "required": ["view", "file_path", "line", "character"]
            }),
        ),
        ToolDefinition::new(
            "capabilities",
            "Discover supported inspection presets and limits",
            json!({
                "type": "object",
                "properties": {
                    "gating_mode": {"type": "string"}
                }
            }),
        ),
    ]
}