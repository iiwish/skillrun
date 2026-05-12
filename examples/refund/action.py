import os
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


class Input(BaseModel):
    order_id: str = Field(min_length=1)
    amount: int = Field(ge=1)
    reason: Literal["damaged", "duplicate", "wrong_item"]
    customer_tier: Literal["standard", "gold", "enterprise"] = "standard"
    manager_approval_id: str | None = None


class Output(BaseModel):
    decision: Literal["approved", "needs_approval", "rejected"]
    amount: int
    reasoning_summary: str
    audit_note: str


def preflight(input: Input, ctx) -> None:
    if input.amount > 500 and not input.manager_approval_id:
        raise ValueError(
            "manager approval is required for refunds above 500 before an agent can continue"
        )


def run(input: Input, ctx):
    approval_note = (
        f"manager approval {input.manager_approval_id} recorded"
        if input.manager_approval_id
        else "within automatic approval limit"
    )
    output = Output(
        decision="approved",
        amount=input.amount,
        reasoning_summary=(
            f"Refund reason {input.reason} is supported and {approval_note}."
        ),
        audit_note=(
            f"Order {input.order_id}: approved refund decision only; no external money movement performed."
        ),
    )

    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])
    receipt_name = f"{input.order_id}-refund-decision.md"
    receipt = artifact_dir / receipt_name
    receipt.write_text(
        "\n".join(
            [
                f"# Refund Decision: {input.order_id}",
                "",
                f"- decision: {output.decision}",
                f"- amount: {output.amount}",
                f"- reason: {input.reason}",
                f"- customer_tier: {input.customer_tier}",
                f"- approval: {approval_note}",
                "",
                output.audit_note,
            ]
        ),
        encoding="utf-8",
    )

    return {
        "output": output,
        "artifacts": [
            {
                "name": "refund_decision_receipt",
                "kind": "markdown",
                "path": receipt_name,
            }
        ],
        "display": {
            "markdown": f"Refund decision for `{input.order_id}`: **{output.decision}**."
        },
    }
