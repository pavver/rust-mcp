# Rust MCP Server: Agent Instruction Guide

This document provides a comprehensive guide for AI Agents interacting with the Rust MCP Server. It details available tools, best practices, and critical operational constraints.

## ⚠️ CRITICAL REQUIREMENT: ABSOLUTE PATHS

**You MUST use ABSOLUTE FILE PATHS for all tools.**
*   **Incorrect:** `src/main.rs`, `./src/lib.rs`
*   **Correct:** `/home/user/project/src/main.rs`

The server will return an error if a relative path is provided. Always resolve the current working directory before calling tools.

## 1. Tool Categories

### 🔍 Code Analysis & Navigation
Use these tools to understand the codebase structure and symbol meanings.

*   **`get_hover`** (PREFERRED for Signatures)
    *   **Purpose:** Retrieves the signature, types, and documentation (doc comments) for a symbol at a specific position.
    *   **Parameters:** `file_path`, `line`, `character`.
    *   **Use Case:** When you need to know how to call a function or what a struct looks like. Returns formatted Markdown.
*   **`find_definition`**
    *   **Purpose:** Locates where a symbol is defined.
    *   **Parameters:** `file_path`, `line`, `character`.
    *   **Note:** Can return paths to external crates (in Cargo registry).
*   **`find_references`**
    *   **Purpose:** Finds all usages of a symbol across the workspace.
*   **`workspace_symbols`**
    *   **Purpose:** Fuzzy search for symbols by name across the entire project.
    *   **Use Case:** You know the name (e.g., "RustMcpServer") but not the location.
*   **`get_diagnostics`**
    *   **Purpose:** Retrieves compilation errors and warnings for a specific file.

### 🛠 Code Generation
Helpers for scaffolding boilerplate code.

*   **`generate_struct`**: Creates structs with fields and derives.
*   **`generate_enum`**: Creates enums with variants.
*   **`generate_trait_impl`**: Generates trait implementation stubs.
*   **`generate_tests`**: Scaffolds unit tests for a specific function.

### ♻️ Refactoring
Tools to modify code structure safely.

*   **`rename_symbol`**: Renames symbols with scope awareness (safe refactoring).
*   **`extract_function`**: Moves selected code into a new function.
*   **`inline_function`**: Replaces a function call with its body.
*   **`organize_imports`**: Sorts and deduplicates `use` statements.
*   **`format_code`**: Applies `cargo fmt` style.

### ✅ Quality Assurance
*   **`apply_clippy_suggestions`**: Automatically fixes common linting errors.
*   **`validate_lifetimes`**: Checks for borrow checker issues.

### 📦 Project Management
*   **`analyze_manifest`**: Parses `Cargo.toml`.
*   **`run_cargo_check`**: Runs compilation check. useful to verify code state if analysis seems broken.

## 2. Best Practices & Workflows

### 🚀 Strategy for External Libraries (Dependencies)
You cannot use `read_file` on files outside the project workspace (e.g., Cargo registry), but you often need to see signatures of external libraries (like `tokio`, `serde`).

**The Protocol:**
1.  **Option A (Best):** Use **`get_hover`** on a usage of the external function in your local code. This provides the signature immediately without file access issues.
2.  **Option B (Fallback):**
    *   Use `find_definition` to get the path (e.g., `/home/user/.cargo/registry/.../lib.rs`).
    *   **DO NOT** use `read_file` (it will fail due to security limits).
    *   **USE** `run_shell_command` with `sed` or `cat` to read the specific lines needed.
        *   *Example:* `run_shell_command("sed -n '400,420p' /abs/path/to/external/lib.rs")`

### 🎯 Positioning Accuracy
LSP tools (`get_hover`, `find_definition`) require exact `line` (0-based) and `character` (0-based) coordinates.
*   If you are unsure of the exact character index, use `read_file` first to inspect the context.
*   Target the **start** of the symbol name.

### 🔄 Troubleshooting
If tools return "No result" or generic errors:
1.  Run `run_cargo_check` to ensure the code compiles. `rust-analyzer` struggles with broken code.
2.  Use `workspace_symbols` to verify the server can "see" the project symbols.