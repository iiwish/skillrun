# SkillRun v0.5.0 Feature Spec: Language-agnostic Adapter Protocol

Version: v0.5.0
Status: Ready_For_User_Review
Created: 2026-05-14
Updated: 2026-05-14
Source: User asked to start v0.5.0 from documentation and clarify the core problem.
Review: User requested review and continuation on 2026-05-14; assistant rereview found no blocking issue and prepared plan/work graph for user review.

## 一句话判断

v0.5.0 的核心问题是：SkillRun 已经证明 Python/JS adapter path 可行，但还没有把“语言接入”定义成稳定协议。v0.5.0 应该交付 Language-agnostic Adapter Protocol，并实际实现 Level 0 command adapter 作为协议原生证明，而不是急着增加更多 blessed language adapters。

## Product Positioning

SkillRun Core must remain Rust, Manifest-driven and language-agnostic. Language ecosystems integrate through adapters and SDK wrappers. The v0.5.0 product value is a stable southbound protocol that lets future language support scale without contaminating Core semantics.

## Target Users

- SkillRun maintainers who need a stable boundary before adding more adapter behavior.
- Community contributors who want to build Ruby, PHP, Go or other adapters.
- AI engineers who need to understand whether a capsule can be consumed without trusting dynamic source imports.
- Platform engineers who care about conformance, diagnostics and runtime requirements.

## User Stories

### US-050-001: Adapter author understands the minimum contract

As a community adapter author, I can read a protocol document and know what metadata phase, run phase, envelopes and artifacts my adapter must implement.

### US-050-002: Maintainer can test adapters consistently

As a maintainer, I can run conformance fixtures against Python stable, JS alpha and future adapters without writing one-off tests for each language.

### US-050-003: Consumer can trust Manifest boundaries

As a capsule consumer, I can rely on Consumer Mode checks staying static and Manifest-bound regardless of the action language.

### US-050-004: Author can choose a low-level adapter path

As an advanced author, I can use a minimal command/static-schema path when no blessed SDK exists, without forcing Core to understand my language ecosystem.

## Functional Requirements

- FR-050-001: Define the Adapter Protocol concepts and lifecycle.
- FR-050-002: Define metadata phase inputs, outputs, timeout and non-secret constraints.
- FR-050-003: Define run phase IPC, envelope, artifact and stdout/stderr discipline.
- FR-050-004: Define adapter capability levels: command adapter, community adapter and blessed adapter.
- FR-050-005: Define runtime requirement reporting as diagnostics, not installation.
- FR-050-006: Define conformance fixture categories.
- FR-050-007: Map existing Python stable and JS alpha behavior to the protocol.
- FR-050-008: Preserve Consumer Mode no-import behavior.
- FR-050-009: Preserve current user-visible CLI behavior unless explicitly approved later.
- FR-050-010: Implement Level 0 command adapter with explicit argv command and static schema.
- FR-050-011: Ensure Level 0 command adapter uses standard SkillRun IPC and output/error/artifact envelopes.
- FR-050-012: Ensure Level 0 command adapter does not dynamically import or inspect action source for metadata.

## Non-functional Requirements

- NFR-050-001: Core remains Rust and Manifest-driven.
- NFR-050-002: No new language ecosystem should define SkillRun product semantics.
- NFR-050-003: Documentation must clearly distinguish Adapter Protocol, Language Adapter and Language SDK.
- NFR-050-004: v0.5.0 must not imply dependency installation, vendoring or sandboxing.
- NFR-050-005: v0.5.0 must remain compatible with existing Python and JS capsules unless a later approved plan says otherwise.

## Functional Scope

### In Scope For The Design Phase

- Public adapter protocol documentation.
- Feature-scoped spec and later plan/task artifacts.
- Protocol vocabulary and capability levels.
- Level 0 command adapter as approved implementation direction.
- Open questions for implementation planning.

### Potential Implementation Scope After Approval

- Internal adapter trait/protocol cleanup.
- Conformance tests for existing Python and JS adapters.
- Level 0 command adapter.
- At least one command-adapter example that does not become a blessed language adapter.

## Non-goals

- Full TypeScript runtime.
- npm/pip/Bundler/Composer/Go module install flow.
- Dependency vendoring.
- Registry or marketplace.
- OS sandbox.
- New MCP transport.
- `.skr` runtime image.
- New blessed language adapter before protocol conformance exists.

## Edge Cases

- Adapter metadata succeeds but runtime dependency is missing.
- Adapter runtime writes malformed envelope.
- Adapter writes artifact outside the artifact directory.
- Adapter prints valid-looking JSON to stdout but does not write output file.
- Static schema exists without metadata extraction.
- `runtime.adapter` and future `runtime.command` are both present.
- command argv points to a missing executable.
- command process exits successfully but does not write output envelope.
- command process writes artifact path outside artifact dir.
- command adapter schema is malformed or missing.

## Constraints And Assumptions

- Existing Python stable path and JS alpha path must remain useful references.
- Consumer Mode must remain static and no-import for metadata.
- `.skr` remains source + Manifest archive.
- `skillrun.config.json` remains the likely override point for command/static-schema configuration.

## Data Or Integration Needs

- Current Manifest runtime fields.
- Current adapter metadata and run behavior.
- Existing readiness checks from v0.4.
- Existing output/error/artifact envelope contracts.

## Success Criteria

- A maintainer can explain the Core/Adapter/SDK boundary in one diagram.
- A community adapter author can identify the minimum Level 0 or Level 1 requirements.
- Existing Python and JS behavior can be mapped to the protocol without inventing new product semantics.
- Level 0 command adapter proves a non-blessed language/process can run through standard SkillRun IPC and envelopes.

## Acceptance Criteria

- v0.5.0 planning maps every FR/NFR to a task.
- Open questions are answered in plan or explicitly deferred.
- Implementation may only begin after plan, checklist, analysis, work graph and execution packets are ready.

## Clarifications

- 2026-05-14: User asked whether v0.5.0 can begin and requested starting from documentation.
- 2026-05-14: User clarified that v0.5.0 should actually implement Level 0 command adapter.

## Open Questions

1. Should adapter protocol version be stored as `runtime.protocol_version` in Manifest?
2. Should conformance be a CLI command, Rust integration test suite, or external fixture package?
3. Should `skillrun.config.json` support full JSON Schema only, or also a lightweight schema sugar?
4. What is the minimum public API for community adapter authors?
5. Which non-blessed command-adapter example best proves Level 0 without confusing users into thinking that language is officially supported?
