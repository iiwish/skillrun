use std::process::ExitCode;

mod adapters;
mod capsule_import;
mod check;
mod cli;
mod config;
mod consumer;
mod doctor;
mod errors;
mod hashing;
mod host;
mod init;
mod inspect;
mod manifest;
mod manifest_access;
mod mcp;
mod mount_plan;
mod pack;
mod permissions;
mod readiness;
mod registry;
mod router;
mod run_record;
mod runtime;
mod schemas;
mod switchboard;

fn main() -> ExitCode {
    cli::run(std::env::args())
}
