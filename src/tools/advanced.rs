use crate::analyzer::RustAnalyzerClient;
use crate::tools::types::ToolResult;
use anyhow::Result;
use serde_json::{Value, json};

use tokio::fs;
use crate::tools::analysis::find_symbol_location;

pub async fn get_type_hierarchy_impl(
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

    let file_content = fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    let (line, character) = find_symbol_location(&file_content, symbol, code_block, occurrence)?;

    let result = analyzer
        .get_type_hierarchy(file_path, line, character)
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

pub async fn suggest_dependencies_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;
    let workspace_path = args
        .get("workspace_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing workspace_path parameter"))?;

    let result = analyzer.suggest_dependencies(query, workspace_path).await?;

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

pub async fn move_items_impl(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let source_file = args
        .get("source_file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing source_file parameter"))?;
    let target_file = args
        .get("target_file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target_file parameter"))?;
    let item_names_value = args
        .get("item_names")
        .ok_or_else(|| anyhow::anyhow!("Missing item_names parameter"))?;

    let item_names: Vec<&str> = item_names_value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("item_names must be an array"))?
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();

    let result = analyzer
        .move_items(source_file, target_file, &item_names)
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
