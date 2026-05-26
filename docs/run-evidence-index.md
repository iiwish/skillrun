# Run Evidence and Local Index

SkillRun records run evidence inside each capsule under that capsule's local `.skillrun/runs/` directory. The consumer-facing CLI does not require Desktop, scripts, or MCP clients to read those internal directories directly.

## Stable Surfaces

Use these commands as the stable run evidence surfaces:

```bash
skillrun consumer runs list --json
skillrun consumer runs list --json --capsule <id>
skillrun consumer runs list --json --status <status>
skillrun consumer runs list --json --mode <mode>
skillrun consumer runs list --json --ok true
skillrun consumer runs list --json --error-code <code>
skillrun consumer runs list --json --since <rfc3339> --until <rfc3339>
skillrun consumer runs list --json --source scan
skillrun consumer runs list --json --source index
skillrun consumer runs inspect <run-id> --json
skillrun consumer runs inspect <run-id> --json --capsule <id>
skillrun consumer runs index rebuild --json
skillrun consumer runs index status --json
```

`consumer runs list` remains registry-scoped and summary-only. By default it uses `--source scan`, which scans currently registered capsules and returns summary metadata. It does not return full input, envelope body, stdout, or stderr content.

`consumer runs list --source index` is an explicit opt-in cache read path. It reads `$SKILLRUN_HOME/runs-index.json`, applies the same list filters to indexed summaries, and returns `source.kind = "index"` in JSON output. It fails instead of silently falling back when the index is missing, unreadable, unsupported, invalid, or stale.

`consumer runs inspect` is the detail surface for a single run. It reports availability of private evidence files by default. It does not include input or log bodies unless a future explicit contract adds that behavior.

## Local Index

`consumer runs index rebuild` writes:

```text
$SKILLRUN_HOME/runs-index.json
```

The file uses `consumer.runs.index.v1` and stores summary metadata plus stable `run_ref` values:

- `run_id`
- `run_ref`
- `capsule_id`
- `capsule_path`
- mode / status / ok / error code
- started / finished / duration
- Manifest and source hashes
- artifact count
- `input_included: false`

The index is a rebuildable metadata cache. It is not the only copy of run evidence, not a package artifact, not a retention policy, and not a trust or sandbox boundary.

## Status and Staleness

`consumer runs index status --json` reports whether the index exists, is readable, uses a supported schema, and may be stale relative to the registry or capsule run directories.

If `stale` is `true`, consumers should rebuild before using the index as a query cache:

```bash
skillrun consumer runs index rebuild --json
```

The status command reads index metadata and filesystem modification times. It does not read input, envelope body, stdout, or stderr content.

`consumer runs list --source index` uses the same readiness checks. If the registry or any registered capsule run evidence appears newer than the index `generated_at`, the command fails with a stale-index error and asks the caller to rebuild. This keeps Desktop and automation from accidentally treating old cached data as current data.

## Privacy Boundary

The list and index surfaces are intentionally summary-only. They must not include:

- full input content
- output or error envelope body
- stdout content
- stderr content
- artifact content

Consumers that need detail should use `consumer runs inspect` for one run at a time and rely on its explicit inclusion fields.

## Current Non-Goals

The run evidence index does not introduce:

- a daemon
- background indexing
- remote upload or sync
- marketplace behavior
- automatic dependency installation
- OS sandboxing
- global history across unregistered capsules
- default `consumer runs list` reads from the index
