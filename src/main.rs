use std::process::ExitCode;

mod cli;
mod config;
mod consumer;
mod errors;
mod hashing;
mod init;
mod inspect;
mod manifest;
mod mcp;
mod permissions;
mod run_record;
mod runtime;
mod schemas;
mod adapters {
    pub mod python;
}

fn main() -> ExitCode {
    cli::run(std::env::args())
}
