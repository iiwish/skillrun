use std::process::ExitCode;

mod cli;
mod init;

fn main() -> ExitCode {
    cli::run(std::env::args())
}
