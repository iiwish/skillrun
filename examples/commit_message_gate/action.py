import json
import os
import re
import subprocess
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


COMMIT_RE = re.compile(
    r"^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\([a-z0-9._-]+\))?: [^\s].+"
)


class Input(BaseModel):
    message: str = Field(min_length=1, max_length=200)
    perform_commit: bool = False


class Output(BaseModel):
    decision: Literal["accepted", "committed"]
    normalized_message: str
    audit_note: str
    git_stdout: str | None = None


def preflight(input: Input, ctx) -> None:
    message = input.message.strip()
    if message != input.message:
        raise ValueError("commit message must not have leading or trailing whitespace")
    if "\n" in message or "\r" in message:
        raise ValueError("commit message must be a single subject line")
    if len(message) > 70:
        raise ValueError("commit message subject must be 70 characters or fewer")
    if not COMMIT_RE.match(message):
        raise ValueError(
            "commit message must follow Conventional Commits, for example `fix: handle stale manifest`"
        )


def run(input: Input, ctx):
    if not input.perform_commit:
        output = Output(
            decision="accepted",
            normalized_message=input.message,
            audit_note="Message accepted; no Git command was executed.",
        )
        return success(output, "Commit message accepted without executing Git.")

    completed = subprocess.run(
        ["git", "commit", "-m", input.message],
        text=True,
        capture_output=True,
        check=False,
        timeout=20,
    )
    if completed.returncode != 0:
        write_dependency_error(
            f"git commit failed with exit code {completed.returncode}: {completed.stderr.strip()[:240]}",
            "Check that the current directory is a Git repository with staged changes, then retry.",
        )

    output = Output(
        decision="committed",
        normalized_message=input.message,
        audit_note="Git commit completed for already staged changes only.",
        git_stdout=completed.stdout,
    )
    return success(output, "Git commit completed.")


def success(output: Output, display: str):
    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])
    artifact_name = "commit-message-gate.md"
    (artifact_dir / artifact_name).write_text(
        "\n".join(
            [
                "# Commit Message Gate",
                "",
                f"- decision: {output.decision}",
                f"- normalized_message: {output.normalized_message}",
                f"- audit_note: {output.audit_note}",
            ]
        ),
        encoding="utf-8",
    )
    return {
        "output": output,
        "artifacts": [
            {
                "name": "commit_message_gate",
                "kind": "markdown",
                "path": artifact_name,
            }
        ],
        "display": {"markdown": display},
    }


def write_dependency_error(message: str, llm_hint: str) -> None:
    output_path = Path(os.environ["SKILLRUN_OUTPUT_JSON"])
    output_path.write_text(
        json.dumps(
            {
                "ok": False,
                "error": {
                    "code": "DependencyError",
                    "message": message,
                    "recoverable": True,
                    "llm_hint": llm_hint,
                },
                "display": {"markdown": message},
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    raise SystemExit(1)
