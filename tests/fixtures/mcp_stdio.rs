use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

pub struct ScriptedMcpClient {
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Receiver<String>,
}

impl ScriptedMcpClient {
    pub fn spawn(capsule: &Path) -> Self {
        Self::spawn_with_env(capsule, &[])
    }

    #[allow(dead_code)]
    pub fn spawn_with_path(capsule: &Path, path: &Path) -> Self {
        Self::spawn_with_env(capsule, &[("PATH", path.to_string_lossy().as_ref())])
    }

    fn spawn_with_env(capsule: &Path, envs: &[(&str, &str)]) -> Self {
        let cwd = capsule.to_string_lossy().to_string();
        let mut command = Command::new(env!("CARGO_BIN_EXE_skillrun"));
        command
            .args(["serve", "--mcp", "--cwd", &cwd])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in envs {
            command.env(key, value);
        }
        let mut child = command.spawn().expect("MCP server process should spawn");

        let stdin = child.stdin.take().expect("MCP stdin should be piped");
        let stdout = child.stdout.take().expect("MCP stdout should be piped");
        let (stdout_tx, stdout_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                let read = reader.read_line(&mut line).unwrap_or(0);
                if read == 0 {
                    break;
                }
                let _ = stdout_tx.send(line.trim_end_matches(['\r', '\n']).to_string());
            }
        });

        Self {
            child,
            stdin,
            stdout_lines: stdout_rx,
        }
    }

    pub fn send(&mut self, message: Value) {
        let line = serde_json::to_string(&message).expect("MCP request should serialize");
        self.stdin
            .write_all(line.as_bytes())
            .expect("MCP request should write to stdin");
        self.stdin
            .write_all(b"\n")
            .expect("MCP request newline should write");
        self.stdin.flush().expect("MCP stdin should flush");
    }

    pub fn read_response(&mut self, label: &str) -> Value {
        let line = self
            .stdout_lines
            .recv_timeout(Duration::from_secs(5))
            .unwrap_or_else(|error| panic!("timed out waiting for {label}: {error}"));
        let response: Value = serde_json::from_str(&line)
            .unwrap_or_else(|error| panic!("{label} should be JSON-RPC, got {line:?}: {error}"));
        assert_eq!(response["jsonrpc"], "2.0", "{label} should be JSON-RPC");
        response
    }

    #[allow(dead_code)]
    pub fn expect_no_stdout_line(&mut self, label: &str) {
        match self.stdout_lines.recv_timeout(Duration::from_millis(200)) {
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                panic!("MCP stdout closed while checking {label}");
            }
            Ok(line) => {
                panic!("{label} should not produce stdout, got {line:?}");
            }
        }
    }

    pub fn initialize(&mut self) -> Value {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "skillrun-test-client",
                    "version": "0.0.0"
                }
            }
        }));
        self.read_response("initialize response")
    }

    pub fn initialized(&mut self) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }));
    }
}

impl Drop for ScriptedMcpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
