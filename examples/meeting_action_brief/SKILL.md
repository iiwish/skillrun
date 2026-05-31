# Meeting Action Brief Skill

## Purpose

This SkillRun capsule turns rough meeting notes into a structured action brief.
It is safe for an agent to call when the user needs local, auditable follow-up
planning, not when the agent should send messages, create tickets, or mutate
external systems.

## SOP

1. Accept rough notes, transcript excerpts, or bullet notes from one meeting.
2. Require enough content to identify at least one decision, action, risk, or
   open question.
3. Reject obvious secrets, API keys, passwords, private keys, and webhook URLs.
4. Extract decisions, action items, risks, and open questions into stable fields.
5. Mark missing owners or due dates explicitly instead of inventing them.
6. Produce a markdown brief artifact that the user can copy into a team update.
7. Never send the brief to Slack, Lark, email, a ticket tracker, or a calendar.

## Required Context

- `meeting_title`: short title for the brief.
- `notes`: meeting notes or transcript excerpt.
- `audience`: one of `self`, `team`, `leadership`, or `customer`.
- `follow_up_style`: one of `concise`, `detailed`, or `executive`.

## Recovery Guidance

If the action returns `PolicyViolation`, remove secrets or provide more concrete
meeting notes before retrying. If it returns `ValidationError`, provide a title,
non-empty notes, and supported audience / style values.

## Prohibited Behavior

- Do not send or publish the brief.
- Do not create calendar events, tickets, commits, or chat messages.
- Do not invent owners, deadlines, or decisions that are not present in notes.
- Do not include secrets, tokens, passwords, private keys, or webhook URLs.
