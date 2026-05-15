use std::path::PathBuf;

use crate::readiness;

#[derive(Debug)]
pub struct CheckOptions {
    pub cwd: PathBuf,
    pub json: bool,
}

pub struct CheckReport {
    pub output: String,
    pub ok: bool,
}

pub fn run(options: &CheckOptions) -> Result<CheckReport, String> {
    let report = readiness::evaluate(&options.cwd)?;
    let output = if options.json {
        readiness::render_json("check", &report)?
    } else {
        readiness::render_check(&report)
    };
    Ok(CheckReport {
        output,
        ok: report.ok,
    })
}
