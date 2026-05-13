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
mod mcp;
mod pack;
mod permissions;
mod readiness;
mod run_record;
mod runtime;
mod schemas;

fn main() -> ExitCode {
    cli::run(std::env::args())
}
