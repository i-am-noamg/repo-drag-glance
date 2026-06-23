# VPR Dashboard Docs

Purpose:

- Provide a simple, open-source dashboard that surfaces git-based health signals
  so a VP R&D can spot anomalies early.

Audience:

- Engineering leaders and senior engineers who need quick, reliable signals.

Docs:

- `docs/git-metrics.md` - the initial set of git command metrics and how to read them
- `docs/cli-usage.md` - CLI commands, flags, install, and how tests are run
- `docs/architecture.md` - Rust CLI architecture, layout, and guardrails

Sources:

- Inspired by [The Git Commands I Run Before Reading Any Code](https://piechowski.io/post/git-commands-before-reading-code/)

Notes:

- These metrics are only as good as the commit history. If teams squash or
  write vague messages, some signals will be weaker.
