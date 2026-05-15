use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub struct TimeoutMessages<'a> {
    pub spawn: &'a str,
    pub poll: &'a str,
    pub collect: &'a str,
    pub timeout: &'a str,
}

pub fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
    messages: TimeoutMessages<'_>,
) -> Result<Output, String> {
    prepare_process_tree(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("{}: {}", messages.spawn, spawn_error_text(&error)))?;
    let started_at = Instant::now();

    loop {
        if child
            .try_wait()
            .map_err(|error| format!("{}: {error}", messages.poll))?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|error| format!("{}: {error}", messages.collect));
        }

        if started_at.elapsed() >= timeout {
            terminate_process_tree(&mut child);
            let _ = child.wait_with_output();
            return Err(format!(
                "{} after {} ms",
                messages.timeout,
                timeout.as_millis()
            ));
        }

        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn prepare_process_tree(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(not(unix))]
fn prepare_process_tree(_command: &mut Command) {}

#[cfg(windows)]
fn terminate_process_tree(child: &mut Child) {
    let pid = child.id().to_string();
    let status = Command::new("taskkill")
        .args(["/PID", pid.as_str(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !matches!(status, Ok(status) if status.success()) {
        let _ = child.kill();
    }
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    let process_group = format!("-{}", child.id());
    let _ = Command::new("kill")
        .args(["-TERM", process_group.as_str()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(50));
    let _ = child.kill();
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

fn spawn_error_text(error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        "program not found".to_string()
    } else {
        error.to_string()
    }
}
