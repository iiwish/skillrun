use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};

use crate::registry;

pub struct TeamCatalogOptions {
    pub command: TeamCatalogCommand,
}

pub enum TeamCatalogCommand {
    Inspect {
        catalog: PathBuf,
        json: bool,
    },
    InstallPlan {
        catalog: PathBuf,
        item_id: String,
        json: bool,
    },
}

impl TeamCatalogOptions {
    pub fn json(&self) -> bool {
        match &self.command {
            TeamCatalogCommand::Inspect { json, .. }
            | TeamCatalogCommand::InstallPlan { json, .. } => *json,
        }
    }

    pub fn command_name(&self) -> &'static str {
        match &self.command {
            TeamCatalogCommand::Inspect { .. } => "team catalog inspect",
            TeamCatalogCommand::InstallPlan { .. } => "team catalog install plan",
        }
    }

    pub fn schema_version(&self) -> &'static str {
        match &self.command {
            TeamCatalogCommand::Inspect { .. } => "team.catalog.inspect.v1",
            TeamCatalogCommand::InstallPlan { .. } => "team.catalog.install_plan.v1",
        }
    }
}

pub struct TeamCatalogOutput {
    pub output: String,
}

#[derive(Debug)]
pub struct TeamCatalogError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    schema_version: String,
    catalog_id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    updated_at: String,
    #[serde(default)]
    homepage: Option<String>,
    items: Vec<CatalogItem>,
}

#[derive(Debug, Deserialize)]
struct CatalogItem {
    id: String,
    kind: String,
    name: String,
    description: String,
    version: String,
    source: CatalogSource,
    #[serde(default)]
    publisher: Option<JsonValue>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    requirements: Option<Vec<JsonValue>>,
    #[serde(default)]
    permissions_summary: Option<Vec<String>>,
    #[serde(default)]
    mcp: Option<JsonValue>,
    #[serde(default)]
    trust_note: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CatalogSource {
    #[serde(rename = "type")]
    source_type: String,
    url: String,
    #[serde(default)]
    sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct InspectView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    catalog: CatalogSummaryView,
    items: Vec<InspectItemView>,
    error: Option<ErrorView>,
}

#[derive(Debug, Serialize)]
struct CatalogSummaryView {
    catalog_id: String,
    name: String,
    description: Option<String>,
    updated_at: String,
    homepage: Option<String>,
    items: usize,
}

#[derive(Debug, Serialize)]
struct InspectItemView {
    id: String,
    kind: String,
    name: String,
    description: String,
    version: String,
    installable: bool,
    installed: bool,
    source_type: String,
    sha256: Option<String>,
    publisher: Option<JsonValue>,
    homepage: Option<String>,
    repository: Option<String>,
    requirements: Option<Vec<JsonValue>>,
    permissions_summary: Option<Vec<String>>,
    mcp: Option<JsonValue>,
    trust_note: Option<String>,
    tags: Option<Vec<String>>,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct InstallPlanView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    catalog_id: String,
    item: PlanItemView,
    registry: PlanRegistryView,
    actions: Vec<PlanActionView>,
    warnings: Vec<WarningView>,
    error: Option<ErrorView>,
}

#[derive(Debug, Serialize)]
struct PlanItemView {
    id: String,
    kind: String,
    version: String,
    source_type: String,
    sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct PlanRegistryView {
    installed: bool,
    source_type: Option<String>,
    enabled: Option<bool>,
    path: Option<String>,
}

#[derive(Debug, Serialize)]
struct PlanActionView {
    #[serde(rename = "type")]
    action_type: &'static str,
    replace: bool,
    requires_confirmation: bool,
}

#[derive(Debug, Serialize)]
struct ErrorJsonView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    error: ErrorView,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct ErrorView {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct WarningView {
    code: &'static str,
    message: String,
}

pub fn run(options: &TeamCatalogOptions) -> Result<TeamCatalogOutput, TeamCatalogError> {
    match &options.command {
        TeamCatalogCommand::Inspect { catalog, json } => inspect(catalog, *json),
        TeamCatalogCommand::InstallPlan {
            catalog,
            item_id,
            json,
        } => install_plan(catalog, item_id, *json),
    }
}

pub fn error_json(
    command: &'static str,
    schema_version: &'static str,
    error: &TeamCatalogError,
) -> Result<String, String> {
    let view = ErrorJsonView {
        command,
        schema_version,
        ok: false,
        error: ErrorView {
            code: error.code,
            message: error.message.clone(),
        },
        warnings: Vec::new(),
    };
    serde_json::to_string_pretty(&view).map_err(|error| error.to_string())
}

fn inspect(path: &Path, json: bool) -> Result<TeamCatalogOutput, TeamCatalogError> {
    let catalog = load_catalog(path)?;
    let items = catalog
        .items
        .iter()
        .map(inspect_item_view)
        .collect::<Result<Vec<_>, _>>()?;
    let view = InspectView {
        command: "team catalog inspect",
        schema_version: "team.catalog.inspect.v1",
        ok: true,
        catalog: catalog_summary(&catalog),
        items,
        error: None,
    };

    if json {
        return json_output(&view);
    }

    Ok(TeamCatalogOutput {
        output: format!(
            "SkillRun Team Catalog\ncatalog: {}\nitems: {}",
            view.catalog.catalog_id, view.catalog.items
        ),
    })
}

fn install_plan(
    path: &Path,
    item_id: &str,
    json: bool,
) -> Result<TeamCatalogOutput, TeamCatalogError> {
    let catalog = load_catalog(path)?;
    let item = catalog
        .items
        .iter()
        .find(|item| item.id == item_id)
        .ok_or_else(|| {
            TeamCatalogError::new(
                "catalog.item_not_found",
                format!("catalog item not found: {item_id}"),
            )
        })?;

    if item.kind != "skillrun.skr" {
        return Err(TeamCatalogError::new(
            "catalog.item_not_installable",
            format!(
                "catalog item {} has kind {}; only skillrun.skr is installable in this phase",
                item.id, item.kind
            ),
        ));
    }
    validate_source_for_plan(&item.source)?;

    let registry_entry = registry::registry_entry_status(&item.id)
        .map_err(|error| TeamCatalogError::new("catalog.registry_read_failed", error))?;
    let registry = match registry_entry {
        Some(entry) => PlanRegistryView {
            installed: true,
            source_type: Some(entry.source_type),
            enabled: Some(entry.enabled),
            path: Some(entry.path),
        },
        None => PlanRegistryView {
            installed: false,
            source_type: None,
            enabled: None,
            path: None,
        },
    };

    if registry.installed
        && registry.source_type.as_deref() != Some(registry::IMPORTED_SKR_SOURCE_TYPE)
    {
        return Err(TeamCatalogError::new(
            "catalog.registry_conflict",
            format!(
                "catalog item {} is already registered with source_type {}; Team Catalog can only replace imported_skr entries",
                item.id,
                registry.source_type.as_deref().unwrap_or("<unknown>")
            ),
        ));
    }

    let replace = registry.installed;
    let view = InstallPlanView {
        command: "team catalog install plan",
        schema_version: "team.catalog.install_plan.v1",
        ok: true,
        catalog_id: catalog.catalog_id,
        item: PlanItemView {
            id: item.id.clone(),
            kind: item.kind.clone(),
            version: item.version.clone(),
            source_type: item.source.source_type.clone(),
            sha256: item.source.sha256.clone(),
        },
        registry,
        actions: vec![PlanActionView {
            action_type: "import",
            replace,
            requires_confirmation: replace,
        }],
        warnings: vec![WarningView {
            code: "trust.not_proven",
            message: "sha256 verifies integrity, not publisher identity.".to_string(),
        }],
        error: None,
    };

    if json {
        return json_output(&view);
    }

    Ok(TeamCatalogOutput {
        output: format!(
            "SkillRun Team Catalog Install Plan\ncatalog: {}\nitem: {}\naction: import\nreplace: {}",
            view.catalog_id, view.item.id, replace
        ),
    })
}

fn load_catalog(path: &Path) -> Result<CatalogFile, TeamCatalogError> {
    let text = fs::read_to_string(path).map_err(|error| {
        TeamCatalogError::new(
            "catalog.read_failed",
            format!("failed to read catalog {}: {error}", path.display()),
        )
    })?;
    let catalog: CatalogFile = serde_json::from_str(&text).map_err(|error| {
        TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("catalog is not valid team.catalog.v1 JSON: {error}"),
        )
    })?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(catalog: &CatalogFile) -> Result<(), TeamCatalogError> {
    if catalog.schema_version != "team.catalog.v1" {
        return Err(TeamCatalogError::new(
            "catalog.schema_unsupported",
            format!(
                "unsupported catalog schema_version: {}",
                catalog.schema_version
            ),
        ));
    }
    validate_catalog_id("catalog_id", &catalog.catalog_id)?;
    validate_non_empty("name", &catalog.name)?;
    parse_rfc3339("updated_at", &catalog.updated_at)?;
    for item in &catalog.items {
        validate_catalog_id("item.id", &item.id)?;
        validate_item_kind(&item.kind)?;
        validate_non_empty("item.name", &item.name)?;
        validate_non_empty("item.description", &item.description)?;
        validate_non_empty("item.version", &item.version)?;
        validate_source(&item.source)?;
    }
    Ok(())
}

fn validate_source(source: &CatalogSource) -> Result<(), TeamCatalogError> {
    match source.source_type.as_str() {
        "file" | "https" => {}
        value => {
            return Err(TeamCatalogError::new(
                "catalog.source_unsupported",
                format!("unsupported catalog source type: {value}"),
            ));
        }
    }
    validate_non_empty("source.url", &source.url)?;
    if source.source_type == "https" && source.sha256.is_none() {
        return Err(TeamCatalogError::new(
            "catalog.source_checksum_required",
            "https catalog source requires sha256".to_string(),
        ));
    }
    if let Some(sha256) = &source.sha256 {
        validate_sha256(sha256)?;
    }
    Ok(())
}

fn validate_source_for_plan(source: &CatalogSource) -> Result<(), TeamCatalogError> {
    validate_source(source)?;
    if source.sha256.is_none() {
        return Err(TeamCatalogError::new(
            "catalog.source_checksum_required",
            "install plan requires source.sha256 for installable items".to_string(),
        ));
    }
    Ok(())
}

fn inspect_item_view(item: &CatalogItem) -> Result<InspectItemView, TeamCatalogError> {
    let registry_entry = registry::registry_entry_status(&item.id)
        .map_err(|error| TeamCatalogError::new("catalog.registry_read_failed", error))?;
    Ok(InspectItemView {
        id: item.id.clone(),
        kind: item.kind.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        version: item.version.clone(),
        installable: item.kind == "skillrun.skr",
        installed: registry_entry.is_some(),
        source_type: item.source.source_type.clone(),
        sha256: item.source.sha256.clone(),
        publisher: item.publisher.clone(),
        homepage: item.homepage.clone(),
        repository: item.repository.clone(),
        requirements: item.requirements.clone(),
        permissions_summary: item.permissions_summary.clone(),
        mcp: item.mcp.clone(),
        trust_note: item.trust_note.clone(),
        tags: item.tags.clone(),
        warnings: item_warnings(item),
    })
}

fn item_warnings(item: &CatalogItem) -> Vec<WarningView> {
    let mut warnings = Vec::new();
    if item.kind != "skillrun.skr" {
        warnings.push(WarningView {
            code: "catalog.item.display_only",
            message: format!(
                "{} items are display-only until Core supports them",
                item.kind
            ),
        });
    }
    if item.source.sha256.is_none() {
        warnings.push(WarningView {
            code: "catalog.source.checksum_missing",
            message: "source.sha256 is missing; install plan/apply should fail closed".to_string(),
        });
    }
    warnings
}

fn catalog_summary(catalog: &CatalogFile) -> CatalogSummaryView {
    CatalogSummaryView {
        catalog_id: catalog.catalog_id.clone(),
        name: catalog.name.clone(),
        description: catalog.description.clone(),
        updated_at: catalog.updated_at.clone(),
        homepage: catalog.homepage.clone(),
        items: catalog.items.len(),
    }
}

fn validate_item_kind(kind: &str) -> Result<(), TeamCatalogError> {
    match kind {
        "skillrun.skr" | "agent.skill" | "mcp.server" => Ok(()),
        value => Err(TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("unsupported item kind: {value}"),
        )),
    }
}

fn validate_catalog_id(label: &str, value: &str) -> Result<(), TeamCatalogError> {
    validate_non_empty(label, value)?;
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("{label} cannot be empty"),
        ));
    };
    if !first.is_ascii_alphanumeric() {
        return Err(TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("{label} must start with an ASCII letter or digit: {value}"),
        ));
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-') {
        return Err(TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("{label} may only contain ASCII letters, digits, '.', '_' or '-': {value}"),
        ));
    }
    Ok(())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), TeamCatalogError> {
    if value.is_empty() {
        return Err(TeamCatalogError::new(
            "catalog.schema_invalid",
            format!("{label} cannot be empty"),
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str) -> Result<(), TeamCatalogError> {
    if value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(TeamCatalogError::new(
        "catalog.schema_invalid",
        "source.sha256 must be 64 hex characters".to_string(),
    ))
}

fn parse_rfc3339(label: &str, value: &str) -> Result<(), TeamCatalogError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map(|_| ())
        .map_err(|error| {
            TeamCatalogError::new(
                "catalog.schema_invalid",
                format!("{label} must be RFC3339: {error}"),
            )
        })
}

fn json_output<T: Serialize>(value: &T) -> Result<TeamCatalogOutput, TeamCatalogError> {
    Ok(TeamCatalogOutput {
        output: serde_json::to_string_pretty(value).map_err(|error| {
            TeamCatalogError::new(
                "catalog.schema_invalid",
                format!("failed to serialize Team Catalog output: {error}"),
            )
        })?,
    })
}

impl TeamCatalogError {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }
}
