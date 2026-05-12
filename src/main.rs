use std::process::ExitCode;

mod cli;
mod config;
mod hashing;
mod init;
mod inspect;
mod manifest;
mod run_record;
mod runtime;
mod schemas;
mod adapters {
    pub mod python;
}

fn main() -> ExitCode {
    cli::run(std::env::args())
}
