use std::path::PathBuf;

use crate::readiness;

#[derive(Debug)]
pub struct DoctorOptions {
    pub cwd: PathBuf,
}

pub struct DoctorReport {
    pub output: String,
    pub ok: bool,
}

pub fn check(options: &DoctorOptions) -> Result<DoctorReport, String> {
    let report = readiness::evaluate(&options.cwd)?;
    Ok(DoctorReport {
        output: readiness::render_doctor(&report),
        ok: report.ok,
    })
}
