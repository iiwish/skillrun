import os
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


ALLOWED_TOP_LEVEL = {"src", "docs", "tests"}
FORBIDDEN_NAMES = {
    ".env",
    ".env.local",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "secrets.json",
}


class Input(BaseModel):
    file_path: str = Field(min_length=1, max_length=240)
    old_text: str = Field(min_length=1, max_length=4000)
    new_text: str = Field(min_length=1, max_length=4000)


class Output(BaseModel):
    decision: Literal["patched"]
    file_path: str
    replacements: int
    audit_note: str


def preflight(input: Input, ctx) -> None:
    path = Path(input.file_path)
    parts = path.parts
    if path.is_absolute():
        raise ValueError("file_path must be project-relative, not absolute")
    if not parts:
        raise ValueError("file_path must include a file name")
    if any(part in {"", ".", ".."} for part in parts):
        raise ValueError("file_path must not contain empty, current or parent traversal segments")
    if any(part.startswith(".") for part in parts):
        raise ValueError("file_path must not include hidden files or directories")
    if parts[0] not in ALLOWED_TOP_LEVEL:
        raise ValueError("file_path must be under src/, docs/ or tests/")
    if path.name in FORBIDDEN_NAMES or path.suffix in {".pem", ".key"}:
        raise ValueError("file_path points to a protected configuration, lock or secret-bearing file")

    target = Path.cwd() / path
    root = Path.cwd().resolve()
    try:
        resolved = target.resolve(strict=False)
    except OSError as error:
        raise ValueError(f"file_path could not be resolved: {error}") from error
    if root not in resolved.parents and resolved != root:
        raise ValueError("file_path resolves outside the capsule workspace")
    if not target.is_file():
        raise ValueError("file_path must point to an existing file")

    text = target.read_text(encoding="utf-8")
    count = text.count(input.old_text)
    if count == 0:
        raise ValueError("old_text was not found in the target file")
    if count > 1:
        raise ValueError("old_text appears more than once; provide a narrower replacement")


def run(input: Input, ctx):
    target = Path(input.file_path)
    text = target.read_text(encoding="utf-8")
    patched = text.replace(input.old_text, input.new_text, 1)
    target.write_text(patched, encoding="utf-8")

    output = Output(
        decision="patched",
        file_path=input.file_path,
        replacements=1,
        audit_note="Applied exactly one old_text/new_text replacement inside an allowed path.",
    )

    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])
    artifact_name = "bounded-file-patch.md"
    (artifact_dir / artifact_name).write_text(
        "\n".join(
            [
                "# Bounded File Patch",
                "",
                f"- file_path: {input.file_path}",
                "- replacements: 1",
                "",
                "## Old Text",
                "",
                input.old_text,
                "",
                "## New Text",
                "",
                input.new_text,
            ]
        ),
        encoding="utf-8",
    )

    return {
        "output": output,
        "artifacts": [
            {
                "name": "bounded_file_patch",
                "kind": "markdown",
                "path": artifact_name,
            }
        ],
        "display": {"markdown": f"Patched `{input.file_path}` with one bounded replacement."},
    }
