use crate::analyzer::RustAnalyzerClient;
use crate::tools::analysis::find_symbol_location;
use crate::tools::types::ToolResult;
use anyhow::Result;
use serde_json::{Value, json};
use tokio::fs;

pub async fn rename_symbol_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing symbol parameter"))?;
    let code_block = args
        .get("code_block")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing code_block parameter"))?;
    let occurrence = args
        .get("occurrence")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;
    let new_name = args
        .get("new_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing new_name parameter"))?;

    let file_content = fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    let (line, character) = find_symbol_location(&file_content, symbol, code_block, occurrence)?;

    // Implementation will use rust-analyzer LSP to rename symbol
    let result = analyzer
        .rename_symbol(file_path, line, character, new_name)
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

pub async fn extract_function_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let code_block = args
        .get("code_block")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing code_block parameter"))?;
    let occurrence = args
        .get("occurrence")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;
    let function_name = args
        .get("function_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing function_name parameter"))?;

    let file_content = fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    let (start_line, start_char, end_line, end_char) =
        crate::tools::analysis::find_block_range(&file_content, code_block, occurrence)?;

    // Implementation will use rust-analyzer LSP to extract function
    let result = analyzer
        .extract_function(
            file_path,
            start_line,
            start_char,
            end_line,
            end_char,
            function_name,
        )
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

pub async fn inline_function_impl(
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

    let result = analyzer
        .inline_function(file_path, line as u32, character as u32)
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
