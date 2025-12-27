use crate::analyzer::protocol::SymbolPathSegment;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    FreeFunction,
    Method,
    Trait,
    Impl,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolIdentity {
    pub crate_name: String,
    pub module_path: Vec<String>,
    pub item_name: String,
    pub kind: SymbolKind,
}

pub fn symbol_kind_from_lsp_kind(kind: u32, name_hint: Option<&str>) -> SymbolKind {
    match kind {
        6 => SymbolKind::Method,
        11 => SymbolKind::Trait,
        12 => SymbolKind::FreeFunction,
        23 => SymbolKind::Impl,
        _ => {
            if let Some(name) = name_hint {
                if name.trim_start().starts_with("impl ") {
                    return SymbolKind::Impl;
                }
            }
            SymbolKind::Unknown
        }
    }
}

pub fn identity_from_definition(
    uri: &str,
    symbol_path: &[SymbolPathSegment],
) -> Option<SymbolIdentity> {
    let item_segment = symbol_path.last()?;
    let mut module_path = module_path_from_uri(uri);
    let crate_name = crate_name_from_uri(uri).unwrap_or_else(|| "unknown".to_string());
    let parent_hint = symbol_path
        .iter()
        .rev()
        .nth(1)
        .map(|segment| segment.name.as_str());
    let kind = symbol_kind_from_lsp_kind(item_segment.kind, parent_hint);

    if symbol_path.len() > 1 {
        module_path.extend(
            symbol_path
                .iter()
                .take(symbol_path.len() - 1)
                .map(|segment| segment.name.clone()),
        );
    }

    Some(SymbolIdentity {
        crate_name,
        module_path,
        item_name: item_segment.name.clone(),
        kind,
    })
}

pub fn identities_from_workspace_symbols(response: &Value) -> Vec<SymbolIdentity> {
    let symbol_array = response
        .get("result")
        .and_then(|result| result.as_array())
        .or_else(|| response.as_array());

    symbol_array
        .into_iter()
        .flatten()
        .filter_map(symbol_information_to_identity)
        .collect()
}

pub fn symbol_information_to_identity(symbol_info: &Value) -> Option<SymbolIdentity> {
    let item_name = symbol_info.get("name")?.as_str()?.to_string();
    let location_uri = symbol_info
        .get("location")
        .and_then(|location| location.get("uri"))
        .and_then(|uri| uri.as_str());
    let container_name = symbol_info
        .get("containerName")
        .and_then(|container| container.as_str());

    let kind = parse_symbol_kind(symbol_info.get("kind"), container_name);
    let (crate_name, module_path) = derive_paths(container_name, location_uri);

    Some(SymbolIdentity {
        crate_name,
        module_path,
        item_name,
        kind,
    })
}

fn derive_paths(container_name: Option<&str>, location_uri: Option<&str>) -> (String, Vec<String>) {
    let mut module_path = Vec::new();
    let crate_name = container_name
        .map(normalize_container_name)
        .and_then(|normalized| {
            let mut segments = container_segments(&normalized);
            if segments.is_empty() {
                return None;
            }
            let crate_segment = segments.remove(0);
            if !segments.is_empty() {
                module_path = segments;
            }
            Some(crate_segment)
        })
        .or_else(|| location_uri.and_then(crate_name_from_uri));

    if module_path.is_empty() {
        module_path = location_uri.map(module_path_from_uri).unwrap_or_default();
    }

    let crate_name = crate_name.unwrap_or_else(|| "unknown".to_string());
    (crate_name, module_path)
}

fn parse_symbol_kind(kind_value: Option<&Value>, container_name: Option<&str>) -> SymbolKind {
    let base_kind = match kind_value {
        Some(Value::Number(number)) => match number.as_u64() {
            Some(6) => SymbolKind::Method,
            Some(11) => SymbolKind::Trait,
            Some(12) => SymbolKind::FreeFunction,
            Some(23) => SymbolKind::Impl,
            _ => SymbolKind::Unknown,
        },
        Some(Value::String(kind)) => match kind.to_lowercase().as_str() {
            "method" => SymbolKind::Method,
            "function" | "fn" => SymbolKind::FreeFunction,
            "trait" => SymbolKind::Trait,
            "impl" => SymbolKind::Impl,
            _ => SymbolKind::Unknown,
        },
        _ => SymbolKind::Unknown,
    };

    if matches!(base_kind, SymbolKind::Unknown | SymbolKind::FreeFunction) {
        if let Some(container) = container_name {
            if container.trim_start().starts_with("impl ") {
                return SymbolKind::Impl;
            }
        }
    }

    base_kind
}

fn normalize_container_name(container: &str) -> String {
    container
        .trim()
        .trim_start_matches("::")
        .trim_start_matches("impl ")
        .to_string()
}

fn container_segments(container: &str) -> Vec<String> {
    container
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.trim().to_string())
        .collect()
}

fn path_from_uri(uri: &str) -> Option<PathBuf> {
    let without_scheme = uri.strip_prefix("file://").unwrap_or(uri);
    Some(PathBuf::from(without_scheme))
}

fn crate_name_from_uri(uri: &str) -> Option<String> {
    let path = path_from_uri(uri)?;
    let components: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    if let Some(src_index) = components.iter().position(|component| component == "src") {
        if src_index >= 1 {
            return components.get(src_index - 1).cloned();
        }
    }

    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
}

fn module_path_from_uri(uri: &str) -> Vec<String> {
    let Some(path) = path_from_uri(uri) else {
        return Vec::new();
    };

    let mut after_src = false;
    let mut segments: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => {
                let part_str = part.to_string_lossy().to_string();
                if after_src {
                    Some(part_str)
                } else if part_str == "src" {
                    after_src = true;
                    None
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    if let Some(last) = segments.pop() {
        let stem = Path::new(&last)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned());
        if let Some(stem) = stem {
            if stem != "mod" {
                segments.push(stem);
            }
        }
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::{
        SymbolIdentity, SymbolKind, identities_from_workspace_symbols,
        symbol_information_to_identity,
    };
    use serde_json::json;

    #[test]
    fn parses_free_function_symbol_information() {
        let symbol = json!({
            "name": "do_thing",
            "kind": 12,
            "location": {"uri": "file:///workspace/demo/src/utils/mod.rs"},
            "containerName": "demo::utils"
        });

        let identity = symbol_information_to_identity(&symbol).unwrap();

        assert_eq!(identity.crate_name, "demo");
        assert_eq!(identity.module_path, vec!["utils".to_string()]);
        assert_eq!(identity.item_name, "do_thing");
        assert_eq!(identity.kind, SymbolKind::FreeFunction);
    }

    #[test]
    fn parses_method_symbol_information() {
        let symbol = json!({
            "name": "handle",
            "kind": 6,
            "location": {"uri": "file:///workspace/demo/src/types/item.rs"},
            "containerName": "demo::types::Item"
        });

        let identity = symbol_information_to_identity(&symbol).unwrap();

        assert_eq!(identity.crate_name, "demo");
        assert_eq!(
            identity.module_path,
            vec!["types".to_string(), "Item".to_string()]
        );
        assert_eq!(identity.item_name, "handle");
        assert_eq!(identity.kind, SymbolKind::Method);
    }

    #[test]
    fn infers_impl_from_container_name() {
        let symbol = json!({
            "name": "new",
            "kind": 0,
            "location": {"uri": "file:///workspace/demo/src/types/item.rs"},
            "containerName": "impl demo::types::Item"
        });

        let identity = symbol_information_to_identity(&symbol).unwrap();

        assert_eq!(identity.crate_name, "demo");
        assert_eq!(
            identity.module_path,
            vec!["types".to_string(), "Item".to_string()]
        );
        assert_eq!(identity.item_name, "new");
        assert_eq!(identity.kind, SymbolKind::Impl);
    }

    #[test]
    fn parses_workspace_symbol_response_array() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": [
                {
                    "name": "navigate",
                    "kind": 12,
                    "location": {"uri": "file:///workspace/demo/src/tools/navigation.rs"},
                    "containerName": null
                }
            ]
        });

        let identities = identities_from_workspace_symbols(&response);

        assert_eq!(identities.len(), 1);
        let SymbolIdentity {
            crate_name,
            module_path,
            item_name,
            kind,
        } = identities[0].clone();

        assert_eq!(crate_name, "demo");
        assert_eq!(
            module_path,
            vec!["tools".to_string(), "navigation".to_string()]
        );
        assert_eq!(item_name, "navigate");
        assert_eq!(kind, SymbolKind::FreeFunction);
    }
}
