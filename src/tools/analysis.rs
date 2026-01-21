use crate::analyzer::RustAnalyzerClient;
use crate::tools::types::ToolResult;
use anyhow::Result;
use serde_json::{Value, json};
use tokio::fs;

pub async fn find_definition_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing line parameter"))?;
    let character = args
        .get("character")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing character parameter"))?;

    // Implementation will use rust-analyzer LSP to find definition
    let result = analyzer
        .find_definition(file_path, line as u32, character as u32)
        .await?;

    Ok(ToolResult {
        content: vec![
            json!({
                "type": "text",
                "text": result
            })
            .as_object()
            .unwrap()
            .clone(),
        ],
    })
}

pub async fn find_references_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing line parameter"))?;
    let character = args
        .get("character")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing character parameter"))?;

    // Implementation will use rust-analyzer LSP to find references
    let result = analyzer
        .find_references(file_path, line as u32, character as u32)
        .await?;

    Ok(ToolResult {
        content: vec![
            json!({
                "type": "text",
                "text": result
            })
            .as_object()
            .unwrap()
            .clone(),
        ],
    })
}

pub async fn get_diagnostics_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;

    // Implementation will use rust-analyzer LSP to get diagnostics
    let diagnostics_result = analyzer.get_diagnostics(file_path).await?;

    Ok(ToolResult {
        content: vec![
            json!({
                "type": "text",
                "text": diagnostics_result
            })
            .as_object()
            .unwrap()
            .clone(),
        ],
    })
}

pub async fn get_hover_impl(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing line parameter"))?;
    let character = args
        .get("character")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing character parameter"))?;

    let hover_result = analyzer
        .get_hover(file_path, line as u32, character as u32)
        .await?;

    Ok(ToolResult {
        content: vec![
            json!({
                "type": "text",
                "text": hover_result
            })
            .as_object()
            .unwrap()
            .clone(),
        ],
    })
}

pub async fn get_symbol_source_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing line parameter"))?;
    let character = args
        .get("character")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing character parameter"))?;

    // Try to read the line content for context (even if get_symbol_source fails)
    let context_line = match fs::read_to_string(file_path).await {
        Ok(content) => content
            .lines()
            .nth(line as usize)
            .unwrap_or("<line out of bounds>")
            .to_string(),
        Err(e) => format!("<failed to read file: {}>", e),
    };
    let context_marker = create_position_marker(&context_line, character as u32);

    match analyzer
        .get_symbol_source(file_path, line as u32, character as u32)
        .await
    {
        Ok((source, range, actual_path)) => {
            let result = json!({
                "request": {
                    "file_path": file_path,
                    "line": line,
                    "character": character,
                    "context_line": context_line,
                    "context_marker": context_marker
                },
                "result": {
                    "file_path": actual_path,
                    "range": range,
                    "source": source
                }
            });

            Ok(ToolResult {
                content: vec![
                    json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result)?
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ],
            })
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to get symbol source. Request context: Line {}: '{}'\nPosition: '{}'. Error: {}",
            line,
            context_line.trim(),
            context_marker,
            e
        )),
    }
}

fn create_position_marker(line_content: &str, char_idx: u32) -> String {
    let mut marker = String::new();
    let mut current_char_count = 0;
    for c in line_content.chars() {
        if current_char_count == char_idx {
            marker.push('^');
            break;
        } else {
            if c == '\t' {
                // Approximate tab width for visual alignment
                marker.push_str("    ");
            } else {
                marker.push(' ');
            }
            current_char_count += 1;
        }
    }
    // If character is beyond line length, append to end or pad with spaces
    while current_char_count < char_idx {
        marker.push(' ');
        current_char_count += 1;
    }
    if current_char_count == char_idx {
        // Handle char_idx exactly at line end
        marker.push('^');
    }
    marker
}
