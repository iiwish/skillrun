use std::path::PathBuf;

use crate::readiness;

#[derive(Debug)]
pub struct CheckOptions {
    pub cwd: PathBuf,
}

pub struct CheckReport {
    pub output: String,
    pub ok: bool,
}

pub fn run(options: &CheckOptions) -> Result<CheckReport, String> {
    let report = readiness::evaluate(&options.cwd)?;
    Ok(CheckReport {
        output: readiness::render_check(&report),
        ok: report.ok,
    })
}
