//! End-to-end test for the GitHub flow, exercised only when explicitly enabled
//! (the test needs network access). Run with:
//!   GITVIEWR_NETWORK_TESTS=1 cargo test --test github_e2e -- --ignored

use gitviewr_lib::models::repository::SourceType;
use gitviewr_lib::services::analyzer::Analyzer;

fn network_tests_enabled() -> bool {
    std::env::var("GITVIEWR_NETWORK_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[test]
#[ignore = "requires network access"]
fn clones_and_analyzes_public_github_repository() {
    if !network_tests_enabled() {
        eprintln!("skipping: set GITVIEWR_NETWORK_TESTS=1 to enable network tests");
        return;
    }

    let base = tempfile::tempdir().unwrap();
    let url = "https://github.com/octocat/Hello-World";

    let data = Analyzer::analyze_github(url, base.path()).expect("clone + analyze");
    assert_eq!(data.repository.source_type, SourceType::GitHub);
    assert_eq!(data.repository.name, "Hello-World");
    assert!(data.stats.commit_count > 0, "expected some history");
    assert_eq!(data.metadata.default_branch.as_deref(), Some("master"));
    assert!(data.metadata.remote_url.is_some());

    // Cloning again must reuse the managed cache, not fail or duplicate.
    let again = Analyzer::analyze_github(url, base.path()).expect("re-clone reuses cache");
    assert_eq!(again.stats.commit_count, data.stats.commit_count);
}
