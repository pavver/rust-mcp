use crate::analyzer::RustAnalyzerClient;
use crate::tools::types::ToolResult;
use anyhow::Result;
use serde_json::{Value, json};

pub async fn run_cargo_check_impl(
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    let workspace_path = args
        .get("workspace_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing workspace_path parameter"))?;

    // Implementation will run cargo check and parse results
    let result = analyzer.run_cargo_check(workspace_path).await?;

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
