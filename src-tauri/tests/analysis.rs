//! End-to-end integration tests for the GitViewr analysis core.

mod common;

use gitviewr_lib::git::errors::{AnalysisError, Result};
use gitviewr_lib::models::analysis::{AnalysisData, FileStat};
use gitviewr_lib::models::commit::FileStatus;
use gitviewr_lib::models::repository::SourceType;
use gitviewr_lib::services::analyzer::Analyzer;

fn analyze_local(path: &std::path::Path) -> Result<gitviewr_lib::models::analysis::AnalysisData> {
    Analyzer::analyze_local(&path.to_string_lossy())
}

#[test]
fn detects_git_repository() {
    let (dir, _repo) = common::repo_with_history(2);
    let data = analyze_local(dir.path()).expect("analysis succeeds");
    assert_eq!(data.repository.source_type, SourceType::Local);
    assert_eq!(
        data.metadata.name,
        dir.path().file_name().unwrap().to_str().unwrap()
    );
}

#[test]
fn rejects_non_git_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.txt"), "hi").unwrap();
    match analyze_local(dir.path()) {
        Err(AnalysisError::NotGitRepository(_)) => {}
        other => panic!("expected NotGitRepository, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn rejects_missing_path() {
    let missing = std::env::temp_dir().join("gitviewr_definitely_missing_dir_xx");
    match analyze_local(&missing) {
        Err(AnalysisError::InvalidPath(_)) => {}
        other => panic!("expected InvalidPath, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn rejects_plain_file_as_repository() {
    let file = tempfile::NamedTempFile::new().unwrap();
    match analyze_local(file.path()) {
        Err(AnalysisError::NotGitRepository(_)) => {}
        other => panic!("expected NotGitRepository, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn extracts_metadata() {
    let (dir, _repo) = common::repo_with_history(5);
    let data = analyze_local(dir.path()).unwrap();
    assert_eq!(data.stats.commit_count, 5);
    assert_eq!(data.stats.tracked_files, 1);
    assert_eq!(data.metadata.contributors.len(), 1);
    assert!(data.stats.first_commit_date.is_some());
    assert!(data.stats.latest_commit_date.is_some());
    // First commit adds "content line i" + "next" (2 lines); the other four
    // commits each replace the single first line (+1).
    assert_eq!(data.stats.total_additions, 6);
    assert_eq!(data.stats.total_deletions, 4);
    assert!(data.stats.branch.is_some());
}

#[test]
fn parses_commits_in_order() {
    let (dir, _repo) = common::repo_with_history(4);
    let data = analyze_local(dir.path()).unwrap();
    assert_eq!(data.commits.len(), 4);
    // Newest commit first.
    assert_eq!(data.commits[0].subject, "commit 3: add changes");
    assert_eq!(data.commits[3].subject, "commit 0: add changes");
    // Each commit touches the one tracked file.
    assert_eq!(data.commits[0].files.len(), 1);
    assert_eq!(data.commits[0].files[0].path, "file.txt");
    assert_eq!(data.commits[0].files[0].status, FileStatus::Modified);
}

#[test]
fn counts_change_statistics() {
    let (dir, _repo) = common::repo_with_history(3);
    let data = analyze_local(dir.path()).unwrap();
    // file.txt had: added in commit 0 (+2), then modified twice (+1 each).
    assert_eq!(data.stats.total_additions, 4);
    let file_stat = data
        .file_stats
        .iter()
        .find(|f| f.path == "file.txt")
        .expect("file present");
    assert_eq!(file_stat.change_count, 3);
    assert_eq!(file_stat.additions, 4);
    assert!(file_stat.last_committed.is_some());
}

#[test]
fn detects_renames() {
    let (dir, _repo) = common::repo_with_rename();
    let data = analyze_local(dir.path()).unwrap();
    let rename_commit = data
        .commits
        .iter()
        .find(|c| c.subject == "rename a.txt to b.txt (same content)")
        .expect("rename commit");
    assert!(
        rename_commit
            .files
            .iter()
            .any(|f| f.status == FileStatus::Renamed && f.path == "b.txt"),
        "expected a renamed delta for b.txt, got {:?}",
        rename_commit.files
    );
}

#[test]
fn aggregates_contributions_by_author() {
    let (dir, _repo) = common::repo_with_two_contributors();
    let data = analyze_local(dir.path()).unwrap();
    let names: Vec<String> = data
        .analytics
        .contributors
        .iter()
        .map(|c| c.name.clone())
        .collect();
    assert!(names.iter().any(|n| n == "Alice"));
    assert!(names.iter().any(|n| n == "Bob"));
    let alice = data
        .analytics
        .contributors
        .iter()
        .find(|c| c.name == "Alice")
        .unwrap();
    assert_eq!(alice.commits, 2);
    assert_eq!(alice.additions, 2);
}

#[test]
fn produces_time_buckets() {
    let (dir, _repo) = common::repo_with_history(6);
    let data = analyze_local(dir.path()).unwrap();
    assert!(!data.analytics.commits_over_time.is_empty());
    // All commits in the current month -> one bucket with 6 commits.
    let bucket = &data.analytics.commits_over_time[0];
    assert!(bucket.period.len() == 7, "period is YYYY-MM");
    assert_eq!(bucket.commits, 6);
}

#[test]
fn github_url_rejects_invalid() {
    let err = Analyzer::analyze_github("not-a-url", std::path::Path::new("/tmp")).unwrap_err();
    assert!(matches!(err, AnalysisError::InvalidGithubUrl(_)));
}

#[test]
fn empty_repository_is_not_confused_with_non_git() {
    let (dir, _repo) = common::empty_repo();
    let data = analyze_local(dir.path()).expect("empty git repo analyzes cleanly");
    assert_eq!(data.commits.len(), 0);
    assert_eq!(data.stats.commit_count, 0);
    assert_eq!(data.stats.total_additions, 0);
}

fn file_stat<'a>(data: &'a AnalysisData, path: &str) -> &'a FileStat {
    data.file_stats
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("expected a file stat for {path}"))
}

#[test]
fn file_stat_status_is_some_for_every_file() {
    let (dir, _repo) = common::repo_with_statuses();
    let data = analyze_local(dir.path()).unwrap();
    for stat in &data.file_stats {
        assert!(
            stat.status.is_some(),
            "{} should have a backend-computed status, got None",
            stat.path
        );
    }
}

#[test]
fn file_stat_status_last_op_after_delete() {
    // a.txt: Added -> Modified -> Deleted. Status must be the most recent op.
    let (dir, _repo) = common::repo_with_statuses();
    let data = analyze_local(dir.path()).unwrap();
    let stat = file_stat(&data, "a.txt");
    assert_eq!(stat.status, Some(FileStatus::Deleted));
    assert_eq!(stat.change_count, 3);
}

#[test]
fn file_stat_status_added_only() {
    let (dir, _repo) = common::repo_with_statuses();
    let data = analyze_local(dir.path()).unwrap();
    assert_eq!(file_stat(&data, "bin.dat").status, Some(FileStatus::Added));
    assert_eq!(
        file_stat(&data, "moved1.txt").status,
        Some(FileStatus::Added)
    );
}

#[test]
fn file_stat_status_rename_then_modify_is_modified() {
    let (dir, _repo) = common::repo_with_statuses();
    let data = analyze_local(dir.path()).unwrap();
    let stat = file_stat(&data, "moved2.txt");
    assert_eq!(stat.status, Some(FileStatus::Modified));
    assert_eq!(stat.change_count, 2);
}

#[test]
fn file_stat_last_op_wins_with_equal_timestamps() {
    // Two commits changing the same file within one second share the same
    // author timestamp; the newer commit (visited first) must still win.
    let (dir, repo) = common::empty_repo();
    common::commit_files(
        &repo,
        &common::sig_at(common::BASE_TIME),
        "add",
        &[("f.txt", "one\n")],
    );
    common::commit_files(
        &repo,
        &common::sig_at(common::BASE_TIME),
        "modify within same second",
        &[("f.txt", "one\ntwo\n")],
    );
    let data = analyze_local(dir.path()).unwrap();
    assert_eq!(file_stat(&data, "f.txt").status, Some(FileStatus::Modified));
}

#[test]
fn timeline_is_cumulative_net_growth() {
    let (dir, repo) = common::empty_repo();
    // First commit (November 2020): one file with 2 lines added.
    common::commit_files(
        &repo,
        &common::sig_at(common::BASE_TIME),
        "one",
        &[("a.txt", "line1\nline2\n")],
    );
    // Second commit, ~40 days later (December 2020): 1 net line added.
    common::commit_files(
        &repo,
        &common::sig_at(common::BASE_TIME + 40 * 24 * 3600),
        "two",
        &[("a.txt", "line1\nline2\nline3\n")],
    );
    let data = analyze_local(dir.path()).unwrap();
    let timeline = &data.analytics.timeline;
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].net, 2);
    assert_eq!(timeline[1].net, 3);
    assert_eq!(
        timeline[1].net, data.stats.net_change,
        "last point is total net change"
    );
    // The growth curve must track the bucket periods.
    assert_eq!(
        timeline
            .iter()
            .map(|p| p.period.clone())
            .collect::<Vec<_>>(),
        data.analytics
            .commits_over_time
            .iter()
            .map(|b| b.period.clone())
            .collect::<Vec<_>>()
    );
}
