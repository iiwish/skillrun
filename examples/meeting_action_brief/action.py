import os
import re
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


class Input(BaseModel):
    meeting_title: str = Field(min_length=1, max_length=120)
    notes: str = Field(min_length=40, max_length=6000)
    audience: Literal["self", "team", "leadership", "customer"] = "team"
    follow_up_style: Literal["concise", "detailed", "executive"] = "concise"


class ActionItem(BaseModel):
    task: str
    owner: str | None = None
    due: str | None = None
    confidence: Literal["high", "medium", "low"]


class Output(BaseModel):
    brief_title: str
    summary: str
    decisions: list[str]
    action_items: list[ActionItem]
    risks: list[str]
    open_questions: list[str]
    follow_up_message: str
    artifact_path: str


SECRET_PATTERNS = [
    re.compile(r"sk-[a-zA-Z0-9]{20,}", re.IGNORECASE),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"password\s*[:=]\s*\S+", re.IGNORECASE),
    re.compile(r"token\s*[:=]\s*\S+", re.IGNORECASE),
    re.compile(r"secret\s*[:=]\s*\S+", re.IGNORECASE),
    re.compile(r"BEGIN (RSA |EC |OPENSSH |PRIVATE )?PRIVATE KEY", re.IGNORECASE),
    re.compile(r"https://(hooks\.slack\.com|qyapi\.weixin\.qq\.com|open\.feishu\.cn)/\S+", re.IGNORECASE),
]

ACTION_MARKERS = (
    "todo",
    "action",
    "owner",
    "due",
    "follow up",
    "follow-up",
    "next",
    "决定",
    "决策",
    "行动",
    "负责人",
    "截止",
    "下周",
    "风险",
    "问题",
)


def preflight(input: Input, ctx) -> None:
    for pattern in SECRET_PATTERNS:
        if pattern.search(input.notes):
            raise ValueError("meeting notes appear to contain a secret, token, password, private key, or webhook URL")

    normalized = input.notes.lower()
    if not any(marker in normalized for marker in ACTION_MARKERS):
        raise ValueError(
            "meeting notes need at least one decision, action, owner, due date, risk, or open question cue"
        )


def run(input: Input, ctx):
    lines = meaningful_lines(input.notes)
    decisions = extract_marked(lines, ("decision", "decided", "agreed", "决定", "决策", "结论"))
    risks = extract_marked(lines, ("risk", "blocked", "concern", "风险", "阻塞", "担心"))
    open_questions = extract_marked(lines, ("question", "open", "unclear", "问题", "待确认", "不确定"))
    action_items = extract_actions(lines)

    if not decisions:
        decisions = ["No explicit decision found; review the notes before sharing."]
    if not risks:
        risks = ["No explicit risk called out."]
    if not open_questions:
        open_questions = ["No explicit open question called out."]
    if not action_items:
        action_items = [
            ActionItem(
                task="Review the meeting notes and assign concrete follow-up owners.",
                owner=None,
                due=None,
                confidence="low",
            )
        ]

    summary = build_summary(input, decisions, action_items, risks, open_questions)
    follow_up_message = build_follow_up(input, summary, action_items, open_questions)
    artifact_name = "meeting-action-brief.md"
    write_artifact(input, artifact_name, summary, decisions, action_items, risks, open_questions, follow_up_message)

    output = Output(
        brief_title=input.meeting_title,
        summary=summary,
        decisions=decisions,
        action_items=action_items,
        risks=risks,
        open_questions=open_questions,
        follow_up_message=follow_up_message,
        artifact_path=artifact_name,
    )
    return {
        "output": output.model_dump(),
        "artifacts": [
            {
                "name": "meeting_action_brief",
                "kind": "markdown",
                "path": artifact_name,
            }
        ],
        "display": {"markdown": f"Meeting action brief generated for `{input.meeting_title}`."},
    }


def meaningful_lines(notes: str) -> list[str]:
    result = []
    for raw in notes.splitlines():
        line = raw.strip().strip("-* ")
        if line:
            result.append(line)
    if not result:
        result.append(notes.strip())
    return result


def extract_marked(lines: list[str], markers: tuple[str, ...]) -> list[str]:
    matches = []
    for line in lines:
        lowered = line.lower()
        if any(marker in lowered for marker in markers):
            matches.append(clean_marker(line))
    return unique(matches)[:6]


def extract_actions(lines: list[str]) -> list[ActionItem]:
    actions = []
    for line in lines:
        lowered = line.lower()
        if any(marker in lowered for marker in ("todo", "action", "follow up", "follow-up", "行动", "待办", "跟进")):
            cleaned = clean_marker(line)
            actions.append(
                ActionItem(
                    task=cleaned,
                    owner=extract_owner(line),
                    due=extract_due(line),
                    confidence="high" if extract_owner(line) or extract_due(line) else "medium",
                )
            )
    return actions[:8]


def clean_marker(line: str) -> str:
    return re.sub(
        r"^(decision|decided|agreed|risk|question|open question|open|todo|action|next|follow[- ]?up|决定|决策|结论|风险|问题|待办|行动|跟进)\s*[:：-]\s*",
        "",
        line.strip(),
        flags=re.IGNORECASE,
    )


def extract_owner(line: str) -> str | None:
    patterns = [
        r"(?:owner|负责人|assigned to|由)\s*[:：]\s*([A-Za-z0-9_\-\u4e00-\u9fff ]{1,30}?)(?=\s*(?:[,;|]|\sdue\s*[:：]|\sdeadline\s*[:：]|\s截止\s*[:：]|$))",
        r"@([A-Za-z0-9_\-\u4e00-\u9fff]{1,30})",
    ]
    for pattern in patterns:
        match = re.search(pattern, line, flags=re.IGNORECASE)
        if match:
            return match.group(1).strip()
    return None


def extract_due(line: str) -> str | None:
    patterns = [
        r"(?:due|deadline|截止|到期)\s*[:：]\s*([A-Za-z0-9_\-/\u4e00-\u9fff ]{1,30}?)(?=\s*(?:[,;|]|$))",
        r"\b(20\d{2}-\d{2}-\d{2})\b",
        r"(tomorrow|next week|this week|下周|明天|本周)",
    ]
    for pattern in patterns:
        match = re.search(pattern, line, flags=re.IGNORECASE)
        if match:
            return match.group(1).strip()
    return None


def build_summary(
    input: Input,
    decisions: list[str],
    action_items: list[ActionItem],
    risks: list[str],
    open_questions: list[str],
) -> str:
    return (
        f"{input.meeting_title}: {len(decisions)} decision(s), "
        f"{len(action_items)} action item(s), {len(risks)} risk note(s), "
        f"and {len(open_questions)} open question(s) prepared for {input.audience}."
    )


def build_follow_up(input: Input, summary: str, action_items: list[ActionItem], open_questions: list[str]) -> str:
    if input.follow_up_style == "executive":
        lead = f"Executive brief: {summary}"
    elif input.follow_up_style == "detailed":
        lead = f"Detailed follow-up for {input.meeting_title}: {summary}"
    else:
        lead = f"Follow-up: {summary}"

    action_lines = [
        f"- {item.task} ({item.owner or 'owner TBD'} / {item.due or 'due TBD'})"
        for item in action_items[:5]
    ]
    question_lines = [f"- {question}" for question in open_questions[:3]]
    return "\n".join(
        [
            lead,
            "",
            "Actions:",
            *action_lines,
            "",
            "Open questions:",
            *question_lines,
        ]
    )


def write_artifact(
    input: Input,
    artifact_name: str,
    summary: str,
    decisions: list[str],
    action_items: list[ActionItem],
    risks: list[str],
    open_questions: list[str],
    follow_up_message: str,
) -> None:
    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])
    artifact_dir.mkdir(parents=True, exist_ok=True)
    lines = [
        f"# Meeting Action Brief: {input.meeting_title}",
        "",
        f"- audience: {input.audience}",
        f"- follow_up_style: {input.follow_up_style}",
        "",
        "## Summary",
        "",
        summary,
        "",
        "## Decisions",
        "",
        *[f"- {item}" for item in decisions],
        "",
        "## Action Items",
        "",
        *[
            f"- {item.task} | owner: {item.owner or 'TBD'} | due: {item.due or 'TBD'} | confidence: {item.confidence}"
            for item in action_items
        ],
        "",
        "## Risks",
        "",
        *[f"- {item}" for item in risks],
        "",
        "## Open Questions",
        "",
        *[f"- {item}" for item in open_questions],
        "",
        "## Follow-up Message",
        "",
        follow_up_message,
    ]
    (artifact_dir / artifact_name).write_text("\n".join(lines), encoding="utf-8")


def unique(values: list[str]) -> list[str]:
    seen = set()
    result = []
    for value in values:
        normalized = value.strip()
        key = normalized.lower()
        if normalized and key not in seen:
            seen.add(key)
            result.append(normalized)
    return result
