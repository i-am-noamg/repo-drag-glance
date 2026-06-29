use anyhow::bail;

use crate::model::MetricId;

pub fn run_explain(name: &str) -> anyhow::Result<()> {
    let Some(id) = MetricId::parse(name) else {
        bail!("unknown metric {:?}; try: churn, bus_factor, bug_hotspots, delivery_pace, firefighting", name);
    };
    println!("{} ({})", id.label(), id.as_str());
    println!();
    println!("{}", id.description());
    println!();
    println!("Blogpost command (from docs/blogpost.md):");
    println!();
    match id {
        MetricId::Churn => {
            println!("  # Run from source dirs (not repo root); CLI: --source-dir src --source-dir apps");
            println!("  git log --format=format: --name-only --since=\"<since>\" -- <source-dirs>");
            println!("    | sort | uniq -c | sort -nr | head -<top>");
            println!();
            println!("CLI equivalent:");
            println!("  git log --format=format: --name-only --since <since> [-- pathspecs]");
            println!("  (paths counted once per log line; filtered to --source-dir prefixes)");
        }
        MetricId::BusFactor => {
            println!("  git shortlog -sn --no-merges");
            println!();
            println!("Secondary window (departed-contributor check):");
            println!("  git shortlog -sn --no-merges --since=\"<recent-since>\"");
            println!();
            println!("CLI equivalent:");
            println!("  git shortlog -sn --no-merges HEAD");
            println!("  git shortlog -sn --no-merges --since <recent-since> HEAD");
            println!("`HEAD` is passed so shortlog does not read from stdin (empty under closed stdin).");
        }
        MetricId::BugHotspots => {
            println!("  # Same source-dir scoping as churn");
            println!("  git log -i -E --grep=\"fix|bug|broken\" --name-only --format='' -- <source-dirs>");
            println!("    | sort | uniq -c | sort -nr | head -<top>");
            println!();
            println!("CLI equivalent:");
            println!("  git log -i -E --grep=fix|bug|broken --name-only --format= [-- pathspecs]");
            println!("  (full history; no --since in the blog command)");
        }
        MetricId::DeliveryPace => {
            println!("  git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c");
            println!();
            println!("CLI equivalent: same git args; counts in Rust.");
        }
        MetricId::Firefighting => {
            println!("  git log --oneline --since=\"<since>\" | grep -iE 'revert|hotfix|emergency|rollback'");
            println!();
            println!("CLI equivalent:");
            println!("  git log --oneline --since <since>");
            println!("  (subjects filtered in Rust for: revert, hotfix, emergency, rollback)");
        }
    }
    Ok(())
}
