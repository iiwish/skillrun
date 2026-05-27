use std::path::PathBuf;

pub(super) struct ClientSpec {
    pub(super) id: &'static str,
    pub(super) name: &'static str,
    pub(super) default_config: Option<PathBuf>,
}

pub(super) fn selected_config_path(config: &Option<PathBuf>, spec: &ClientSpec) -> Option<PathBuf> {
    config.clone().or_else(|| spec.default_config.clone())
}

pub(super) fn client_spec(client: &str) -> Option<ClientSpec> {
    match client {
        "claude-desktop" => Some(ClientSpec {
            id: "claude-desktop",
            name: "Claude Desktop",
            default_config: claude_desktop_config_path(),
        }),
        "cursor" => Some(ClientSpec {
            id: "cursor",
            name: "Cursor",
            default_config: home_path(&[".cursor", "mcp.json"]),
        }),
        _ => None,
    }
}

fn claude_desktop_config_path() -> Option<PathBuf> {
    appdata_path(&["Claude", "claude_desktop_config.json"]).or_else(|| {
        if cfg!(target_os = "macos") {
            home_path(&[
                "Library",
                "Application Support",
                "Claude",
                "claude_desktop_config.json",
            ])
        } else {
            xdg_config_path(&["Claude", "claude_desktop_config.json"])
                .or_else(|| home_path(&[".config", "Claude", "claude_desktop_config.json"]))
        }
    })
}

fn appdata_path(parts: &[&str]) -> Option<PathBuf> {
    env_path("APPDATA").map(|root| join_parts(root, parts))
}

fn xdg_config_path(parts: &[&str]) -> Option<PathBuf> {
    env_path("XDG_CONFIG_HOME").map(|root| join_parts(root, parts))
}

fn home_path(parts: &[&str]) -> Option<PathBuf> {
    env_path("USERPROFILE")
        .or_else(|| env_path("HOME"))
        .map(|root| join_parts(root, parts))
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}

fn join_parts(mut root: PathBuf, parts: &[&str]) -> PathBuf {
    for part in parts {
        root.push(part);
    }
    root
}
