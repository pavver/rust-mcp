# 🦀 Rust MCP Server

[![CI](https://github.com/dexwritescode/rust-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/dexwritescode/rust-mcp/actions/workflows/ci.yml)
![Status: Alpha](https://img.shields.io/badge/status-alpha-red.svg)
![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![MCP Protocol](https://img.shields.io/badge/MCP-2024.11.05-blue.svg)
![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

**Supercharge your AI assistant with professional-grade Rust intelligence.**

Standard AI tools only see your files as text. **Rust MCP** acts as a semantic bridge, giving LLMs (like Claude, GPT-4, or local models) the same deep understanding that `rust-analyzer` provides to your IDE. It enables AI tools to work with Rust code idiomatically through Language Server Protocol capabilities, avoiding brittle string manipulation and providing intelligent code analysis.

> [!WARNING]  
> This project is currently in **Alpha**. It is functional and useful, but expect breaking changes and occasional rough edges. **However, most core tools are fully tested and ready for production-level development tasks.** We welcome feedback and contributions!

## 🌟 Why this server?

While other MCP servers provide basic file access, **Rust MCP** focuses on **Semantic Context**:

- **Context-Aware Refactoring**: Rename symbols reliably using code snippets instead of brittle line numbers.
- **Smart Navigation**: Find real definitions and references, not just text matches.
- **Compiler-Grade Feedback**: Get actual `cargo check` diagnostics directly in your chat.
- **Efficiency**: Outline large files with `document_symbols` to save LLM tokens.

## Quick Start

1. **Build**: `cargo build --release`
2. **Configure**: See the [Configuration](#configuration) section below.
3. **Use**: Through AI assistants with natural language prompts like "Show me the definition of the main function".

## Features - Core Tool Suite

### Code Analysis & Navigation (Context-Aware)
- `get_hover` - Get symbol signature and documentation.
- `get_symbol_source` - Get source code of specific symbol.
- `document_symbols` - Get file structure (outline) - **Recommended for large files**.
- `find_definition` - Navigate to symbol definitions.
- `find_references` - Find all symbol uses.
- `get_diagnostics` - Get compiler errors/warnings for a specific file.
- `workspace_symbols` - Search project symbols.
- `get_type_hierarchy` - Get type relationships for symbols.

### Refactoring
- `rename_symbol` - Rename with scope awareness (context-aware).
- `extract_function` - (Experimental) Extract code into functions.
- `inline_function` - (Experimental) Inline function calls.

### Quality Assurance & Project Management
- `run_cargo_check` - Execute cargo check with full error parsing.
- `apply_clippy_suggestions` - (In Progress) Apply clippy automatic fixes.

## Prerequisites

- Rust toolchain (1.70+)
- `rust-analyzer` installed (defaults to `~/.cargo/bin/rust-analyzer`)
- An MCP-compatible client (Claude Desktop, Gemini CLI, Roo-Code, etc.)

## Installation

```bash
git clone https://github.com/dexwritescode/rust-mcp
cd rust-mcp
cargo build --release
```

The server binary will be available at `target/release/rustmcp`.

## Configuration

Detailed guides for setting up Rust MCP with various clients:

- [**Claude Desktop**](./docs/configuration/claude-desktop.md)
- [**Gemini CLI**](./docs/configuration/gemini-cli.md)
- [**Roo-Code (VS Code)**](./docs/configuration/roo-code.md)
- [**ESP32 & Custom Toolchains**](./docs/configuration/esp32.md)
- [**Other MCP Clients (Cursor, etc.)**](./docs/configuration/other-clients.md)
- [**Environment Variables**](./docs/configuration/environment-variables.md)

## 💡 Usage Examples

These examples show how you can interact with your AI assistant once Rust MCP is configured.

### 🔍 Deep Code Analysis
- "Find the definition of the `AppState` struct and show me its source code."
- "What does the `handle_request` function do? Show me its signature and documentation."
- "Where else is the `UserRegistry` trait used or implemented in this workspace?"
- "Give me a high-level outline of `src/lib.rs` so I can understand its structure."

### 🛠 Semantic Refactoring
- "I want to rename the internal field `count` to `total_processed`. Here is the code block where it's defined..."
- "Please rename the `Storage` trait to `DataStore` across the entire project."
- "Help me rename this local variable `idx` to `index` to improve readability."

### ⚡ Troubleshooting & Project Health
- "Run a full `cargo check` and tell me if my recent changes introduced any new warnings."
- "Check the current file for any borrow checker errors."
- "Show me the type hierarchy for `MyCustomError` to see which traits it implements."

## Architecture

The server is built with a modular architecture:
- `src/analyzer/` - `rust-analyzer` LSP client integration.
- `src/server/` - MCP server implementation and tool handlers.
- `src/tools/` - Modular tool logic.

## Compiler Safety Limits and Errors

To keep the server responsive, compilation helpers run with guardrails (30s timeout, 1MB output cap). See [Compiler Safety](./docs/configuration/environment-variables.md) for more info.

## Troubleshooting

See the [Troubleshooting section in individual guides](./docs/configuration/) or check that `rust-analyzer` is installed and accessible.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes with tests
4. Submit a pull request

---
*Built with ❤️ for the Rust & AI ecosystem.*
