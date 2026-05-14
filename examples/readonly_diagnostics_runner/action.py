import subprocess
import sys
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


COMMANDS = {
    "pwd": [sys.executable, "-c", "import os; print(os.getcwd())"],
    "list": [sys.executable, "-c", "import os; print('\\n'.join(sorted(os.listdir('.'))))"],
    "git_status": ["git", "status", "--short"],
}


class Input(BaseModel):
    diagnostic: Literal["pwd", "list", "git_status"]
    max_output_chars: int = Field(default=4000, ge=200, le=12000)


class Output(BaseModel):
    diagnostic: str
    exit_code: int
    stdout: str
    stderr: str
    timed_out: bool
    audit_note: str


def preflight(input: Input, ctx) -> None:
    if input.diagnostic not in COMMANDS:
        raise ValueError("diagnostic must be one of the declared read-only commands")


def run(input: Input, ctx):
    command = COMMANDS[input.diagnostic]
    timed_out = False
    try:
        completed = subprocess.run(
            command,
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
        )
        exit_code = completed.returncode
        stdout = completed.stdout
        stderr = completed.stderr
    except subprocess.TimeoutExpired as error:
        timed_out = True
        exit_code = 124
        stdout = error.stdout or ""
        stderr = error.stderr or "diagnostic timed out"

    output = Output(
        diagnostic=input.diagnostic,
        exit_code=exit_code,
        stdout=truncate(stdout, input.max_output_chars),
        stderr=truncate(stderr, input.max_output_chars),
        timed_out=timed_out,
        audit_note="Executed one allowlisted read-only diagnostic without shell expansion.",
    )

    artifact_dir = Path(__import__("os").environ["SKILLRUN_ARTIFACT_DIR"])
    artifact_name = "readonly-diagnostic.md"
    (artifact_dir / artifact_name).write_text(
        "\n".join(
            [
                "# Read-only Diagnostic",
                "",
                f"- diagnostic: {output.diagnostic}",
                f"- exit_code: {output.exit_code}",
                f"- timed_out: {output.timed_out}",
                "",
                "## Stdout",
                "",
                output.stdout,
                "",
                "## Stderr",
                "",
                output.stderr,
            ]
        ),
        encoding="utf-8",
    )

    return {
        "output": output,
        "artifacts": [
            {
                "name": "readonly_diagnostic",
                "kind": "markdown",
                "path": artifact_name,
            }
        ],
        "display": {"markdown": f"Diagnostic `{input.diagnostic}` completed with exit code {exit_code}."},
    }


def truncate(value: str | bytes, limit: int) -> str:
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    if len(value) <= limit:
        return value
    return value[:limit] + "\n[truncated]"
