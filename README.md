# repo-drag-glance

Five **git log** drag diagnostics on an unfamiliar codebase before you open a file: churn
hotspots, bus factor, bug clusters, delivery pace, and firefighting-style commits,
plus lightweight alert hints.

Detailed behavior and metric definitions live in [`docs/blogpost.md`](docs/blogpost.md).

## Quick start

Requirements: **Rust** (see `rust-version` in [`Cargo.toml`](Cargo.toml)), **git** on `PATH`, and a repo with **at least one commit**.

```bash
cargo build
cargo run -- scan --repo . --source-dir src
```

JSON:

```bash
cargo run -- scan --repo . --format json
```

Install the binary into `~/.cargo/bin`:

```bash
cargo install --path .
repo-drag-glance scan --repo /path/to/repo
```

## Documentation

- [`docs/README.md`](docs/README.md) — doc index for humans and agents
- [`docs/cli-usage.md`](docs/cli-usage.md) — commands, flags, install, tests
- [`docs/architecture.md`](docs/architecture.md) — Rust layout and guardrails
- [`docs/git-metrics.md`](docs/git-metrics.md) — what each signal means

Inspired by [The Git Commands I Run Before Reading Any Code](https://piechowski.io/post/git-commands-before-reading-code/).

## Contributing

```bash
cargo test
cargo fmt --all
cargo clippy --all-targets
```

CI runs fmt, clippy, and tests (see [`.github/workflows/ci.yml`](.github/workflows/ci.yml)).
