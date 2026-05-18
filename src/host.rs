use serde::Serialize;
use std::path::PathBuf;

use crate::registry;

const MANIFEST_IR_VERSION: &str = "0.1.0";
const IPC_PROTOCOL_VERSION: &str = "0.1.0";
const ADAPTER_PROTOCOL: &str = "adapter.v1";

pub struct HostStatusOptions {
    pub json: bool,
}

pub struct HostStatusOutput {
    pub output: String,
}

#[derive(Debug, Serialize)]
struct HostStatusView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    binary: BinaryView,
    desktop_contract: DesktopContractView,
    contracts: ContractsView,
    platform: PlatformView,
    paths: PathsView,
    capabilities: Vec<CapabilityView>,
    boundaries: BoundariesView,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct BinaryView {
    name: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
struct DesktopContractView {
    name: &'static str,
    version: u32,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ContractsView {
    manifest_ir_version: &'static str,
    ipc_protocol_version: &'static str,
    adapter_protocol: &'static str,
}

#[derive(Debug, Serialize)]
struct PlatformView {
    os: &'static str,
    arch: &'static str,
    family: &'static str,
}

#[derive(Debug, Serialize)]
struct PathsView {
    current_exe: Option<String>,
    skillrun_home: Option<String>,
    registry_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct CapabilityView {
    name: &'static str,
    command: &'static str,
    process: &'static str,
    schema_version: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct BoundariesView {
    desktop_must_use_cli_json: bool,
    desktop_must_not_read_internal_paths: bool,
    desktop_must_not_start_router_as_daemon: bool,
    router_runtime_entry: &'static str,
}

#[derive(Debug, Serialize)]
struct WarningView {
    code: &'static str,
    message: String,
}

pub fn status(options: &HostStatusOptions) -> Result<HostStatusOutput, String> {
    let mut warnings = Vec::new();
    let registry_path = match registry::registry_path_display() {
        Ok(path) => Some(path),
        Err(error) => {
            warnings.push(WarningView {
                code: "registry-path-unavailable",
                message: error,
            });
            None
        }
    };
    let current_exe = std::env::current_exe().ok().map(display_path);
    let skillrun_home = skillrun_home().ok().map(display_path);

    let view = HostStatusView {
        command: "host status",
        schema_version: "host.status.v1",
        ok: true,
        binary: BinaryView {
            name: "skillrun",
            version: env!("CARGO_PKG_VERSION"),
        },
        desktop_contract: DesktopContractView {
            name: "desktop.alpha",
            version: 1,
            status: "frozen",
        },
        contracts: ContractsView {
            manifest_ir_version: MANIFEST_IR_VERSION,
            ipc_protocol_version: IPC_PROTOCOL_VERSION,
            adapter_protocol: ADAPTER_PROTOCOL,
        },
        platform: PlatformView {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            family: std::env::consts::FAMILY,
        },
        paths: PathsView {
            current_exe,
            skillrun_home,
            registry_path,
        },
        capabilities: desktop_capabilities(),
        boundaries: BoundariesView {
            desktop_must_use_cli_json: true,
            desktop_must_not_read_internal_paths: true,
            desktop_must_not_start_router_as_daemon: true,
            router_runtime_entry: "skillrun router serve --mcp",
        },
        warnings,
    };

    if options.json {
        return Ok(HostStatusOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    Ok(HostStatusOutput {
        output: format!(
            "SkillRun Host\nbinary: skillrun {}\ncontracts: manifest {}, ipc {}, adapter {}\nrouter: skillrun router serve --mcp",
            env!("CARGO_PKG_VERSION"),
            MANIFEST_IR_VERSION,
            IPC_PROTOCOL_VERSION,
            ADAPTER_PROTOCOL
        ),
    })
}

fn desktop_capabilities() -> Vec<CapabilityView> {
    vec![
        CapabilityView {
            name: "host_status",
            command: "host status --json",
            process: "short_running",
            schema_version: Some("host.status.v1"),
        },
        CapabilityView {
            name: "import_skr",
            command: "import <package.skr> --json",
            process: "short_running",
            schema_version: Some("import.v1"),
        },
        CapabilityView {
            name: "consumer_inventory",
            command: "consumer inventory --json",
            process: "short_running",
            schema_version: Some("consumer.inventory.v1"),
        },
        CapabilityView {
            name: "consumer_exposure",
            command: "consumer exposure --json",
            process: "short_running",
            schema_version: Some("consumer.exposure.v1"),
        },
        CapabilityView {
            name: "router_dry_run",
            command: "router serve --mcp --dry-run",
            process: "short_running",
            schema_version: Some("router.mcp.v1"),
        },
        CapabilityView {
            name: "router_mcp",
            command: "router serve --mcp",
            process: "long_running_mcp_stdio",
            schema_version: None,
        },
        CapabilityView {
            name: "mount_plan",
            command: "consumer mount plan --client <id> --json",
            process: "short_running",
            schema_version: Some("consumer.mount_plan.v1"),
        },
        CapabilityView {
            name: "mount_apply",
            command: "consumer mount apply --client claude-desktop --json",
            process: "short_running",
            schema_version: Some("consumer.mount_apply.v1"),
        },
        CapabilityView {
            name: "mount_rollback",
            command: "consumer mount rollback --client claude-desktop --backup <path> --json",
            process: "short_running",
            schema_version: Some("consumer.mount_rollback.v1"),
        },
        CapabilityView {
            name: "runs_list",
            command: "consumer runs list --json",
            process: "short_running",
            schema_version: Some("consumer.runs.list.v1"),
        },
        CapabilityView {
            name: "runs_inspect",
            command: "consumer runs inspect <run-id> --json --capsule <id>",
            process: "short_running",
            schema_version: Some("consumer.runs.inspect.v1"),
        },
    ]
}

fn skillrun_home() -> Result<PathBuf, String> {
    if let Some(home) = std::env::var_os("SKILLRUN_HOME") {
        return Ok(PathBuf::from(home));
    }

    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| "SKILLRUN_HOME, USERPROFILE, or HOME must be set".to_string())?;
    Ok(PathBuf::from(home).join(".skillrun"))
}

fn display_path(path: PathBuf) -> String {
    path.display().to_string()
}
