use std::path::PathBuf;

pub(super) struct ClientSpec {
    pub(super) id: &'static str,
    pub(super) name: &'static str,
    pub(super) default_config: PathBuf,
}

pub(super) fn selected_config_path(config: &Option<PathBuf>, spec: &ClientSpec) -> PathBuf {
    config
        .clone()
        .unwrap_or_else(|| spec.default_config.clone())
}

pub(super) fn client_spec(client: &str) -> Option<ClientSpec> {
    match client {
        "claude-desktop" => Some(ClientSpec {
            id: "claude-desktop",
            name: "Claude Desktop",
            default_config: appdata_path(&["Claude", "claude_desktop_config.json"])
                .unwrap_or_else(|| PathBuf::from("claude_desktop_config.json")),
        }),
        "cursor" => Some(ClientSpec {
            id: "cursor",
            name: "Cursor",
            default_config: home_path(&[".cursor", "mcp.json"])
                .unwrap_or_else(|| PathBuf::from("mcp.json")),
        }),
        _ => None,
    }
}

fn appdata_path(parts: &[&str]) -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| join_parts(PathBuf::from(root), parts))
}

fn home_path(parts: &[&str]) -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|root| join_parts(PathBuf::from(root), parts))
}

fn join_parts(mut root: PathBuf, parts: &[&str]) -> PathBuf {
    for part in parts {
        root.push(part);
    }
    root
}
