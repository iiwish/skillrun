# Refund Decision Skill

## Purpose

This SkillRun capsule turns a refund request into an auditable decision. It is
safe for an agent to call when the agent needs a policy decision, not when it
needs to move money or trigger an external refund.

## SOP

1. Confirm the request has an `order_id`, positive `amount`, supported
   `reason`, and known `customer_tier`.
2. Reject invalid input through schema validation before business logic runs.
3. Apply the approval boundary before returning any approval.
4. For refunds above 500, require `manager_approval_id`.
5. Produce a structured decision and a markdown receipt artifact for audit.
6. Never treat stdout as the business result.

## Approval Boundary

- Amounts from 1 to 500 can be approved by policy when the reason is supported.
- Amounts above 500 require `manager_approval_id`.
- The capsule only returns a decision. It does not move money, call payment
  systems, email customers, or mutate external state.

## Required Context

- `order_id`: non-empty order identifier.
- `amount`: positive integer amount in the team's minor currency unit.
- `reason`: one of `damaged`, `duplicate`, or `wrong_item`.
- `customer_tier`: `standard`, `gold`, or `enterprise`.
- `manager_approval_id`: required when `amount` is above 500.

## Recovery Guidance

If the action returns `PolicyViolation`, ask the user for the missing approval
or context before retrying. If it returns `ValidationError`, ask for a supported
reason, positive amount, and non-empty order id.

## Prohibited Behavior

- Do not move money from this capsule.
- Do not approve unsupported refund reasons.
- Do not bypass the approval boundary for high-value refunds.
- Do not infer success from stdout or logs.
