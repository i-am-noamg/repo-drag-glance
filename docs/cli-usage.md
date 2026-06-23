# CLI usage

Binary name: `vprdashboard` (same as the Rust package).

## Requirements

- `git` on `PATH`
- A local clone (bare repos work if `git -C <path>` accepts them)
- At least one commit in the repository (empty `git init` with no commits is rejected with a clear error)

## Commands

### `scan`

Runs all five metrics and prints alert hints.

```bash
cargo run -- scan
```

Common flags:

- `--repo <path>` — repository root (default: `.`)
- `--since <git-date>` — passed to `git --since` where applicable (default: `1 year ago`)
- `--top <n>` — max rows for file/author tables (default: `20`)
- `--format table|json` — output (default: `table`)

JSON example:

```bash
cargo run -- scan --format json --repo /path/to/repo
```

### `metrics`

Runs one metric by id or alias:

- `churn` — high-churn files
- `bus_factor` — `git shortlog` on `HEAD` in the window (avoids stdin-only shortlog; not `--all`)
- `bug_hotspots` — commits matching fix|bug|broken
- `delivery_pace` — commits per `YYYY-MM` (full history)
- `firefighting` — oneline subjects matching revert/hotfix/emergency/rollback

```bash
cargo run -- metrics churn --repo . --since "6 months ago"
```

### `explain`

Prints what a metric means and which `git` arguments the tool uses.

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
