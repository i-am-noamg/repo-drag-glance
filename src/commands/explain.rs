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
    match id {
        MetricId::Churn => {
            println!("Git invocation (no shell pipeline):");
            println!("  git log --since <since> --name-only --format=%H");
            println!("Files are counted once per commit they appear in.");
        }
        MetricId::BusFactor => {
            println!("Git invocation:");
            println!("  git shortlog -sn --no-merges --since <since> HEAD");
            println!("`HEAD` is passed so shortlog does not read from stdin (empty under `output()`).");
            println!("This counts authors on the current branch only, not every ref under refs/.");
        }
        MetricId::BugHotspots => {
            println!("Git invocation:");
            println!("  git log -i -E --grep=fix|bug|broken --since <since> --name-only --format=%H");
        }
        MetricId::DeliveryPace => {
            println!("Git invocation:");
            println!("  git log --format=%ad --date=format:%Y-%m");
            println!("Counts commits per calendar month across full history.");
        }
        MetricId::Firefighting => {
            println!("Git invocation:");
            println!("  git log --oneline --since <since>");
            println!("Then filters subjects for: revert, hotfix, emergency, rollback (case insensitive).");
        }
    }
    Ok(())
}
