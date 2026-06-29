# VPR Dashboard Docs

Purpose:

- Provide a simple, open-source dashboard that surfaces git-based health signals
  so a VP R&D can spot anomalies early.

Audience:

- Engineering leaders and senior engineers who need quick, reliable signals.

Docs:

- `docs/blogpost.md` - canonical reference: five git commands and workflow
- `docs/git-metrics.md` - metric definitions aligned with the blogpost
- `docs/cli-usage.md` - CLI commands, flags, install, and how tests are run
- `docs/architecture.md` - Rust CLI architecture, layout, and guardrails

Sources:

- Inspired by [The Git Commands I Run Before Reading Any Code](https://piechowski.io/post/git-commands-before-reading-code/)

Notes:

- These metrics are only as good as the commit history. If teams squash or
  write vague messages, some signals will be weaker.
