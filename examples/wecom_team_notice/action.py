import json
import os
import shutil
import subprocess
from http import client
from pathlib import Path
from typing import Literal
from urllib import parse

from pydantic import BaseModel, Field


class Input(BaseModel):
    title: str = Field(min_length=1, max_length=80)
    summary: str = Field(min_length=1, max_length=1800)
    audience: Literal["team", "project", "incident", "all_hands"]
    urgency: Literal["normal", "high", "critical"] = "normal"
    dry_run: bool = True
    approval_id: str | None = None
    mentioned_mobile_list: list[str] = Field(default_factory=list, max_length=10)


class Output(BaseModel):
    decision: Literal["preview", "sent", "blocked"]
    message_preview: str
    wecom_response: dict | None = None
    audit_note: str


SECRET_MARKERS = [
    "sk-",
    "AKIA",
    "BEGIN PRIVATE KEY",
    "password=",
    "token=",
    "qyapi.weixin.qq.com/cgi-bin/webhook/send",
]


def preflight(input: Input, ctx) -> None:
    if input.urgency in {"high", "critical"} and not input.approval_id:
        raise ValueError(
            "approval_id is required for high or critical WeCom notices before an agent can continue"
        )
    if input.audience == "all_hands" and not input.approval_id:
        raise ValueError(
            "approval_id is required for all_hands WeCom notices before an agent can continue"
        )
    content = f"{input.title}\n{input.summary}"
    for marker in SECRET_MARKERS:
        if marker.lower() in content.lower():
            raise ValueError(
                f"notice content appears to contain a prohibited secret marker: {marker}"
            )


def run(input: Input, ctx):
    message = build_message(input)
    artifact_name = write_notice_artifact(input, message, None)

    if input.dry_run:
        output = Output(
            decision="preview",
            message_preview=message,
            wecom_response=None,
            audit_note="Dry-run only; no WeCom webhook call was made.",
        )
        return success(output, artifact_name, "WeCom notice preview generated.")

    webhook_url = os.environ.get("WECOM_WEBHOOK_URL")
    if not webhook_url:
        write_dependency_error(
            "WECOM_WEBHOOK_URL is required when dry_run=false. Configure the environment variable or retry with dry_run=true.",
            "Set WECOM_WEBHOOK_URL in the MCP client/server environment, or ask the user to confirm a dry-run preview first.",
        )

    response_payload = send_wecom_message(webhook_url, input, message)
    artifact_name = write_notice_artifact(input, message, response_payload)
    output = Output(
        decision="sent",
        message_preview=message,
        wecom_response=response_payload,
        audit_note="WeCom webhook accepted the message request.",
    )
    return success(output, artifact_name, "WeCom notice sent.")


def build_message(input: Input) -> str:
    lines = [
        f"## {input.title}",
        "",
        input.summary,
        "",
        f"- audience: {input.audience}",
        f"- urgency: {input.urgency}",
        f"- approval_id: {input.approval_id or 'not required'}",
    ]
    if input.mentioned_mobile_list:
        lines.append(f"- mentions: {', '.join(input.mentioned_mobile_list)}")
    return "\n".join(lines)


def send_wecom_message(webhook_url: str, input: Input, message: str) -> dict:
    payload = {
        "msgtype": "markdown",
        "markdown": {
            "content": message,
        },
    }
    if input.mentioned_mobile_list:
        payload["markdown"]["mentioned_mobile_list"] = input.mentioned_mobile_list

    data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    try:
        status, text = post_with_python_http(webhook_url, data)
    except OSError as primary_error:
        status, text = post_with_curl_or_fail(data, primary_error, webhook_url)

    if status >= 400:
        write_dependency_error(
            f"WeCom webhook returned HTTP {status}: {text[:240]}",
            "Check the WeCom robot webhook configuration and retry after fixing it.",
        )

    try:
        body = json.loads(text)
    except json.JSONDecodeError:
        body = {"raw_response": text}
    if isinstance(body, dict) and body.get("errcode") not in (None, 0):
        write_dependency_error(
            f"WeCom webhook returned errcode {body.get('errcode')}: {body.get('errmsg', 'unknown error')}",
            "Check the WeCom robot webhook configuration and retry after fixing it.",
        )
    return body


def post_with_python_http(webhook_url: str, data: bytes) -> tuple[int, str]:
    parsed = parse.urlparse(webhook_url)
    if parsed.scheme != "https" or not parsed.netloc:
        write_dependency_error(
            "WECOM_WEBHOOK_URL must be an https WeCom group robot webhook URL.",
            "Ask the user to provide a valid https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=... URL.",
        )

    target = parsed.path or "/"
    if parsed.query:
        target = f"{target}?{parsed.query}"

    conn = client.HTTPSConnection(parsed.netloc, timeout=10)
    try:
        conn.request(
            "POST",
            target,
            body=data,
            headers={"Content-Type": "application/json"},
        )
        response = conn.getresponse()
        text = response.read().decode("utf-8", errors="replace")
        return response.status, text
    finally:
        conn.close()


def post_with_curl_or_fail(
    data: bytes, primary_error: OSError, webhook_url: str
) -> tuple[int, str]:
    curl = find_curl()
    if not curl:
        write_dependency_error(
            f"WeCom webhook request failed in Python HTTP transport: {primary_error}",
            "Check Python/Windows network access, install curl, or retry with dry_run=true.",
        )

    marker = "__SKILLRUN_HTTP_STATUS__:"
    command = [
        curl,
        "-sS",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "--data-binary",
        "@-",
        "-w",
        f"\n{marker}%{{http_code}}",
        webhook_url,
    ]
    try:
        completed = subprocess.run(
            command,
            input=data,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=15,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as curl_error:
        write_dependency_error(
            f"WeCom webhook request failed in Python HTTP transport: {primary_error}; curl fallback failed: {curl_error}",
            "Check WECOM_WEBHOOK_URL, Windows network access, and outbound access to qyapi.weixin.qq.com.",
        )

    stdout = completed.stdout.decode("utf-8", errors="replace")
    stderr = completed.stderr.decode("utf-8", errors="replace").strip()
    if completed.returncode != 0:
        write_dependency_error(
            f"WeCom webhook request failed in Python HTTP transport: {primary_error}; curl fallback exited with code {completed.returncode}: {stderr[:240]}",
            "Check WECOM_WEBHOOK_URL, Windows network access, and outbound access to qyapi.weixin.qq.com.",
        )

    if f"\n{marker}" not in stdout:
        write_dependency_error(
            f"WeCom webhook curl fallback returned an unexpected response after Python HTTP failed: {primary_error}",
            "Retry with dry_run=true, then check local curl and network configuration.",
        )
    body, status_text = stdout.rsplit(f"\n{marker}", 1)
    try:
        status = int(status_text.strip())
    except ValueError:
        write_dependency_error(
            f"WeCom webhook curl fallback returned an invalid HTTP status after Python HTTP failed: {primary_error}",
            "Retry with dry_run=true, then check local curl and network configuration.",
        )
    return status, body


def find_curl() -> str | None:
    discovered = shutil.which("curl.exe") or shutil.which("curl")
    if discovered:
        return discovered

    candidates = [
        Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32" / "curl.exe",
        Path(os.environ.get("WINDIR", r"C:\Windows")) / "System32" / "curl.exe",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)
    return None


def write_notice_artifact(input: Input, message: str, response_payload: dict | None) -> str:
    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])
    artifact_name = "notice.md"
    artifact = artifact_dir / artifact_name
    response_text = (
        json.dumps(response_payload, ensure_ascii=False, indent=2)
        if response_payload is not None
        else "not sent"
    )
    artifact.write_text(
        "\n".join(
            [
                f"# WeCom Notice: {input.title}",
                "",
                f"- audience: {input.audience}",
                f"- urgency: {input.urgency}",
                f"- dry_run: {input.dry_run}",
                f"- approval_id: {input.approval_id or 'not required'}",
                "",
                "## Message Preview",
                "",
                message,
                "",
                "## WeCom Response",
                "",
                response_text,
            ]
        ),
        encoding="utf-8",
    )
    return artifact_name


def success(output: Output, artifact_name: str, display: str):
    return {
        "output": output,
        "artifacts": [
            {
                "name": "wecom_notice",
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
