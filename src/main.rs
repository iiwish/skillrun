use std::process::ExitCode;

mod adapters;
mod check;
mod cli;
mod config;
mod consumer;
mod doctor;
mod errors;
mod hashing;
mod init;
mod inspect;
mod manifest;
mod manifest_access;
mod mcp;
mod pack;
mod permissions;
mod readiness;
mod registry;
mod run_record;
mod runtime;
mod schemas;
mod switchboard;

fn main() -> ExitCode {
    cli::run(std::env::args())
}
