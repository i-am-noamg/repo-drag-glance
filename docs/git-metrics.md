# Git Health Metrics

Canonical source: [`docs/blogpost.md`](blogpost.md) — five git commands run before reading code.

## 1) High-churn files

Run from source directories (`src/`, `app/`), not the repo root — lockfiles and generated files dominate otherwise.

Command:

```bash
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20
```

CLI: `--source-dir src` (repeatable), `--since "1 year ago"`, `--top 20`.

Why it matters:

- High churn can indicate unstable architecture or concentrated risk.
- Cross-check with bug hotspots to find files that change a lot and break a lot.

## 2) Bus factor / ownership concentration

Full history on the current branch (`HEAD`):

```bash
git shortlog -sn --no-merges
```

Secondary window for departed-contributor check:

```bash
git shortlog -sn --no-merges --since="6 months ago"
```

CLI: `--recent-since "6 months ago"`. The tool passes `HEAD` explicitly so `shortlog` does not read from stdin (empty under a closed stdin in subprocesses).

Why it matters:

- If one person dominates, a departure creates knowledge risk.
- If the top contributor from full history is absent in the recent window, flag it immediately.

## 3) Bug hotspots

Same source-dir scoping as churn. Full history (no `--since` in the blog command):

```bash
git log -i -E --grep="fix|bug|broken" --name-only --format='' | sort | uniq -c | sort -nr | head -20
```

CLI: `--source-dir src` (repeatable), `--top 20`.

Why it matters:

- Frequent bug fixes in the same files are a strong quality signal.
- Compare against churn hotspots; overlap is highest risk.

## 4) Delivery pace

Commit volume by month, full history:

```bash
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c
```

Why it matters:

- Sharp drops can indicate staffing changes or stalled delivery.
- Spiky output can indicate batched releases instead of steady shipping.

## 5) Firefighting frequency

```bash
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback'
```

CLI: `--since "1 year ago"`.

Why it matters:

- Frequent reverts indicate fragile deployment or low confidence in changes.

## Per-metric `--since` rules

| Metric | `--since` | `--recent-since` | `--source-dir` |
|--------|-----------|------------------|----------------|
| churn | yes (default `1 year ago`) | — | yes |
| bus_factor | no (full history) | yes (alerts) | no |
| bug_hotspots | no (full history) | — | yes |
| delivery_pace | no (full history) | — | no |
| firefighting | yes (default `1 year ago`) | — | no |
