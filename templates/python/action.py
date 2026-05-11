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
        raise ValueError("manager approval is required for refunds above 500")


def run(input: Input, ctx) -> Output:
    if input.amount > 500:
        return Output(
            decision="needs_approval",
            amount=input.amount,
            reasoning_summary="Refund requires manager approval before execution.",
            audit_note=f"Order {input.order_id} requires approval before refund.",
        )

    return Output(
        decision="approved",
        amount=input.amount,
        reasoning_summary=f"Refund reason {input.reason} is allowed by the starter policy.",
        audit_note=f"Order {input.order_id} approved for refund decision only.",
    )
