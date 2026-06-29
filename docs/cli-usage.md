# CLI usage

Binary name: `vprdashboard` (same as the Rust package).

Canonical metric definitions: [`docs/blogpost.md`](blogpost.md).

## Requirements

- `git` on `PATH`
- A local clone (bare repos work if `git -C <path>` accepts them)
- At least one commit in the repository (empty `git init` with no commits is rejected with a clear error)

## Commands

### `scan`

Runs all five metrics and prints alert hints.

```bash
cargo run -- scan --repo . --source-dir src
```

Common flags:

- `--repo <path>` — repository root (default: `.`)
- `--source-dir <path>` — repeatable; scopes churn and bug_hotspots (blog: run from `src/` or `app/`)
- `--since <git-date>` — churn and firefighting only (default: `1 year ago`)
- `--recent-since <git-date>` — bus-factor departed-contributor check (default: `6 months ago`)
- `--top <n>` — max rows for file/author tables (default: `20`)
- `--format table|json` — output (default: `table`)

When `--source-dir` is omitted, file metrics scan the whole repo and a warning is printed at the start of the output.

JSON example:

```bash
cargo run -- scan --format json --repo /path/to/repo --source-dir src --source-dir apps
```

### `metrics`

Runs one metric by id or alias:

- `churn` — high-churn files (`--since`, `--source-dir`)
- `bus_factor` — full-history shortlog on `HEAD` (`--recent-since` for alerts)
- `bug_hotspots` — commits matching fix|bug|broken (`--source-dir`, no `--since`)
- `delivery_pace` — commits per `YYYY-MM` (full history)
- `firefighting` — oneline subjects matching revert/hotfix/emergency/rollback (`--since`)

```bash
cargo run -- metrics churn --repo . --source-dir src --since "1 year ago"
cargo run -- metrics bus_factor --repo . --recent-since "6 months ago"
```

### `explain`

Prints the blogpost command and the CLI's git equivalent.

```bash
cargo run -- explain bus_factor
```

## Install (from source)

```bash
cargo install --path .
```

## Tests

```bash
cargo test
```

See [`tests/README.md`](../tests/README.md): integration tests build a temporary
git repository and run the `vprdashboard` binary (`CARGO_BIN_EXE_vprdashboard`).
