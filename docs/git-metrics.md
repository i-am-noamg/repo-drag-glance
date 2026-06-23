# Git Health Metrics

This dashboard starts with a small set of git-derived signals. Each metric
should map to a simple card in the UI with a short explanation and trend.

## 1) High-churn files

Shows the most-changed files in a time window.

Command:

```bash
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20
```

Why it matters:

- High churn can indicate unstable architecture or concentrated risk.
- Cross-check with bug hotspots to find files that change a lot and break a lot.

## 2) Bus factor / ownership concentration

Shows who commits most often.

Command:

```bash
git shortlog -sn --no-merges
```

When driving `git` from a subprocess, pass an explicit revision (this tool uses
`HEAD`, i.e. the current branch). With no revision, `shortlog` reads commit
objects from **stdin**; a closed stdin yields an empty list and looks like
“zero contributors”. We avoid `--all` here: it walks every ref under `refs/`
and can double-count and include unrelated branches.

Why it matters:

- If one person dominates, a departure creates knowledge risk.
- Compare against a recent window to see if key owners are still active.

## 3) Bug hotspots

Finds files that appear in commits with bug-like keywords.

Command:

```bash
git log -i -E --grep="fix|bug|broken" --name-only --format='' | sort | uniq -c | sort -nr | head -20
```

Why it matters:

- Frequent bug fixes in the same files are a strong quality signal.
- This depends on commit message discipline.

## 4) Delivery pace

Commit volume by month.

Command:

```bash
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c
```

Why it matters:

- Sharp drops can indicate staffing changes or stalled delivery.
- Spiky output can indicate batched releases instead of steady shipping.

## 5) Firefighting frequency

Counts explicit reverts or emergency terms.

Command:

```bash
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback'
```

Why it matters:

- Frequent reverts indicate fragile deployment or low confidence in changes.

## Initial dashboard ideas

- A list of metric cards with counts and a short interpretation.
- A drill-down table that links top files to their recent commits.
- A simple alert threshold (for example, churn or bug hotspots above a limit).
