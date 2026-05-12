use std::path::PathBuf;
use std::process::ExitCode;

use crate::init::{self, InitOptions};
use crate::inspect::{self, InspectOptions};
use crate::manifest::{self, ManifestOptions};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const UNIMPLEMENTED_COMMANDS: &[&str] = &["test", "run", "serve", "pack"];

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
            Ok(options) => match init::create_python_capsule(&options) {
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
                eprintln!("usage: skillrun init <name> --python [--output <dir>]");
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
        Some(command) if UNIMPLEMENTED_COMMANDS.contains(&command) => {
            eprintln!("error: command not implemented yet: {command}");
            ExitCode::from(2)
        }
        Some(command) => {
            eprintln!("error: unknown command: {command}");
            eprintln!("run `skillrun --help` to see available commands");
            ExitCode::from(2)
        }
    }
}

fn parse_init(args: Vec<String>) -> Result<InitOptions, String> {
    let mut name = None;
    let mut python = false;
    let mut output_dir = PathBuf::from(".");
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--python" => {
                python = true;
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

    if !python {
        return Err("init currently requires --python".to_string());
    }

    let name = name.ok_or_else(|| "init requires a capsule name".to_string())?;
    Ok(InitOptions { name, output_dir })
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
  init       create a Python action capsule skeleton
  manifest   generate the Manifest from SOP, action metadata, config and examples
  inspect    show capsule contract, permissions and instruction-only status
  test       run the default example through the runtime contract
  run        run a capsule with an explicit input file
  serve      expose Manifest-driven MCP tools
  pack       create a .skr package

Implemented:
  init --python
  manifest
  inspect

Later tasks implement runtime, MCP, and packaging behavior."
    );
}
