export const inputSchema = {
  type: "object",
  required: ["order_id", "amount", "reason"],
  additionalProperties: false,
  properties: {
    order_id: { type: "string", minLength: 1 },
    amount: { type: "integer", minimum: 1 },
    reason: { type: "string", enum: ["damaged", "duplicate", "wrong_item"] },
    customer_tier: {
      type: "string",
      enum: ["standard", "gold", "enterprise"],
      default: "standard"
    },
    manager_approval_id: { type: ["string", "null"] }
  }
};

export const outputSchema = {
  type: "object",
  required: ["decision", "amount", "reasoning_summary", "audit_note"],
  additionalProperties: false,
  properties: {
    decision: { type: "string", enum: ["approved", "needs_approval", "rejected"] },
    amount: { type: "integer", minimum: 1 },
    reasoning_summary: { type: "string" },
    audit_note: { type: "string" }
  }
};

export function preflight(input, ctx) {
  if (input.amount > 500 && !input.manager_approval_id) {
    throw new Error("manager approval is required for refunds above 500");
  }
}

export async function run(input, ctx) {
  if (input.amount > 500) {
    return {
      decision: "needs_approval",
      amount: input.amount,
      reasoning_summary: "Refund requires manager approval before execution.",
      audit_note: `Order ${input.order_id} requires approval before refund.`
    };
  }

  return {
    decision: "approved",
    amount: input.amount,
    reasoning_summary: `Refund reason ${input.reason} is allowed by the starter policy.`,
    audit_note: `Order ${input.order_id} approved for refund decision only.`
  };
}
