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

    // Implementation will use rust-analyzer LSP to find definition
    let result = analyzer
        .find_definition(file_path, line, character)
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

    // Implementation will use rust-analyzer LSP to find references
    let result = analyzer
        .find_references(file_path, line, character)
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

    let hover_result = analyzer
        .get_hover(file_path, line, character)
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

pub fn find_block_range(
    file_content: &str,
    code_block: &str,
    occurrence: usize,
) -> Result<(u32, u32, u32, u32)> {
    let mut current_pos = 0;
    let mut current_occurrence = 0;

    while let Some(start_idx) = file_content[current_pos..].find(code_block) {
        let absolute_start_idx = current_pos + start_idx;
        current_occurrence += 1;

        if current_occurrence == occurrence {
            let absolute_end_idx = absolute_start_idx + code_block.len();
            let (start_line, start_char) = index_to_line_col(file_content, absolute_start_idx);
            let (end_line, end_char) = index_to_line_col(file_content, absolute_end_idx);
            return Ok((start_line, start_char, end_line, end_char));
        }

        current_pos = absolute_start_idx + 1;
    }

    Err(anyhow::anyhow!(
        "Code block not found (occurrence #{}) in file. Ensure the code block is an exact match.",
        occurrence
    ))
}

pub fn find_symbol_location(
    file_content: &str,
    symbol: &str,
    code_block: &str,
    occurrence: usize,
) -> Result<(u32, u32)> {
    // Find the code block
    // We assume the LLM copies the block accurately.
    let block_start_idx = file_content
        .find(code_block)
        .ok_or_else(|| anyhow::anyhow!("Code block not found in file. Ensure the code block is an exact match."))?;

    // Find the symbol within the code block
    let block_content = &file_content[block_start_idx..block_start_idx + code_block.len()];
    
    let mut current_occurrence = 0;
    let mut symbol_offset_in_block = 0;
    let mut found = false;
    
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';

    for (idx, _) in block_content.match_indices(symbol) {
        // Check boundary before
        let valid_start = if idx == 0 {
            true
        } else {
            // Get the character immediately preceding the match
            block_content[..idx].chars().next_back().map_or(true, |c| !is_ident_char(c))
        };

        // Check boundary after
        let valid_end = if idx + symbol.len() == block_content.len() {
            true
        } else {
            block_content[idx + symbol.len()..].chars().next().map_or(true, |c| !is_ident_char(c))
        };

        let absolute_symbol_idx = block_start_idx + idx;
        let is_code = is_valid_code_context(file_content, absolute_symbol_idx);

        if valid_start && valid_end && is_code {
            current_occurrence += 1;
            if current_occurrence == occurrence {
                symbol_offset_in_block = idx;
                found = true;
                break;
            }
        }
    }

    if !found {
        return Err(anyhow::anyhow!(
            "Found only {} occurrences of symbol '{}' (whole word, not in comment/string) in the code block, expected #{}", 
            current_occurrence, 
            symbol, 
            occurrence
        ));
    }

    let absolute_symbol_idx = block_start_idx + symbol_offset_in_block;

    // Convert index to line and character (LSP compatible)
    Ok(index_to_line_col(file_content, absolute_symbol_idx))
}

fn index_to_line_col(text: &str, index: usize) -> (u32, u32) {
    let prefix = &text[..index];
    let line = prefix.matches('\n').count() as u32;
    
    let last_newline_pos = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_str = &prefix[last_newline_pos..];
    
    // Convert to UTF-16 code units for LSP
    let character = line_str.encode_utf16().count() as u32;
    
    (line, character)
}

fn is_valid_code_context(text: &str, target_idx: usize) -> bool {
    let mut chars = text.char_indices().peekable();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0;
    
    while let Some((idx, c)) = chars.next() {
        if idx >= target_idx {
             return !in_string && !in_line_comment && block_comment_depth == 0;
        }

        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        
        if block_comment_depth > 0 {
             if c == '/' {
                if let Some((_, '*')) = chars.peek() {
                    chars.next();
                    block_comment_depth += 1;
                }
            } else if c == '*' {
                if let Some((_, '/')) = chars.peek() {
                    chars.next();
                    block_comment_depth -= 1;
                }
            }
            continue;
        }
        
        if in_string {
            if c == '\\' {
                chars.next();
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        
        match c {
            '/' => {
                if let Some((_, '/')) = chars.peek() {
                    chars.next();
                    in_line_comment = true;
                } else if let Some((_, '*')) = chars.peek() {
                    chars.next();
                    block_comment_depth += 1;
                }
            }
            '"' => {
                in_string = true;
            }
            _ => {}
        }
    }
    
    !in_string && !in_line_comment && block_comment_depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_to_line_col() {
        let text = "hello\nworld\n! 123";
        // "h" -> 0, 0
        assert_eq!(index_to_line_col(text, 0), (0, 0));
        // "w" in "world" -> index 6 (hello=5 + \n=1)
        assert_eq!(index_to_line_col(text, 6), (1, 0));
        // "!" -> index 12 (hello\n=6, world\n=6)
        assert_eq!(index_to_line_col(text, 12), (2, 0));
        // "1" -> index 14
        assert_eq!(index_to_line_col(text, 14), (2, 2));
    }

    #[test]
    fn test_unicode() {
        let text = "a\nб\nв"; // 'б' is 2 bytes
        // 'a' (0) -> 0,0
        // '\n' (1)
        // 'б' (2) -> 1,0
        assert_eq!(index_to_line_col(text, 2), (1, 0));
        // '\n' after 'б' is at 2+2=4.
        // 'в' at 5.
        assert_eq!(index_to_line_col(text, 5), (2, 0));
    }

    #[test]
    fn test_is_valid_code_context() {
        let code = "
        let x = 1; // comment with x
        let s = \"string with x\";
        /* 
           multi-line comment with x
        */
        let x = 2;
        ";
        
        // Find all 'x' indices manually
        // x at line 2: "let x"
        let first_x = code.find("let x").unwrap() + 4;
        assert!(is_valid_code_context(code, first_x));
        
        // x in comment: "// comment with x"
        let comment_start = code.find("//").unwrap();
        let x_in_comment = comment_start + code[comment_start..].find("x").unwrap();
        assert!(!is_valid_code_context(code, x_in_comment));
        
        // x in string: "\"string with x\""
        let string_start = code.find("\"").unwrap();
        let x_in_string = string_start + code[string_start..].find("x").unwrap();
        assert!(!is_valid_code_context(code, x_in_string));
        
        // x in block comment
        let block_start = code.find("/*").unwrap();
        let x_in_block = block_start + code[block_start..].find("x").unwrap();
        assert!(!is_valid_code_context(code, x_in_block));

        // last x
        let last_let = code.rfind("let x").unwrap();
        let last_x = last_let + 4;
        assert!(is_valid_code_context(code, last_x));
    }

    #[tokio::test]
    async fn test_word_boundary_logic() {
        // This simulates the logic inside get_hover_impl
        let block_content = "let service = rust_server.serve(stdio()).await?;";
        let symbol = "serve";
        let occurrence = 1;

        let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';
        
        let mut current_occurrence = 0;
        let mut found_idx = None;

        for (idx, _) in block_content.match_indices(symbol) {
             let valid_start = if idx == 0 {
                true
            } else {
                block_content[..idx].chars().next_back().map_or(true, |c| !is_ident_char(c))
            };

            let valid_end = if idx + symbol.len() == block_content.len() {
                true
            } else {
                block_content[idx + symbol.len()..].chars().next().map_or(true, |c| !is_ident_char(c))
            };

            if valid_start && valid_end {
                current_occurrence += 1;
                if current_occurrence == occurrence {
                    found_idx = Some(idx);
                    break;
                }
            }
        }

        // "rust_server" starts at 14. "serve" inside it starts at 19.
        // ".serve" starts at 25 (dot) -> serve at 26.
        // match_indices("serve") will return indices: 19, 26.
        
        // 1. Index 19: char before is '_'. is_ident_char('_') is true. !true is false. valid_start = false. SKIP.
        // 2. Index 26: char before is '.'. is_ident_char('.') is false. valid_start = true.
        //              char after is '('. is_ident_char('(') is false. valid_end = true. MATCH.
        
        assert_eq!(found_idx, Some(26), "Should find the standalone 'serve', skipping 'rust_server'");
    }
}


pub async fn get_symbol_source_impl(
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
    
    // Try to read the line content for context (even if get_symbol_source fails)
    let context_line = file_content
        .lines()
        .nth(line as usize)
        .unwrap_or("<line out of bounds>")
        .to_string();

    let context_marker = create_position_marker(&context_line, character as u32);

    match analyzer
        .get_symbol_source(file_path, line as u32, character as u32)
        .await
    {
        Ok((source, range, actual_path)) => {
            let result = json!({
                "request": {
                    "file_path": file_path,
                    "symbol": symbol,
                    "occurrence": occurrence,
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
