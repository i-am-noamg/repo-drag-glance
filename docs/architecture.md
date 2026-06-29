# CLI Architecture (Rust)

This repo ships a Rust CLI that runs the five git commands from [`docs/blogpost.md`](blogpost.md).
It stays simple, fast, and cross-platform.

## Goals

- Single CLI entry point, easy to run locally.
- Drive `git` via `std::process::Command` with explicit arguments (no shell
  pipelines).
- Small surface area; a future web UI can reuse the same **JSON report shape**
  without sharing Rust code.
- **`docs/blogpost.md`** is the canonical spec for metric behavior.

## Crate layout

```text
vprdashboard/
  Cargo.toml
  src/
    lib.rs              # library root (metrics + git + report for tests)
    main.rs             # thin entry: clap dispatch
    cli.rs               # clap `Cli` / `CommonOpts` / subcommands
    model.rs             # MetricId, MetricResult, ScanReport, AlertHint, OutputFormat
    alerts.rs            # alert hints from metric results
    commands/
      mod.rs
      scan.rs
      metrics.rs
      explain.rs
    git/
      mod.rs             # git helpers + parsers (also unit tests)
      run.rs             # `git_stdout`: -C repo, stdin null
      error.rs            # GitError (thiserror)
    metrics/
      mod.rs             # ScanOptions, run_all / run_single
    report/
      mod.rs             # table + JSON render
  tests/
    cli_integration.rs   # temp repo + binary smoke tests
  docs/                  # AI-first documentation
```

## CLI flow

1. Parse args with `clap`.
2. `scan` / `metrics`: `git::check_has_commits` (empty repo → clear error).
3. Run git queries via `git_stdout` (stdin always **null** — see below).
4. Parse stdout in Rust; build `MetricResult` values.
5. `alerts::compute_alerts` → attach to `ScanReport`.
6. `report::render` → table or JSON.

## Per-metric git invocations

| Metric | Git args | `--since` | `--source-dir` |
|--------|----------|-----------|----------------|
| churn | `log --format=format: --name-only --since … [-- pathspecs]` | yes | pathspec + post-filter |
| bus_factor | `shortlog -sn --no-merges HEAD` (+ recent window for alerts) | no | no |
| bug_hotspots | `log -i -E --grep=fix\|bug\|broken --name-only --format= [-- pathspecs]` | no | pathspec + post-filter |
| delivery_pace | `log --format=%ad --date=format:%Y-%m` | no | no |
| firefighting | `log --oneline --since …` + keyword filter in Rust | yes | no |

File metrics count non-empty path lines (blog: `sort | uniq -c`), optionally filtered to `--source-dir` prefixes.

## Git subprocess rules

- **Stdin is always closed** (`Stdio::null()` in `git_stdout`). Some git
  commands, notably `git shortlog` **without a revision**, read commits from
  stdin; with a null stdin that looks like “zero contributors”. We always pass
  an explicit **`HEAD`** for shortlog (current branch only).
- Use `git -C <repo> …` rather than `current_dir` + relative git.

## Metric / report types

- **`MetricId`**: stable ids (`churn`, `bus_factor`, …) for CLI and JSON.
- **`MetricResult`**: id, label, summary, optional `rows`, optional `scalar`.
- **`ScanReport`**: `warnings`, `repo`, `since`, `recent_since`, `source_dirs`, `metrics`, `alerts`.
- **`AlertHint`**: `severity`, `code`, `message`, optional `evidence`.

## Guardrails

- **Stable Rust** only; `rust-version` in `Cargo.toml` (currently 1.75+); no
  nightly-only features.
- **`cargo fmt`** and **`cargo clippy`** in CI (default lints; CI uses
  `-D warnings` on clippy).
- **`anyhow`** at command boundaries; **`thiserror`** for `GitError`.
- Keep dependencies minimal; each crate should have a clear reason.

## Dependencies (current)

- `clap` — CLI
- `serde` / `serde_json` — JSON report
- `anyhow` / `thiserror` — errors
- `tabled` — terminal tables

Dev: `tempfile`, `serde_json` (integration tests).

## Subcommands

- **`scan`** — all five metrics + alerts; `--format table|json`.
- **`metrics <name>`** — one metric + alerts for that slice.
- **`explain <name>`** — blogpost command + CLI git equivalent.

## Testing

- **Unit tests** in `src/**` modules (`#[cfg(test)]`).
- **Integration tests** in `tests/cli_integration.rs` (see `tests/README.md`).
- **CI**: `.github/workflows/ci.yml` — fmt, clippy, test on Ubuntu.
