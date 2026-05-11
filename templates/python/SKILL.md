# {{name}} Skill

## Purpose

Describe the business job this capsule performs and the situations where an
agent is allowed to call it.

## SOP

1. Validate that the request contains enough context to make a decision.
2. Apply policy limits before returning an approval.
3. Return a structured rejection when approval is missing or the request is out
   of scope.
4. Keep the final decision auditable.

## Required Context

- `order_id`
- `amount`
- `reason`
- `customer_tier`
- `manager_approval_id` when policy requires escalation

## Prohibited Behavior

- Do not move money or trigger external side effects from this template.
- Do not approve requests that violate the local policy.
- Do not treat stdout as the business result.

## Recovery Guidance

If the action returns a policy error, ask for the missing approval or context
instead of retrying blindly.
