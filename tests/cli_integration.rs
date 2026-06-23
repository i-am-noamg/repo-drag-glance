//! Integration tests: temp git repo + CLI JSON smoke.

use std::fs;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("run git");
    assert!(st.success(), "git {:?} failed", args);
}

fn init_fixture_repo(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "fixture@vprdashboard.test"],
    );
    git(root, &["config", "user.name", "Fixture"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "// lib\n").unwrap();
    fs::write(root.join("README.md"), "# x\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "init"]);
    fs::write(root.join("src/lib.rs"), "// lib fix bug\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "fix bug in lib"]);
    git(root, &["commit", "--allow-empty", "-m", "Revert bad deploy"]);
}

fn vprdashboard_bin() -> std::path::PathBuf {
    if let Some(p) = std::env::var_os("CARGO_BIN_EXE_vprdashboard") {
        return std::path::PathBuf::from(p);
    }
    let target = std::env::var_os("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR");
    std::path::PathBuf::from(target)
        .join("debug")
        .join("vprdashboard")
}

#[test]
fn scan_json_has_metrics_and_alerts() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = Command::new(vprdashboard_bin())
        .args([
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--since",
            "2000 years ago",
            "--format",
            "json",
        ])
        .output()
        .expect("run vprdashboard");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(v.get("metrics").and_then(|m| m.as_array()).is_some());
    assert!(v.get("alerts").and_then(|a| a.as_array()).is_some());
    let metrics = v["metrics"].as_array().unwrap();
    assert_eq!(metrics.len(), 5);
    let ids: Vec<_> = metrics
        .iter()
        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
        .collect();
    assert!(ids.contains(&"churn"));
    assert!(ids.contains(&"firefighting"));
}

#[test]
fn metrics_single_churn_json() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = Command::new(vprdashboard_bin())
        .args([
            "metrics",
            "churn",
            "--repo",
            repo.to_str().unwrap(),
            "--since",
            "2000 years ago",
            "--format",
            "json",
        ])
        .output()
        .expect("run vprdashboard");

    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(v["metrics"].as_array().map(|a| a.len()), Some(1));
}

#[test]
fn scan_fails_clearly_on_repo_with_no_commits() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    fs::create_dir_all(repo).unwrap();
    git(repo, &["init"]);

    let out = Command::new(vprdashboard_bin())
        .args([
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--since",
            "1 year ago",
            "--format",
            "json",
        ])
        .output()
        .expect("run vprdashboard");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no commits") || stderr.contains("empty history"),
        "expected empty-repo hint, got: {stderr}"
    );
}
