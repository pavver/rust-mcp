# Using Rust MCP with Other Clients

The Rust MCP server follows the standard Model Context Protocol and uses **stdio** as the transport mechanism. This makes it compatible with most MCP clients.

## General Connection Parameters

- **Command**: Absolute path to the `rustmcp` binary (e.g., `/home/user/rust-mcp/target/release/rustmcp`).
- **Arguments**: None required.
- **Environment**: Ensure `PATH` includes the directory for `rustc` and `cargo` if they are not in standard locations.

## Cursor

Cursor is increasingly supporting MCP. You can add the Rust MCP server in the Cursor settings under the "Features" or "MCP" section by providing the path to the binary.

## Custom Implementations (TypeScript/Python)

If you are building your own MCP client, you can connect to the server using the MCP SDK.

### Example (TypeScript SDK)

```typescript
const client = new Client({
  name: "my-rust-client",
  version: "1.0.0"
}, {
  capabilities: {}
});

const transport = new StdioClientTransport({
  command: "/path/to/rustmcp"
});

await client.connect(transport);
```
