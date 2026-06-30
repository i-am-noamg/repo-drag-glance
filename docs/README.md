# repo-drag-glance Docs

Purpose:

- Provide a simple, open-source CLI for the five git-log drag diagnostics from
  [`blogpost.md`](blogpost.md) — a first pass on an unfamiliar codebase before you read code.

Audience:

- R&D consultants and staff engineers triaging a codebase before reading code.

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
