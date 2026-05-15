use std::path::PathBuf;

use crate::readiness;

#[derive(Debug)]
pub struct DoctorOptions {
    pub cwd: PathBuf,
    pub json: bool,
}

pub struct DoctorReport {
    pub output: String,
    pub ok: bool,
}

pub fn check(options: &DoctorOptions) -> Result<DoctorReport, String> {
    let report = readiness::evaluate(&options.cwd)?;
    let output = if options.json {
        readiness::render_json("doctor", &report)?
    } else {
        readiness::render_doctor(&report)
    };
    Ok(DoctorReport {
        output,
        ok: report.ok,
    })
}
