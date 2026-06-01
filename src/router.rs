use serde::Serialize;
use serde_json::Value as JsonValue;
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

#[derive(Debug, Clone)]
struct RouterError {
    code: &'static str,
    message: String,
}

struct RouterReport {
    routes: Vec<McpRoute>,
    diagnostics: Vec<RouterRouteDiagnostic>,
    issues: Vec<RouterIssueView>,
}

#[derive(Debug, Serialize)]
struct RouterStatusView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    router: RouterSnapshotView,
    tools: Vec<RouterToolView>,
    resources: Vec<RouterResourceView>,
    routes: Vec<RouterRouteView>,
    issues: Vec<RouterIssueView>,
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

#[derive(Debug, Clone, Serialize)]
struct RouterIssueView {
    code: &'static str,
    severity: &'static str,
    message: String,
    capsule_id: Option<String>,
    tool_name: Option<String>,
    recommended_action: String,
}

struct RouterRouteDiagnostic {
    capsule_id: String,
    capsule_path: String,
    enabled: bool,
    readiness_status: String,
    readiness_reason: Option<String>,
    tool_name: Option<String>,
    manifest_sha256: Option<String>,
    issue: Option<RouterIssueView>,
    route: Option<McpRoute>,
}

#[derive(Debug, Serialize)]
struct RouterRouteView {
    capsule_id: String,
    capsule_path: String,
    enabled: bool,
    state: &'static str,
    readiness_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    readiness_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifest_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issue: Option<RouterIssueView>,
    recommended_action: String,
}

pub fn run(options: &RouterOptions) -> Result<RouterOutcome, String> {
    match options.command {
        RouterCommand::Serve { dry_run } => serve_mcp(dry_run),
        RouterCommand::Status { json } => status(json),
    }
}

fn serve_mcp(dry_run: bool) -> Result<RouterOutcome, String> {
    let report = match build_route_report() {
        Ok(report) => report,
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
        let success = report.ok();
        let error = report.error_view_value()?;
        let routes = if success {
            report.routes.as_slice()
        } else {
            &[]
        };
        return Ok(RouterOutcome::Output {
            output: mcp::router_dry_run_contract_with_diagnostics(
                if success {
                    "router serve --mcp"
                } else {
                    "router serve --mcp --dry-run"
                },
                success,
                routes,
                report.route_views_value()?,
                report.issues_value()?,
                error,
            )?,
            success,
        });
    }

    if let Some(error) = report.error() {
        return Err(error.message);
    }

    mcp::serve_router_stdio(&report.routes)?;
    Ok(RouterOutcome::Served)
}

fn status(json: bool) -> Result<RouterOutcome, String> {
    let (report, error) = match build_route_report() {
        Ok(report) => {
            let error = report.error();
            (report, error)
        }
        Err(error) => (RouterReport::empty(), Some(error)),
    };
    let success = error.is_none();

    if json {
        return Ok(RouterOutcome::Output {
            output: router_status_json(&report, error.as_ref())?,
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
            report.routes.len()
        ),
    };

    Ok(RouterOutcome::Output { output, success })
}

fn build_route_report() -> Result<RouterReport, RouterError> {
    let candidates = registry::router_candidates().map_err(|error| RouterError {
        code: "registry-unavailable",
        message: error,
    })?;
    let mut diagnostics = Vec::new();
    let mut tool_names = BTreeMap::new();

    for capsule in candidates {
        let mut diagnostic = RouterRouteDiagnostic {
            capsule_id: capsule.id.clone(),
            capsule_path: capsule.path.display().to_string(),
            enabled: capsule.enabled,
            readiness_status: capsule.readiness_status.clone(),
            readiness_reason: capsule.readiness_reason.clone(),
            tool_name: capsule.tool_name.clone(),
            manifest_sha256: capsule.manifest_hash.clone(),
            issue: None,
            route: None,
        };

        if !capsule.readiness_ok {
            diagnostic.issue = Some(RouterIssueView {
                code: "capsule-not-ready",
                severity: "warning",
                message: format!(
                    "cannot expose {}: readiness status is {}",
                    capsule.id, capsule.readiness_status
                ),
                capsule_id: Some(capsule.id.clone()),
                tool_name: capsule.tool_name.clone(),
                recommended_action: capsule.readiness_next_step,
            });
            diagnostics.push(diagnostic);
            continue;
        }

        let Some(snapshot_tool_name) = capsule.tool_name.clone() else {
            diagnostic.issue = Some(RouterIssueView {
                code: "tool-contract-missing",
                severity: "warning",
                message: format!(
                    "cannot expose {}: Manifest does not declare a tool name",
                    capsule.id
                ),
                capsule_id: Some(capsule.id.clone()),
                tool_name: None,
                recommended_action: format!(
                    "Regenerate the Manifest with `skillrun manifest --cwd {}` or disable this capsule.",
                    capsule.path.display()
                ),
            });
            diagnostics.push(diagnostic);
            continue;
        };

        let Some(snapshot_manifest_hash) = capsule.manifest_hash.clone() else {
            diagnostic.issue = Some(RouterIssueView {
                code: "manifest-snapshot-missing",
                severity: "error",
                message: format!(
                    "cannot expose {}: registry Manifest hash snapshot is missing",
                    capsule.id
                ),
                capsule_id: Some(capsule.id.clone()),
                tool_name: Some(snapshot_tool_name),
                recommended_action: format!(
                    "Regenerate the Manifest with `skillrun manifest --cwd {}` or disable this capsule.",
                    capsule.path.display()
                ),
            });
            diagnostics.push(diagnostic);
            continue;
        };

        let manifest = match consumer::validate(&capsule.path, "skillrun router serve --mcp") {
            Ok(manifest) => manifest,
            Err(error) => {
                diagnostic.issue = Some(RouterIssueView {
                    code: "capsule-validation-failed",
                    severity: "error",
                    message: format!("cannot expose {}: {error}", capsule.id),
                    capsule_id: Some(capsule.id.clone()),
                    tool_name: Some(snapshot_tool_name),
                    recommended_action: format!(
                        "Run `skillrun check --cwd {}` and fix the reported issue, or disable this capsule.",
                        capsule.path.display()
                    ),
                });
                diagnostics.push(diagnostic);
                continue;
            }
        };
        let manifest_view = ManifestView::new(&manifest.value);
        let skill_name = manifest_view.skill_name().unwrap_or("skill");
        let tool_name = manifest_view.tool_name().unwrap_or(skill_name).to_string();

        if tool_name != snapshot_tool_name {
            diagnostic.issue = Some(RouterIssueView {
                code: "tool-snapshot-mismatch",
                severity: "error",
                message: format!(
                    "cannot expose {}: registry tool snapshot {} does not match Manifest tool {}",
                    capsule.id, snapshot_tool_name, tool_name
                ),
                capsule_id: Some(capsule.id.clone()),
                tool_name: Some(snapshot_tool_name),
                recommended_action: format!(
                    "Regenerate the Manifest with `skillrun manifest --cwd {}` or disable this capsule.",
                    capsule.path.display()
                ),
            });
            diagnostics.push(diagnostic);
            continue;
        }
        if manifest.sha256 != snapshot_manifest_hash {
            diagnostic.issue = Some(RouterIssueView {
                code: "manifest-snapshot-mismatch",
                severity: "error",
                message: format!(
                    "cannot expose {}: registry Manifest hash snapshot does not match Consumer Mode validation",
                    capsule.id
                ),
                capsule_id: Some(capsule.id.clone()),
                tool_name: Some(tool_name.clone()),
                recommended_action: format!(
                    "Regenerate the Manifest with `skillrun manifest --cwd {}` or re-add this capsule to the registry.",
                    capsule.path.display()
                ),
            });
            diagnostics.push(diagnostic);
            continue;
        }
        tool_names
            .entry(tool_name.clone())
            .or_insert_with(Vec::new)
            .push(diagnostics.len());

        diagnostic.tool_name = Some(tool_name);
        diagnostic.manifest_sha256 = Some(manifest.sha256.clone());
        diagnostic.route = Some(McpRoute {
            capsule_id: capsule.id,
            capsule_dir: capsule.path,
            manifest,
        });
        diagnostics.push(diagnostic);
    }

    for (tool_name, route_indices) in tool_names {
        if route_indices.len() <= 1 {
            continue;
        }

        let capsule_ids = route_indices
            .iter()
            .map(|index| diagnostics[*index].capsule_id.clone())
            .collect::<Vec<_>>();
        let message = format!(
            "cannot start Router: duplicate MCP tool name {tool_name} from capsules {}",
            capsule_ids.join(" and ")
        );
        for index in route_indices {
            let capsule_id = diagnostics[index].capsule_id.clone();
            diagnostics[index].issue = Some(RouterIssueView {
                code: "duplicate-tool-name",
                severity: "error",
                message: message.clone(),
                capsule_id: Some(capsule_id),
                tool_name: Some(tool_name.clone()),
                recommended_action: "Disable one capsule with `skillrun switchboard disable <id>` or rename one Manifest tool, then regenerate and re-add the capsule.".to_string(),
            });
            diagnostics[index].route = None;
        }
    }

    let issues = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.issue.clone())
        .collect::<Vec<_>>();
    let has_fatal_issue = issues.iter().any(|issue| issue.severity == "error");
    let routes = if has_fatal_issue {
        Vec::new()
    } else {
        diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.route.clone())
            .collect()
    };

    Ok(RouterReport {
        routes,
        diagnostics,
        issues,
    })
}

fn router_status_json(
    report: &RouterReport,
    error: Option<&RouterError>,
) -> Result<String, String> {
    let tools = report
        .routes
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
    let resources = report
        .routes
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
            capsules: report.routes.len(),
        },
        tools,
        resources,
        routes: report.route_views(),
        issues: report.issues.clone(),
        error: error.map(|error| RouterErrorView {
            code: error.code,
            message: error.message.clone(),
        }),
    };

    serde_json::to_string_pretty(&view)
        .map_err(|error| format!("failed to serialize Router status: {error}"))
}

impl RouterReport {
    fn empty() -> Self {
        Self {
            routes: Vec::new(),
            diagnostics: Vec::new(),
            issues: Vec::new(),
        }
    }

    fn ok(&self) -> bool {
        self.error().is_none()
    }

    fn error(&self) -> Option<RouterError> {
        self.issues
            .iter()
            .find(|issue| issue.severity == "error")
            .map(|issue| RouterError {
                code: issue.code,
                message: issue.message.clone(),
            })
    }

    fn route_views(&self) -> Vec<RouterRouteView> {
        self.diagnostics
            .iter()
            .map(RouterRouteDiagnostic::view)
            .collect()
    }

    fn route_views_value(&self) -> Result<JsonValue, String> {
        serde_json::to_value(self.route_views())
            .map_err(|error| format!("failed to serialize Router route diagnostics: {error}"))
    }

    fn issues_value(&self) -> Result<JsonValue, String> {
        serde_json::to_value(&self.issues)
            .map_err(|error| format!("failed to serialize Router issues: {error}"))
    }

    fn error_view_value(&self) -> Result<JsonValue, String> {
        match self.error() {
            Some(error) => serde_json::to_value(RouterErrorView {
                code: error.code,
                message: error.message,
            })
            .map_err(|error| format!("failed to serialize Router error: {error}")),
            None => Ok(JsonValue::Null),
        }
    }
}

impl RouterRouteDiagnostic {
    fn view(&self) -> RouterRouteView {
        let state = if self.issue.is_some() {
            "blocked"
        } else {
            "routable"
        };
        let uri_prefix = if self.issue.is_none() {
            Some(format!("skillrun://router/{}/", self.capsule_id))
        } else {
            None
        };
        let recommended_action = self
            .issue
            .as_ref()
            .map(|issue| issue.recommended_action.clone())
            .unwrap_or_else(|| "Mount `skillrun router serve --mcp` in the MCP client when you want this capsule exposed.".to_string());

        RouterRouteView {
            capsule_id: self.capsule_id.clone(),
            capsule_path: self.capsule_path.clone(),
            enabled: self.enabled,
            state,
            readiness_status: self.readiness_status.clone(),
            readiness_reason: self.readiness_reason.clone(),
            tool_name: self.tool_name.clone(),
            manifest_sha256: self.manifest_sha256.clone(),
            uri_prefix,
            issue: self.issue.clone(),
            recommended_action,
        }
    }
}
