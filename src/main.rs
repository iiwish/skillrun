use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PLANNED_COMMANDS: &[&str] = &[
    "init", "manifest", "inspect", "test", "run", "serve", "pack",
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None | Some("-h") | Some("--help") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("-V") | Some("--version") => {
            println!("skillrun {VERSION}");
            ExitCode::SUCCESS
        }
        Some(command) if PLANNED_COMMANDS.contains(&command) => {
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

fn print_help() {
    println!(
        "\
SkillRun

Rust CLI for turning one SOP and one action into a tested MCP skill package.

Usage:
  skillrun [--help]
  skillrun [--version]
  skillrun <command> [options]

Planned MVP commands:
  init       create a Python action capsule skeleton
  manifest   generate the Manifest from SOP, action metadata, config and examples
  inspect    show capsule contract, permissions and instruction-only status
  test       run the default example through the runtime contract
  run        run a capsule with an explicit input file
  serve      expose Manifest-driven MCP tools
  pack       create a .skr package

T001 only provides this Rust CLI skeleton. Runtime behavior lands in later tasks."
    );
}
