#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FINAL_OUT_DIR="${SKILLRUN_DESKTOP_HERO_OUT:-"$ROOT/target/desktop-hero-skr"}"
if [[ -z "$FINAL_OUT_DIR" || "$FINAL_OUT_DIR" == "/" ]]; then
  echo "refusing unsafe SKILLRUN_DESKTOP_HERO_OUT: ${FINAL_OUT_DIR:-<empty>}" >&2
  exit 1
fi
mkdir -p "$(dirname "$FINAL_OUT_DIR")"
OUT_DIR="$(mktemp -d "$(dirname "$FINAL_OUT_DIR")/desktop-hero-skr.XXXXXX")"
WORK_DIR="$OUT_DIR/source"
CATALOG_PATH="$OUT_DIR/catalog.json"
FINAL_CATALOG_PATH="$FINAL_OUT_DIR/catalog.json"

cleanup() {
  rm -rf "$OUT_DIR"
}

trap cleanup EXIT

run_skillrun() {
  if [[ -n "${SKILLRUN_CLI:-}" ]]; then
    "$SKILLRUN_CLI" "$@"
  else
    cargo run --manifest-path "$ROOT/Cargo.toml" --quiet -- "$@"
  fi
}

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

copy_tracked_example() {
  local id="$1"
  local dest="$WORK_DIR/$id"

  mkdir -p "$dest"
  git -C "$ROOT" ls-files -z "examples/$id" | while IFS= read -r -d '' file; do
    local relative="${file#examples/$id/}"
    mkdir -p "$dest/$(dirname "$relative")"
    cp "$ROOT/$file" "$dest/$relative"
  done
}

build_package() {
  local id="$1"
  local source_dir="$WORK_DIR/$id"

  copy_tracked_example "$id"
  run_skillrun manifest --cwd "$source_dir" >/dev/null || return 1
  run_skillrun pack --cwd "$source_dir" >/dev/null || return 1

  local archive
  archive="$(find "$source_dir/dist" -maxdepth 1 -type f -name "$id-*.skr" -print | sort | tail -n 1)"
  if [[ -z "$archive" ]]; then
    echo "failed to find generated .skr package for $id" >&2
    exit 1
  fi

  cp "$archive" "$OUT_DIR/$(basename "$archive")"
  basename "$archive"
}

mkdir -p "$WORK_DIR"

COMMAND_ARCHIVE="$(build_package "command_hello")"

COMMAND_VERSION="${COMMAND_ARCHIVE#command_hello-}"
COMMAND_VERSION="${COMMAND_VERSION%.skr}"
COMMAND_SHA="$(sha256_file "$OUT_DIR/$COMMAND_ARCHIVE")"
UPDATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

MEETING_ARCHIVE=""
MEETING_WARNING=""
if MEETING_ARCHIVE="$(build_package "meeting_action_brief" 2>"$OUT_DIR/meeting_action_brief.warn.log")"; then
  MEETING_VERSION="${MEETING_ARCHIVE#meeting_action_brief-}"
  MEETING_VERSION="${MEETING_VERSION%.skr}"
  MEETING_SHA="$(sha256_file "$OUT_DIR/$MEETING_ARCHIVE")"
else
  MEETING_ARCHIVE=""
  MEETING_WARNING="$(cat "$OUT_DIR/meeting_action_brief.warn.log")"
fi

cat >"$CATALOG_PATH" <<EOF
{
  "schema_version": "team.catalog.v1",
  "catalog_id": "skillrun.local.hero-skr",
  "name": "SkillRun Local Hero SKR Manual Test Kit",
  "description": "Local .skr packages for Desktop Team Library, switchboard, Router, and run evidence manual testing.",
  "updated_at": "$UPDATED_AT",
  "items": [
    {
      "id": "command_hello",
      "kind": "skillrun.skr",
      "name": "Command Hello",
      "description": "Minimal command adapter package for low-friction Team Library smoke tests.",
      "version": "$COMMAND_VERSION",
      "publisher": {
        "name": "SkillRun Local Manual Test Kit"
      },
      "source": {
        "type": "file",
        "url": "./$COMMAND_ARCHIVE",
        "sha256": "$COMMAND_SHA"
      },
      "requirements": [
        {
          "kind": "python",
          "summary": "python3 command executable; no pydantic package required"
        }
      ],
      "permissions_summary": [
        "reads typed name input",
        "writes run-local greeting artifact"
      ],
      "mcp": {
        "exposes_tools": true,
        "mount": "skillrun-router"
      },
      "trust_note": "Local manual-test package generated from tracked SkillRun repository examples. Review before enabling.",
      "tags": [
        "default-smoke",
        "reference",
        "command-adapter"
      ]
    }
EOF

if [[ -n "$MEETING_ARCHIVE" ]]; then
  cat >>"$CATALOG_PATH" <<EOF
    ,
    {
      "id": "meeting_action_brief",
      "kind": "skillrun.skr",
      "name": "Meeting Action Brief",
      "description": "Turn meeting notes into decisions, action items, risks, open questions, follow-up copy, and an audit artifact.",
      "version": "$MEETING_VERSION",
      "publisher": {
        "name": "SkillRun Local Manual Test Kit"
      },
      "source": {
        "type": "file",
        "url": "./$MEETING_ARCHIVE",
        "sha256": "$MEETING_SHA"
      },
      "requirements": [
        {
          "kind": "python",
          "summary": "Python 3.13+ with pydantic 2.x for ready/run tests"
        }
      ],
      "permissions_summary": [
        "reads provided meeting notes",
        "writes run-local markdown action brief"
      ],
      "mcp": {
        "exposes_tools": true,
        "mount": "skillrun-router"
      },
      "trust_note": "Local manual-test package generated from tracked SkillRun repository examples. Review before enabling.",
      "tags": [
        "product-hero",
        "meeting",
        "follow-up",
        "knowledge-work"
      ]
    }
EOF
fi

cat >>"$CATALOG_PATH" <<EOF
  ]
}
EOF

cat >"$OUT_DIR/README.md" <<EOF
# SkillRun Desktop Hero SKR Manual Test Kit

Generated from tracked SkillRun examples for local Desktop manual testing.

Default smoke item: command_hello.
Product hero item: meeting_action_brief when packaging prerequisites are available.

Use catalog:

$FINAL_CATALOG_PATH

Use individual packages:

- command_hello: $FINAL_OUT_DIR/$COMMAND_ARCHIVE
EOF

if [[ -n "$MEETING_ARCHIVE" ]]; then
  cat >>"$OUT_DIR/README.md" <<EOF
- meeting_action_brief: $FINAL_OUT_DIR/$MEETING_ARCHIVE
EOF
else
  cat >>"$OUT_DIR/README.md" <<EOF
- meeting_action_brief: package skipped because Python 3.13+ with pydantic 2.x
  was not available during generation.
EOF
fi

cat >>"$OUT_DIR/README.md" <<EOF

The catalog uses local file sources with sha256 checksums. It does not install
dependencies, enable capsules, mount clients, run actions, or mark packages
trusted.

meeting_action_brief requires Python 3.13+ with pydantic 2.x before it becomes
router-routable. command_hello is the low-friction default smoke entry and only
requires a python3 executable.
EOF

if [[ -n "$MEETING_WARNING" ]]; then
  cat >>"$OUT_DIR/README.md" <<EOF

meeting_action_brief packaging warning:

$(printf '%s\n' "$MEETING_WARNING" | sed 's/^/    /')
EOF
fi

rm -rf "$WORK_DIR"
rm -f "$OUT_DIR/meeting_action_brief.warn.log"

rm -rf "$FINAL_OUT_DIR"
mv "$OUT_DIR" "$FINAL_OUT_DIR"
trap - EXIT

echo "$FINAL_CATALOG_PATH"
