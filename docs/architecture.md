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
repo-drag-glance/
  Cargo.toml
  src/
    lib.rs              # library root (metrics + git + report for tests)
    main.rs             # thin entry: clap dispatch
    cli.rs               # clap `Cli` / `CommonOpts` / subcommands
    model.rs             # MetricId, MetricResult, ScanReport, AlertHint, OutputFormat
    validate.rs          # CLI input validation (source-dir, since, top)
    sanitize.rs          # strip ANSI/control chars from git-derived output
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
2. `validate::validate_common_opts` (reject abusive flags before git runs).
3. `scan` / `metrics`: `git::check_has_commits` (empty repo → clear error).
4. Run git queries via `git_stdout` (stdin always **null** — see below).
5. Parse stdout in Rust; build `MetricResult` values.
6. `alerts::compute_alerts` → attach to `ScanReport`.
7. `report::render` → sanitize git-derived strings → table or JSON.

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
- **Child environment is scrubbed** — only a small allowlist (`PATH`, `HOME`, …)
  is inherited; `GIT_*` and dynamic-linker injection vars are dropped.

## Metric / report types

- **`MetricId`**: stable ids (`churn`, `bus_factor`, …) for CLI and JSON.
- **`MetricResult`**: id, label, summary, optional `rows`, optional `scalar`.
- **`ScanReport`**: `warnings`, `repo`, `since`, `recent_since`, `source_dirs`, `metrics`, `alerts`.
- **`AlertHint`**: `severity`, `code`, `message`, optional `evidence`.

## Guardrails

- **Stable Rust** only; `rust-version` in `Cargo.toml` (currently 1.85+); no
  nightly-only features. CI runs a dedicated **msrv** job on that version.
- **[`rust-toolchain.toml`](../rust-toolchain.toml)** — local dev uses stable with
  `rustfmt` and `clippy`.
- **`cargo fmt`** and **`cargo clippy`** in CI (default lints; CI uses
  `-D warnings` on clippy and `--locked` builds).
- **`anyhow`** at command boundaries; **`thiserror`** for `GitError`.
- Keep dependencies minimal; each crate should have a clear reason.

## Security

See [`SECURITY.md`](../SECURITY.md) for threat model and reporting.

Subprocess rules (see also `src/git/run.rs`):

- **No shell** — `git` args are a fixed argv list.
- **Scrubbed environment** — `GIT_*`, `LD_PRELOAD`, `DYLD_*`, and
  `REPO_DRAG_GLANCE_*` are not inherited by git children. Override the git
  binary with `REPO_DRAG_GLANCE_GIT` (single-line path).
- **Validated CLI input** — `src/validate.rs` rejects abusive `--source-dir`
  pathspecs, `--` values, and oversized `--since` / `--top`.
- **Sanitized output** — git-derived strings pass through `src/sanitize.rs`
  before table/JSON render (ANSI/control stripping).
- **Redacted errors** — git stderr is hidden unless `REPO_DRAG_GLANCE_VERBOSE=1`.

Supply chain: [`deny.toml`](../deny.toml), RustSec audit, and Dependabot in CI.

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
  Require **git** on `PATH` (CI verifies this on all platforms).
- **CI** ([`.github/workflows/ci.yml`](../.github/workflows/ci.yml)):
  - **fmt** — `cargo fmt --all -- --check` (Ubuntu)
  - **clippy** — `cargo clippy --all-targets --locked -- -D warnings` (Ubuntu)
  - **test** — `cargo test --locked` on Ubuntu, macOS, and Windows
  - **msrv** — `cargo test --locked` on Rust 1.85
  - **install-smoke** — `cargo install --path . --locked` + binary smoke test
  - **audit** — RustSec advisory check (`rustsec/audit-check`)
  - **deny** — license/advisory/source policy (`cargo deny check all`)
- **Dependabot** (`.github/dependabot.yml`) — weekly Cargo and monthly GitHub Actions updates.
