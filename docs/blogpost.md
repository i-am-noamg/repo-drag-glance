---
title: The Git Commands I Run Before Reading Any Code
description: "Five git log commands that diagnose a new codebase before you open a single file: code churn hotspots, bus factor, bug clusters, and crisis patterns."
url: https://piechowski.io/post/git-commands-before-reading-code/
site: piechowski.io
author: Ally Piechowski
publish_date: 2026-04-08T08:30:00Z
---

# The Git Commands I Run Before Reading Any Code

Five git log commands that diagnose a new codebase before you open a single file: code churn hotspots, bus factor, bug clusters, and crisis patterns.

Ally Piechowski · Apr 8, 2026 · 4 min read

* [development](/tags/development)
* [git](/tags/git)
* [legacy](/tags/legacy)
* [technical-debt](/tags/technical-debt)

![The Git Commands I Run Before Reading Any Code](/post/git-commands-before-reading-code/cover_hu3f66e25b7571f7e32d40f355f31a2ca9_56928_1500x0_resize_q75_h2_box_2.webp)

The first thing I usually do when I pick up a new codebase isn’t opening the code. It’s opening a terminal and running a handful of git commands. Before I look at a single file, the commit history gives me a diagnostic picture of the project: who built it, where the problems cluster, whether the team is shipping with confidence or tiptoeing around land mines.

## What Changes the Most

```bash
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20
```

I run this from `app/` or `src/`, not the repo root. Lockfiles, changelogs, and generated code will dominate the list otherwise.

The 20 most-changed files in the last year. The file at the top is almost always the one people warn me about. “Oh yeah, that file. Everyone’s afraid to touch it.”

High churn on a file doesn’t mean it’s bad. Sometimes it’s just active development. But high churn on a file that nobody wants to own is the clearest signal of codebase drag I know. That’s the file where every change is a patch on a patch. The blast radius of a small edit is unpredictable. The team pads their estimates because they know it’s going to fight back.

A [2005 Microsoft Research study](https://www.microsoft.com/en-us/research/publication/use-of-relative-code-churn-measures-to-predict-system-defect-density/) found churn-based metrics predicted defects more reliably than complexity metrics alone. I take the top 5 files from this list and cross-reference them against the bug hotspot command below. A file that’s high-churn *and* high-bug is your single biggest risk. Adam Tornhill’s [Your Code as a Crime Scene](https://pragprog.com/titles/atcrime2/your-code-as-a-crime-scene-second-edition/) builds a full methodology around churn-based analysis, including complexity overlays that these raw commands don’t cover.

## Who Built This

```bash
git shortlog -sn --no-merges
```

Every contributor ranked by commit count. If one person accounts for 60% or more, that’s your bus factor. If they left six months ago, it’s a crisis. If the top contributor from the overall shortlog doesn’t appear in a 6-month window (`git shortlog -sn --no-merges --since="6 months ago"`), I flag that to the client immediately.

I also look at the tail. Thirty contributors but only three active in the last year. The people who built this system aren’t the people maintaining it.

One caveat: squash-merge workflows compress authorship. If the team squashes every PR into a single commit, this output reflects who merged, not who wrote. Worth asking about the merge strategy before drawing conclusions.

## Where Do Bugs Cluster

```bash
git log -i -E --grep="fix|bug|broken" --name-only --format='' | sort | uniq -c | sort -nr | head -20
```

Same shape as the churn command, filtered to commits with bug-related keywords. Compare this list against the churn hotspots. Files that appear on both are your highest-risk code: they keep breaking and keep getting patched, but never get properly fixed.

This depends on commit message discipline. If the team writes “update stuff” for every commit, you’ll get nothing. But even a rough map of bug density is better than no map.

## Is This Project Accelerating or Dying

```bash
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c
```

Commit count by month, for the entire history of the repo. I scan the output looking for shapes. A steady rhythm is healthy. But what does it look like when the count drops by half in a single month? Usually someone left. A declining curve over 6 to 12 months tells you the team is losing momentum. Periodic spikes followed by quiet months means the team batches work into releases instead of shipping continuously.

I once showed a CTO their commit velocity chart and they said “that’s when we lost our second senior engineer.” They hadn’t connected the timeline before. This is team data, not code data.

## How Often Is the Team Firefighting

```bash
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback'
```

Revert and hotfix frequency. A handful over a year is normal. Reverts every couple of weeks means the team doesn’t trust its deploy process. They’re evidence of a [deeper issue](/post/codebase-drag-audit/#2-deploy-fear): unreliable tests, missing staging, or a deploy pipeline that makes rollbacks harder than they should be. Zero results is also a signal; either the team is stable, or nobody writes descriptive commit messages.

Crisis patterns are easy to read. Either they’re there or they’re not.

***

These five commands take a couple minutes to run. They won’t tell you everything. But you’ll know which code to read first, and what to look for when you get there. That’s the difference between spending your first day reading the codebase methodically and spending it wandering.

This is the first hour of what I do in a [full codebase audit](/post/how-i-audit-a-legacy-rails-codebase/).

***

## Related Articles

* [Why Your Engineering Team Is Slow (It's the Codebase, Not the People)](/post/codebase-drag-audit/)
* [How I Audit a Legacy Rails Codebase in the First Week](/post/how-i-audit-a-legacy-rails-codebase/)
* [Ruby 3.2 Is EOL: What You Actually Need to Do](/post/ruby-3-2-eol/)
* [Rails 7.2 to 8.1 Upgrade: What Actually Breaks and How to Fix It](/post/rails-7-to-8-upgrade-what-actually-breaks/)
* [How to Close a Tab in Vim](/post/vim-tabclose/)

---

Powered by [curl.md](https://curl.md)