use anyhow::bail;

use crate::model::MetricId;
use crate::report::Style;

pub fn run_explain(name: &str, no_color: bool) -> anyhow::Result<()> {
    let style = Style::new(no_color);
    let Some(id) = MetricId::parse(name) else {
        bail!("unknown metric {:?}; try: churn, bus_factor, bug_hotspots, delivery_pace, firefighting", name);
    };
    println!("{} ({})", style.section(id.label()), id.as_str());
    println!();
    println!("{}", style.summary(id.description()));
    println!();
    println!(
        "{}",
        style.header_label("Blogpost command (from docs/blogpost.md):")
    );
    println!();
    match id {
        MetricId::Churn => {
            println!(
                "  # Run from source dirs (not repo root); CLI: --source-dir src --source-dir apps"
            );
            println!("  git log --format=format: --name-only --since=\"<since>\" -- <source-dirs>");
            println!("    | sort | uniq -c | sort -nr | head -<top>");
            println!();
            println!("{}", style.header_label("CLI equivalent:"));
            println!("  git log --format=format: --name-only --since <since> [-- pathspecs]");
            println!("  (paths counted once per log line; filtered to --source-dir prefixes)");
        }
        MetricId::BusFactor => {
            println!("  git shortlog -sn --no-merges");
            println!();
            println!(
                "{}",
                style.header_label("Secondary window (departed-contributor check):")
            );
            println!("  git shortlog -sn --no-merges --since=\"<recent-since>\"");
            println!();
            println!("{}", style.header_label("CLI equivalent:"));
            println!("  git shortlog -sn --no-merges HEAD");
            println!("  git shortlog -sn --no-merges --since <recent-since> HEAD");
            println!(
                "`HEAD` is passed so shortlog does not read from stdin (empty under closed stdin)."
            );
        }
        MetricId::BugHotspots => {
            println!("  # Same source-dir scoping as churn");
            println!("  git log -i -E --grep=\"fix|bug|broken\" --name-only --format='' -- <source-dirs>");
            println!("    | sort | uniq -c | sort -nr | head -<top>");
            println!();
            println!("{}", style.header_label("CLI equivalent:"));
            println!("  git log -i -E --grep=fix|bug|broken --name-only --format= [-- pathspecs]");
            println!("  (full history; no --since in the blog command)");
        }
        MetricId::DeliveryPace => {
            println!(
                "  git log --format='%ad' --date=format:'%Y-%m' --since=\"<since>\" | sort | uniq -c"
            );
            println!();
            println!("{}", style.header_label("CLI equivalent:"));
            println!("  git log --format=%ad --date=format:%Y-%m --since <since>");
            println!("  (CLI defaults --since to \"1 year ago\"; blog uses full history)");
        }
        MetricId::Firefighting => {
            println!("  git log --oneline --since=\"<since>\" | grep -iE 'revert|hotfix|emergency|rollback'");
            println!();
            println!("{}", style.header_label("CLI equivalent:"));
            println!("  git log --oneline --since <since>");
            println!("  (subjects filtered in Rust for: revert, hotfix, emergency, rollback)");
        }
    }
    Ok(())
}
