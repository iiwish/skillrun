use std::collections::BTreeMap;

use crate::consumer;
use crate::manifest_access::ManifestView;
use crate::mcp::{self, McpRoute};
use crate::registry;

pub struct RouterOptions {
    pub dry_run: bool,
}

pub enum RouterOutcome {
    DryRun(String),
    Served,
}

pub fn serve_mcp(options: &RouterOptions) -> Result<RouterOutcome, String> {
    let routes = build_routes()?;
    if options.dry_run {
        return mcp::router_dry_run_contract(&routes).map(RouterOutcome::DryRun);
    }

    mcp::serve_router_stdio(&routes)?;
    Ok(RouterOutcome::Served)
}

fn build_routes() -> Result<Vec<McpRoute>, String> {
    let exposed = registry::exposed_capsules()?;
    let mut routes = Vec::new();
    let mut tool_names = BTreeMap::new();

    for capsule in exposed {
        let manifest = consumer::validate(&capsule.path, "skillrun router serve --mcp")
            .map_err(|error| format!("cannot expose {}: {error}", capsule.id))?;
        let manifest_view = ManifestView::new(&manifest.value);
        let skill_name = manifest_view.skill_name().unwrap_or("skill");
        let tool_name = manifest_view.tool_name().unwrap_or(skill_name).to_string();

        if tool_name != capsule.tool_name {
            return Err(format!(
                "cannot expose {}: registry tool snapshot {} does not match Manifest tool {}",
                capsule.id, capsule.tool_name, tool_name
            ));
        }
        if manifest.sha256 != capsule.manifest_hash {
            return Err(format!(
                "cannot expose {}: registry Manifest hash snapshot does not match Consumer Mode validation",
                capsule.id
            ));
        }
        if let Some(existing) = tool_names.insert(tool_name.clone(), capsule.id.clone()) {
            return Err(format!(
                "cannot start Router: duplicate MCP tool name {tool_name} from capsules {existing} and {}",
                capsule.id
            ));
        }

        routes.push(McpRoute {
            capsule_id: capsule.id,
            capsule_dir: capsule.path,
            manifest,
        });
    }

    Ok(routes)
}
