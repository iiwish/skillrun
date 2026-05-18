use std::path::PathBuf;
use std::process::ExitCode;

use crate::capsule_import::{self, ImportOptions};
use crate::check::{self, CheckOptions};
use crate::consumer;
use crate::doctor::{self, DoctorOptions};
use crate::host::{self, HostStatusOptions};
use crate::init::{self, InitLanguage, InitOptions};
use crate::inspect::{self, InspectOptions};
use crate::manifest::{self, ManifestOptions};
use crate::mcp;
use crate::mount_plan::{self, MountApplyOptions, MountPlanOptions, MountRollbackOptions};
use crate::pack::{self, PackOptions};
use crate::registry::{self, RegistryCommand, RegistryOptions};
use crate::router::{self, RouterOptions};
use crate::runtime::{self, RunOptions, TestOptions};
use crate::switchboard::{self, SwitchboardCommand, SwitchboardOptions};

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub fn run<I>(args: I) -> ExitCode
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter().skip(1);

    match args.next().as_deref() {
        None | Some("-h") | Some("--help") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("-V") | Some("--version") => {
            println!("skillrun {VERSION}");
            ExitCode::SUCCESS
        }
        Some("host") => match parse_host(args.collect()) {
            Ok(options) => match host::status(&options) {
                Ok(output) => {
                    println!("{}", output.output);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun host status [--json]");
                ExitCode::from(2)
            }
        },
        Some("init") => match parse_init(args.collect()) {
            Ok(options) => match init::create_capsule(&options) {
                Ok(path) => {
                    println!("created {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun init <name> (--python|--py|--js) [--output <dir>]");
                ExitCode::from(2)
            }
        },
        Some("manifest") => match parse_manifest(args.collect()) {
            Ok(options) => match manifest::generate(&options) {
                Ok(path) => {
                    println!("generated {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun manifest [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some("inspect") => match parse_inspect(args.collect()) {
            Ok(options) => match inspect::render(&options) {
                Ok(summary) => {
                    println!("{summary}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun inspect [--json] [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some("check") => match parse_check(args.collect()) {
            Ok(options) => match check::run(&options) {
                Ok(report) => {
                    println!("{}", report.output);
                    if report.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(2)
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun check [--json] [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some("doctor") => match parse_doctor(args.collect()) {
            Ok(options) => match doctor::check(&options) {
                Ok(report) => {
                    println!("{}", report.output);
                    if report.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(2)
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun doctor [--json] [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some("import") => match parse_import(args.collect()) {
            Ok(options) => match capsule_import::run(&options) {
                Ok(output) => {
                    println!("{}", output.output);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    if options.json {
                        match capsule_import::error_json(&error) {
                            Ok(output) => println!("{output}"),
                            Err(render_error) => {
                                eprintln!("error: {error}");
                                eprintln!(
                                    "error: failed to render import JSON error: {render_error}"
                                );
                            }
                        }
                    } else {
                        eprintln!("error: {error}");
                    }
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun import <package.skr> [--id <id>] [--to <dir>] [--json]");
                ExitCode::from(2)
            }
        },
        Some("consumer") => match parse_consumer(args.collect()) {
            Ok(command) => match command {
                ConsumerCommand::Inventory { json } => match registry::consumer_inventory(json) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::Exposure { json } => match registry::consumer_exposure(json) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::RunsList {
                    json,
                    capsule,
                    limit,
                } => match registry::consumer_runs_list(json, capsule.as_deref(), limit) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::RunsInspect {
                    run_id,
                    json,
                    capsule,
                } => match registry::consumer_runs_inspect(&run_id, json, capsule.as_deref()) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::MountPlan(options) => match mount_plan::plan(&options) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::MountApply(options) => match mount_plan::apply(&options) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                ConsumerCommand::MountRollback(options) => match mount_plan::rollback(&options) {
                    Ok(output) => {
                        println!("{}", output.output);
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun consumer <inventory|exposure|runs|mount> [options]");
                ExitCode::from(2)
            }
        },
        Some("registry") => match parse_registry(args.collect()) {
            Ok(options) => match registry::run(&options) {
                Ok(output) => {
                    println!("{}", output.output);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun registry <add|list|inspect|remove> [options]");
                ExitCode::from(2)
            }
        },
        Some("router") => match parse_router(args.collect()) {
            Ok(options) => match router::serve_mcp(&options) {
                Ok(router::RouterOutcome::DryRun(output)) => {
                    println!("{output}");
                    ExitCode::SUCCESS
                }
                Ok(router::RouterOutcome::Served) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun router serve --mcp [--dry-run]");
                ExitCode::from(2)
            }
        },
        Some("switchboard") => match parse_switchboard(args.collect()) {
            Ok(options) => match switchboard::run(&options) {
                Ok(output) => {
                    println!("{}", output.output);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun switchboard <list|enable|disable> [options]");
                ExitCode::from(2)
            }
        },
        Some("test") => match parse_test(args.collect()) {
            Ok(options) => match runtime::run_test(&options) {
                Ok(outcome) => {
                    println!("{}", outcome.envelope);
                    if outcome.success {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(2)
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun test [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some("run") => match parse_run(args.collect()) {
            Ok(options) => match runtime::run_with_input(&options) {
                Ok(outcome) => {
                    println!("{}", outcome.envelope);
                    if outcome.success {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(2)
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun run [--cwd <dir>] --input <file>");
                ExitCode::from(2)
            }
        },
        Some("serve") => match parse_serve(args.collect()) {
            Ok(options) => match consumer::validate(&options.cwd, "skillrun serve --mcp") {
                Ok(manifest) if options.dry_run => {
                    match mcp::dry_run_contract(&options.cwd, &manifest) {
                        Ok(contract) => {
                            println!("{contract}");
                            ExitCode::SUCCESS
                        }
                        Err(error) => {
                            eprintln!("error: {error}");
                            ExitCode::from(2)
                        }
                    }
                }
                Ok(manifest) => match mcp::serve_stdio(&options.cwd, &manifest) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                },
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun serve --mcp [--cwd <dir>] [--dry-run]");
                ExitCode::from(2)
            }
        },
        Some("pack") => match parse_pack(args.collect()) {
            Ok(options) => match pack::create(&options) {
                Ok(summary) => {
                    println!("{summary}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                eprintln!("usage: skillrun pack [--cwd <dir>]");
                ExitCode::from(2)
            }
        },
        Some(command) => {
            eprintln!("error: unknown command: {command}");
            eprintln!("run `skillrun --help` to see available commands");
            ExitCode::from(2)
        }
    }
}

struct ServeOptions {
    cwd: PathBuf,
    dry_run: bool,
}

enum ConsumerCommand {
    Inventory {
        json: bool,
    },
    Exposure {
        json: bool,
    },
    RunsList {
        json: bool,
        capsule: Option<String>,
        limit: Option<usize>,
    },
    RunsInspect {
        run_id: String,
        json: bool,
        capsule: Option<String>,
    },
    MountPlan(MountPlanOptions),
    MountApply(MountApplyOptions),
    MountRollback(MountRollbackOptions),
}

fn parse_init(args: Vec<String>) -> Result<InitOptions, String> {
    let mut name = None;
    let mut language = None;
    let mut output_dir = PathBuf::from(".");
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--python" | "--py" => {
                set_language(&mut language, InitLanguage::Python)?;
                index += 1;
            }
            "--js" => {
                set_language(&mut language, InitLanguage::Js)?;
                index += 1;
            }
            "--output" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--output requires a directory".to_string());
                };
                output_dir = PathBuf::from(value);
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown init option: {value}"));
            }
            value => {
                if name.is_some() {
                    return Err(format!("unexpected init argument: {value}"));
                }
                name = Some(value.to_string());
                index += 1;
            }
        }
    }

    let name = name.ok_or_else(|| "init requires a capsule name".to_string())?;
    let language = language.ok_or_else(|| "init requires --python, --py, or --js".to_string())?;
    Ok(InitOptions {
        name,
        output_dir,
        language,
    })
}

fn set_language(language: &mut Option<InitLanguage>, value: InitLanguage) -> Result<(), String> {
    match language {
        Some(existing) if *existing != value => {
            Err("choose only one language: --python/--py or --js".to_string())
        }
        Some(_) => Ok(()),
        None => {
            *language = Some(value);
            Ok(())
        }
    }
}

fn parse_host(args: Vec<String>) -> Result<HostStatusOptions, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("host requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    match command {
        "status" => parse_host_status(rest),
        value => Err(format!("unknown host subcommand: {value}")),
    }
}

fn parse_host_status(args: Vec<String>) -> Result<HostStatusOptions, String> {
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value => return Err(format!("unexpected host status argument: {value}")),
        }
    }

    Ok(HostStatusOptions { json })
}

fn parse_manifest(args: Vec<String>) -> Result<ManifestOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected manifest argument: {value}")),
        }
    }

    Ok(ManifestOptions { cwd })
}

fn parse_inspect(args: Vec<String>) -> Result<InspectOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected inspect argument: {value}")),
        }
    }

    Ok(InspectOptions { cwd, json })
}

fn parse_doctor(args: Vec<String>) -> Result<DoctorOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected doctor argument: {value}")),
        }
    }

    Ok(DoctorOptions { cwd, json })
}

fn parse_check(args: Vec<String>) -> Result<CheckOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected check argument: {value}")),
        }
    }

    Ok(CheckOptions { cwd, json })
}

fn parse_import(args: Vec<String>) -> Result<ImportOptions, String> {
    let mut package = None;
    let mut id = None;
    let mut target_dir = None;
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--id" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--id requires a registry id".to_string());
                };
                id = Some(value.to_string());
                index += 2;
            }
            "--to" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--to requires a directory".to_string());
                };
                target_dir = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(format!("unexpected import argument: {value}"));
            }
            value => {
                if package.is_some() {
                    return Err(format!("unexpected import argument: {value}"));
                }
                package = Some(PathBuf::from(value));
                index += 1;
            }
        }
    }

    let package = package.ok_or_else(|| "import requires <package.skr>".to_string())?;
    Ok(ImportOptions {
        package,
        id,
        target_dir,
        json,
    })
}

fn parse_consumer(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("consumer requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    match command {
        "inventory" => parse_consumer_inventory(rest),
        "exposure" => parse_consumer_exposure(rest),
        "runs" => parse_consumer_runs(rest),
        "mount" => parse_consumer_mount(rest),
        value => Err(format!("unknown consumer subcommand: {value}")),
    }
}

fn parse_consumer_inventory(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value => return Err(format!("unexpected consumer inventory argument: {value}")),
        }
    }

    Ok(ConsumerCommand::Inventory { json })
}

fn parse_consumer_exposure(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value => return Err(format!("unexpected consumer exposure argument: {value}")),
        }
    }

    Ok(ConsumerCommand::Exposure { json })
}

fn parse_consumer_runs(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("consumer runs requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    match command {
        "list" => parse_consumer_runs_list(rest),
        "inspect" => parse_consumer_runs_inspect(rest),
        value => Err(format!("unknown consumer runs subcommand: {value}")),
    }
}

fn parse_consumer_runs_list(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut json = false;
    let mut capsule = None;
    let mut limit = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--capsule" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--capsule requires a registry id".to_string());
                };
                capsule = Some(value.to_string());
                index += 2;
            }
            "--limit" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--limit requires a number".to_string());
                };
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| format!("--limit must be a positive integer: {value}"))?;
                if parsed == 0 {
                    return Err("--limit must be greater than 0".to_string());
                }
                limit = Some(parsed);
                index += 2;
            }
            value => return Err(format!("unexpected consumer runs list argument: {value}")),
        }
    }

    Ok(ConsumerCommand::RunsList {
        json,
        capsule,
        limit,
    })
}

fn parse_consumer_runs_inspect(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut run_id = None;
    let mut json = false;
    let mut capsule = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--capsule" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--capsule requires a registry id".to_string());
                };
                capsule = Some(value.to_string());
                index += 2;
            }
            value if value.starts_with("--") => {
                return Err(format!(
                    "unexpected consumer runs inspect argument: {value}"
                ));
            }
            value => {
                if run_id.is_some() {
                    return Err(format!(
                        "unexpected consumer runs inspect argument: {value}"
                    ));
                }
                run_id = Some(value.to_string());
                index += 1;
            }
        }
    }

    let run_id = run_id.ok_or_else(|| "consumer runs inspect requires <run-id>".to_string())?;
    Ok(ConsumerCommand::RunsInspect {
        run_id,
        json,
        capsule,
    })
}

fn parse_consumer_mount(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("consumer mount requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    match command {
        "plan" => parse_consumer_mount_plan(rest),
        "apply" => parse_consumer_mount_apply(rest),
        "rollback" => parse_consumer_mount_rollback(rest),
        value => Err(format!("unknown consumer mount subcommand: {value}")),
    }
}

fn parse_consumer_mount_plan(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut client = None;
    let mut config = None;
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--client" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--client requires a value".to_string());
                };
                client = Some(value.to_string());
                index += 2;
            }
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--config requires a path".to_string());
                };
                config = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            value => return Err(format!("unexpected consumer mount plan argument: {value}")),
        }
    }

    let client = client.ok_or_else(|| "consumer mount plan requires --client <id>".to_string())?;
    Ok(ConsumerCommand::MountPlan(MountPlanOptions {
        client,
        config,
        json,
    }))
}

fn parse_consumer_mount_apply(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut client = None;
    let mut config = None;
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--client" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--client requires a value".to_string());
                };
                client = Some(value.to_string());
                index += 2;
            }
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--config requires a path".to_string());
                };
                config = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            value => return Err(format!("unexpected consumer mount apply argument: {value}")),
        }
    }

    let client = client.ok_or_else(|| "consumer mount apply requires --client <id>".to_string())?;
    Ok(ConsumerCommand::MountApply(MountApplyOptions {
        client,
        config,
        json,
    }))
}

fn parse_consumer_mount_rollback(args: Vec<String>) -> Result<ConsumerCommand, String> {
    let mut client = None;
    let mut config = None;
    let mut backup = None;
    let mut json = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--client" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--client requires a value".to_string());
                };
                client = Some(value.to_string());
                index += 2;
            }
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--config requires a path".to_string());
                };
                config = Some(PathBuf::from(value));
                index += 2;
            }
            "--backup" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--backup requires a path".to_string());
                };
                backup = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            value => {
                return Err(format!(
                    "unexpected consumer mount rollback argument: {value}"
                ))
            }
        }
    }

    let client =
        client.ok_or_else(|| "consumer mount rollback requires --client <id>".to_string())?;
    let backup =
        backup.ok_or_else(|| "consumer mount rollback requires --backup <path>".to_string())?;
    Ok(ConsumerCommand::MountRollback(MountRollbackOptions {
        client,
        config,
        backup,
        json,
    }))
}

fn parse_registry(args: Vec<String>) -> Result<RegistryOptions, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("registry requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    let command = match command {
        "add" => parse_registry_add(rest)?,
        "list" => parse_registry_list(rest)?,
        "inspect" => parse_registry_inspect(rest)?,
        "remove" => parse_registry_remove(rest)?,
        value => return Err(format!("unknown registry subcommand: {value}")),
    };
    Ok(RegistryOptions { command })
}

fn parse_router(args: Vec<String>) -> Result<RouterOptions, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("router requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    match command {
        "serve" => parse_router_serve(rest),
        value => Err(format!("unknown router subcommand: {value}")),
    }
}

fn parse_router_serve(args: Vec<String>) -> Result<RouterOptions, String> {
    let mut mcp = false;
    let mut dry_run = false;

    for value in args {
        match value.as_str() {
            "--mcp" => mcp = true,
            "--dry-run" => dry_run = true,
            value => return Err(format!("unexpected router serve argument: {value}")),
        }
    }

    if !mcp {
        return Err("router serve currently requires --mcp".to_string());
    }

    Ok(RouterOptions { dry_run })
}

fn parse_registry_add(args: Vec<String>) -> Result<RegistryCommand, String> {
    let mut cwd = None;
    let mut id = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = Some(PathBuf::from(value));
                index += 2;
            }
            "--id" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--id requires a value".to_string());
                };
                id = Some(value.to_string());
                index += 2;
            }
            value => return Err(format!("unexpected registry add argument: {value}")),
        }
    }

    let cwd = cwd.ok_or_else(|| "registry add requires --cwd <capsule>".to_string())?;
    Ok(RegistryCommand::Add { cwd, id })
}

fn parse_registry_list(args: Vec<String>) -> Result<RegistryCommand, String> {
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value => return Err(format!("unexpected registry list argument: {value}")),
        }
    }

    Ok(RegistryCommand::List { json })
}

fn parse_registry_inspect(args: Vec<String>) -> Result<RegistryCommand, String> {
    let mut id = None;
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unexpected registry inspect argument: {value}"));
            }
            value => {
                if id.is_some() {
                    return Err(format!("unexpected registry inspect argument: {value}"));
                }
                id = Some(value.to_string());
            }
        }
    }

    let id = id.ok_or_else(|| "registry inspect requires an id".to_string())?;
    Ok(RegistryCommand::Inspect { id, json })
}

fn parse_registry_remove(args: Vec<String>) -> Result<RegistryCommand, String> {
    if args.len() != 1 {
        return Err("registry remove requires exactly one id".to_string());
    }
    Ok(RegistryCommand::Remove {
        id: args[0].clone(),
    })
}

fn parse_switchboard(args: Vec<String>) -> Result<SwitchboardOptions, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("switchboard requires a subcommand".to_string());
    };
    let rest = args[1..].to_vec();
    let command = match command {
        "list" => parse_switchboard_list(rest)?,
        "enable" => parse_switchboard_toggle(rest, true)?,
        "disable" => parse_switchboard_toggle(rest, false)?,
        value => return Err(format!("unknown switchboard subcommand: {value}")),
    };
    Ok(SwitchboardOptions { command })
}

fn parse_switchboard_list(args: Vec<String>) -> Result<SwitchboardCommand, String> {
    let mut json = false;

    for value in args {
        match value.as_str() {
            "--json" => json = true,
            value => return Err(format!("unexpected switchboard list argument: {value}")),
        }
    }

    Ok(SwitchboardCommand::List { json })
}

fn parse_switchboard_toggle(args: Vec<String>, enable: bool) -> Result<SwitchboardCommand, String> {
    if args.len() != 1 {
        let command = if enable { "enable" } else { "disable" };
        return Err(format!("switchboard {command} requires exactly one id"));
    }

    let id = args[0].clone();
    if enable {
        Ok(SwitchboardCommand::Enable { id })
    } else {
        Ok(SwitchboardCommand::Disable { id })
    }
}

fn parse_test(args: Vec<String>) -> Result<TestOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected test argument: {value}")),
        }
    }

    Ok(TestOptions { cwd })
}

fn parse_run(args: Vec<String>) -> Result<RunOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut input = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            "--input" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--input requires a file".to_string());
                };
                input = Some(PathBuf::from(value));
                index += 2;
            }
            value => return Err(format!("unexpected run argument: {value}")),
        }
    }

    let input = input.ok_or_else(|| "run requires --input <file>".to_string())?;
    Ok(RunOptions { cwd, input })
}

fn parse_serve(args: Vec<String>) -> Result<ServeOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut mcp = false;
    let mut dry_run = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--mcp" => {
                mcp = true;
                index += 1;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            value => return Err(format!("unexpected serve argument: {value}")),
        }
    }

    if !mcp {
        return Err("serve currently requires --mcp".to_string());
    }

    Ok(ServeOptions { cwd, dry_run })
}

fn parse_pack(args: Vec<String>) -> Result<PackOptions, String> {
    let mut cwd = PathBuf::from(".");
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a directory".to_string());
                };
                cwd = PathBuf::from(value);
                index += 2;
            }
            value => return Err(format!("unexpected pack argument: {value}")),
        }
    }

    Ok(PackOptions { cwd })
}

fn print_help() {
    println!(
        "\
SkillRun

Rust CLI for turning one SOP and one action into a tested MCP skill package.

Usage:
  skillrun [--help]
  skillrun [--version]
  skillrun <command> [options]

MVP commands:
  init       create a Python stable or JS alpha action capsule skeleton
  manifest   generate the Manifest from SOP, action metadata, config and examples
  host       show Desktop-facing host readiness and Core contract status
  inspect    show capsule contract, permissions and instruction-only status
  check      check capsule readiness from Manifest without running action source
  doctor     diagnose capsule files, Manifest freshness and adapter recovery steps
  import     import a .skr package into the local capsule registry
  consumer   expose headless consumer control-plane JSON
  registry   manage local capsule inventory
  switchboard enable or disable registered capsules
  test       run the default example through the runtime contract
  run        run a capsule with an explicit input file
  serve      expose Manifest-driven MCP tools
  pack       create a .skr package

Implemented:
  init --python
  init --py
  init --js (alpha)
  host status [--json]
  manifest
  inspect [--json]
  check [--json]
  doctor [--json]
  import <package.skr> [--id <id>] [--to <dir>] [--json]
  consumer inventory [--json]
  consumer exposure [--json]
  consumer runs list [--json] [--capsule <id>] [--limit <n>]
  consumer runs inspect <run-id> [--json] [--capsule <id>]
  consumer mount plan --client <id> [--config <path>] [--json]
  consumer mount apply --client claude-desktop [--config <path>] [--json]
  consumer mount rollback --client claude-desktop --backup <path> [--config <path>] [--json]
  router serve --mcp [--dry-run]
  registry add/list/inspect/remove
  switchboard list/enable/disable
  test
  run
  serve --mcp
  serve --mcp --dry-run
  pack"
    );
}
