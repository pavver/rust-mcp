use rmcp::schemars;

// Parameter structs for tools
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindDefinitionParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindReferencesParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetHoverParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetDiagnosticsParams {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetDocumentSymbolsParams {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetSymbolSourceParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WorkspaceSymbolsParams {
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenameSymbolParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
    pub new_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunCargoCheckParams {
    pub workspace_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExtractFunctionParams {
    pub file_path: String,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub function_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InlineFunctionParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApplyClippySuggestionsParams {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetTypeHierarchyParams {
    pub file_path: String,
    pub symbol: String,
    pub code_block: String,
    pub occurrence: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InspectMirParams {
    pub file_path: String,
    pub line: Option<u32>,
    pub character: Option<u32>,
    pub symbol_name: Option<String>,
    pub opt_level: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InspectLlvmIrParams {
    pub file_path: String,
    pub line: Option<u32>,
    pub character: Option<u32>,
    pub symbol_name: Option<String>,
    pub opt_level: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InspectAsmParams {
    pub file_path: String,
    pub line: Option<u32>,
    pub character: Option<u32>,
    pub symbol_name: Option<String>,
    pub opt_level: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InspectParams {
    pub view: String,
    pub file_path: String,
    pub line: u32,
    pub character: u32,
    pub symbol_name: Option<String>,
    pub opt_level: Option<String>,
    pub target: Option<String>,
    pub gating_mode: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CapabilitiesParams {
    pub gating_mode: Option<String>,
}
