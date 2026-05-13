use std::path::PathBuf;
use std::process::ExitCode;

use crate::check::{self, CheckOptions};
use crate::consumer;
use crate::doctor::{self, DoctorOptions};
use crate::init::{self, InitLanguage, InitOptions};
use crate::inspect::{self, InspectOptions};
use crate::manifest::{self, ManifestOptions};
use crate::mcp;
use crate::pack::{self, PackOptions};
use crate::runtime::{self, RunOptions, TestOptions};

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
                eprintln!("usage: skillrun inspect [--cwd <dir>]");
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
                eprintln!("usage: skillrun check [--cwd <dir>]");
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
                eprintln!("usage: skillrun doctor [--cwd <dir>]");
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
            value => return Err(format!("unexpected inspect argument: {value}")),
        }
    }

    Ok(InspectOptions { cwd })
}

fn parse_doctor(args: Vec<String>) -> Result<DoctorOptions, String> {
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
            value => return Err(format!("unexpected doctor argument: {value}")),
        }
    }

    Ok(DoctorOptions { cwd })
}

fn parse_check(args: Vec<String>) -> Result<CheckOptions, String> {
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
            value => return Err(format!("unexpected check argument: {value}")),
        }
    }

    Ok(CheckOptions { cwd })
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
  inspect    show capsule contract, permissions and instruction-only status
  check      check capsule readiness from Manifest without running action source
  doctor     diagnose capsule files, Manifest freshness and adapter recovery steps
  test       run the default example through the runtime contract
  run        run a capsule with an explicit input file
  serve      expose Manifest-driven MCP tools
  pack       create a .skr package

Implemented:
  init --python
  init --py
  init --js (alpha)
  manifest
  inspect
  check
  doctor
  test
  run
  serve --mcp
  serve --mcp --dry-run
  pack"
    );
}
