use serde::Serialize;
use std::collections::BTreeMap;

use crate::consumer;
use crate::manifest_access::ManifestView;
use crate::mcp::{self, McpRoute};
use crate::registry;

pub struct RouterOptions {
    pub command: RouterCommand,
}

pub enum RouterCommand {
    Serve { dry_run: bool },
    Status { json: bool },
}

pub enum RouterOutcome {
    Output { output: String, success: bool },
    Served,
}

#[derive(Debug)]
struct RouterError {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct RouterStatusView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    router: RouterSnapshotView,
    tools: Vec<RouterToolView>,
    resources: Vec<RouterResourceView>,
    error: Option<RouterErrorView>,
}

#[derive(Debug, Serialize)]
struct RouterSnapshotView {
    snapshot: bool,
    capsules: usize,
}

#[derive(Debug, Serialize)]
struct RouterToolView {
    capsule_id: String,
    capsule_path: String,
    name: String,
    manifest_sha256: String,
}

#[derive(Debug, Serialize)]
struct RouterResourceView {
    capsule_id: String,
    uri_prefix: String,
}

#[derive(Debug, Serialize)]
struct RouterErrorView {
    code: &'static str,
    message: String,
}

pub fn run(options: &RouterOptions) -> Result<RouterOutcome, String> {
    match options.command {
        RouterCommand::Serve { dry_run } => serve_mcp(dry_run),
        RouterCommand::Status { json } => status(json),
    }
}

fn serve_mcp(dry_run: bool) -> Result<RouterOutcome, String> {
    let routes = match build_routes() {
        Ok(routes) => routes,
        Err(error) if dry_run => {
            let output = mcp::router_error_contract(
                "router serve --mcp --dry-run",
                "router.mcp.v1",
                error.code,
                &error.message,
            )?;
            return Ok(RouterOutcome::Output {
                output,
                success: false,
            });
        }
        Err(error) => return Err(error.message),
    };

    if dry_run {
        return Ok(RouterOutcome::Output {
            output: mcp::router_dry_run_contract(&routes)?,
            success: true,
        });
    }

    mcp::serve_router_stdio(&routes)?;
    Ok(RouterOutcome::Served)
}

fn status(json: bool) -> Result<RouterOutcome, String> {
    let (routes, error, success) = match build_routes() {
        Ok(routes) => (routes, None, true),
        Err(error) => (Vec::new(), Some(error), false),
    };

    if json {
        return Ok(RouterOutcome::Output {
            output: router_status_json(&routes, error.as_ref())?,
            success,
        });
    }

    let output = match error {
        Some(error) => format!(
            "SkillRun Router\nstatus: error\ncode: {}\nmessage: {}",
            error.code, error.message
        ),
        None => format!(
            "SkillRun Router\nstatus: ok\ncapsules: {}\nentry: skillrun router serve --mcp",
            routes.len()
        ),
    };

    Ok(RouterOutcome::Output { output, success })
}

fn build_routes() -> Result<Vec<McpRoute>, RouterError> {
    let exposed = registry::exposed_capsules().map_err(|error| RouterError {
        code: "registry-unavailable",
        message: error,
    })?;
    let mut routes = Vec::new();
    let mut tool_names = BTreeMap::new();

    for capsule in exposed {
        let manifest =
            consumer::validate(&capsule.path, "skillrun router serve --mcp").map_err(|error| {
                RouterError {
                    code: "capsule-validation-failed",
                    message: format!("cannot expose {}: {error}", capsule.id),
                }
            })?;
        let manifest_view = ManifestView::new(&manifest.value);
        let skill_name = manifest_view.skill_name().unwrap_or("skill");
        let tool_name = manifest_view.tool_name().unwrap_or(skill_name).to_string();

        if tool_name != capsule.tool_name {
            return Err(RouterError {
                code: "tool-snapshot-mismatch",
                message: format!(
                    "cannot expose {}: registry tool snapshot {} does not match Manifest tool {}",
                    capsule.id, capsule.tool_name, tool_name
                ),
            });
        }
        if manifest.sha256 != capsule.manifest_hash {
            return Err(RouterError {
                code: "manifest-snapshot-mismatch",
                message: format!(
                    "cannot expose {}: registry Manifest hash snapshot does not match Consumer Mode validation",
                    capsule.id
                ),
            });
        }
        if let Some(existing) = tool_names.insert(tool_name.clone(), capsule.id.clone()) {
            return Err(RouterError {
                code: "duplicate-tool-name",
                message: format!(
                    "cannot start Router: duplicate MCP tool name {tool_name} from capsules {existing} and {}",
                    capsule.id
                ),
            });
        }

        routes.push(McpRoute {
            capsule_id: capsule.id,
            capsule_dir: capsule.path,
            manifest,
        });
    }

    Ok(routes)
}

fn router_status_json(routes: &[McpRoute], error: Option<&RouterError>) -> Result<String, String> {
    let tools = routes
        .iter()
        .map(|route| {
            let manifest_view = ManifestView::new(&route.manifest.value);
            let skill_name = manifest_view.skill_name().unwrap_or("skill");
            RouterToolView {
                capsule_id: route.capsule_id.clone(),
                capsule_path: route.capsule_dir.display().to_string(),
                name: manifest_view.tool_name().unwrap_or(skill_name).to_string(),
                manifest_sha256: route.manifest.sha256.clone(),
            }
        })
        .collect::<Vec<_>>();
    let resources = routes
        .iter()
        .map(|route| RouterResourceView {
            capsule_id: route.capsule_id.clone(),
            uri_prefix: format!("skillrun://router/{}/", route.capsule_id),
        })
        .collect::<Vec<_>>();
    let view = RouterStatusView {
        command: "router status",
        schema_version: "router.status.v1",
        ok: error.is_none(),
        router: RouterSnapshotView {
            snapshot: true,
            capsules: routes.len(),
        },
        tools,
        resources,
        error: error.map(|error| RouterErrorView {
            code: error.code,
            message: error.message.clone(),
        }),
    };

    serde_json::to_string_pretty(&view)
        .map_err(|error| format!("failed to serialize Router status: {error}"))
}
