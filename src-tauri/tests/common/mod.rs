#![allow(dead_code)]

//! Shared helpers for integration tests that create real Git repositories
//! and exercise the analysis core end to end.

use std::path::Path;

use git2::{Commit, Oid, Repository, Signature};
use gitviewr_lib::git::errors::Result;
use gitviewr_lib::models::analysis::AnalysisData;
use gitviewr_lib::services::analyzer::Analyzer;

pub const AUTHOR_NAME: &str = "Test Author";
pub const AUTHOR_EMAIL: &str = "test@example.com";

pub fn signature() -> Signature<'static> {
    Signature::now(AUTHOR_NAME, AUTHOR_EMAIL).expect("signature")
}

/// A signature pinned to an explicit wall-clock time (git seconds), so tests
/// get deterministic commit ordering even when created within one second.
pub fn sig_at(secs: i64) -> Signature<'static> {
    Signature::new(AUTHOR_NAME, AUTHOR_EMAIL, &git2::Time::new(secs, 0)).expect("signature")
}

pub const BASE_TIME: i64 = 1_600_000_000;

/// Create an empty repository plus its temp working dir.
pub fn empty_repo() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init repo");
    (dir, repo)
}

/// Stage the given files (adding missing parents), write the tree and commit.
pub fn commit_files(
    repo: &Repository,
    sig: &Signature,
    message: &str,
    files: &[(&str, &str)],
) -> Oid {
    let workdir = repo.workdir().expect("workdir").to_path_buf();
    let mut index = repo.index().expect("index");
    let mut paths = Vec::new();
    for (name, content) in files {
        let p = workdir.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&p, content).expect("write file");
        index.add_path(Path::new(name)).expect("add path");
        paths.push(name.to_string());
    }
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let parents: Vec<Commit> = repo
        .head()
        .ok()
        .map(|h| h.peel_to_commit().expect("peel head"))
        .into_iter()
        .collect();
    let parent_refs: Vec<&Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), sig, sig, message, &tree, &parent_refs)
        .expect("commit")
}

/// Analyze a path with the public analyzer.
pub fn analyze(path: &str) -> Result<AnalysisData> {
    Analyzer::analyze_local(path)
}

/// Build a repo with a small history where `file.txt` changes across commits.
pub fn repo_with_history(commit_count: usize) -> (tempfile::TempDir, Repository) {
    let (dir, repo) = empty_repo();
    for i in 0..commit_count {
        commit_files(
            &repo,
            &sig_at(BASE_TIME + (i as i64) * 60),
            &format!("commit {i}: add changes"),
            &[("file.txt", &format!("content line {i}\nnext\n"))],
        );
    }
    (dir, repo)
}

/// Build a repo with a true rename so rename detection can be exercised.
pub fn repo_with_rename() -> (tempfile::TempDir, Repository) {
    let (dir, repo) = empty_repo();
    commit_files(
        &repo,
        &sig_at(BASE_TIME),
        "add a.txt",
        &[("a.txt", "content a")],
    );

    let next = BASE_TIME + 60;
    let workdir = repo.workdir().expect("workdir").to_path_buf();
    std::fs::remove_file(workdir.join("a.txt")).expect("remove a.txt");
    let mut index = repo.index().expect("index");
    index
        .remove_path(Path::new("a.txt"))
        .expect("remove from index");
    index.write().expect("write index");
    drop(index);
    commit_files(
        &repo,
        &sig_at(next),
        "rename a.txt to b.txt (same content)",
        &[("b.txt", "content a")],
    );
    commit_files(
        &repo,
        &sig_at(next + 60),
        "modify b.txt",
        &[("b.txt", "content a modified")],
    );
    (dir, repo)
}

/// Build a repo with two contributors across distinct commits.
#[allow(dead_code)]
pub fn repo_with_two_contributors() -> (tempfile::TempDir, Repository) {
    let (dir, repo) = empty_repo();
    let sig_a = Signature::now("Alice", "alice@example.com").expect("sig a");
    let sig_b = Signature::now("Bob", "bob@example.com").expect("sig b");
    commit_files(&repo, &sig_a, "alice one", &[("alice.txt", "a")]);
    commit_files(&repo, &sig_b, "bob one", &[("bob.txt", "b")]);
    commit_files(&repo, &sig_a, "alice two", &[("alice.txt", "a2")]);
    (dir, repo)
}

/// Build a repo exercising every status type plus "last status wins" semantics:
///   - a.txt:    Added -> Modified -> Deleted  => last = Deleted
///   - bin.dat:  Added (binary)               => Added
///   - moved1.txt: Added                      => Added
///   - moved2.txt: Renamed (from moved1) -> Modified => Modified
#[allow(dead_code)]
pub fn repo_with_statuses() -> (tempfile::TempDir, Repository) {
    let (dir, repo) = empty_repo();
    let workdir = repo.workdir().expect("workdir").to_path_buf();

    // commit 0 (BASE_TIME): add a.txt and a binary file
    std::fs::write(workdir.join("bin.dat"), [0xffu8, 0xfe, 0x00, b'\n']).expect("write bin.dat");
    {
        let mut index = repo.index().expect("index");
        index.add_path(Path::new("bin.dat")).expect("add bin.dat");
        index.write().expect("write index");
        drop(index);
    }
    commit_files(
        &repo,
        &sig_at(BASE_TIME),
        "commit 0: initial add",
        &[("a.txt", "v1\n")],
    );

    // commit 1 (BASE_TIME + 60): modify a.txt, add moved1.txt
    commit_files(
        &repo,
        &sig_at(BASE_TIME + 60),
        "commit 1: modify a.txt, add moved1.txt",
        &[("a.txt", "v1\nv2\n"), ("moved1.txt", "to be moved\n")],
    );

    // commit 2 (BASE_TIME + 120): delete a.txt
    {
        std::fs::remove_file(workdir.join("a.txt")).expect("remove a.txt");
        let mut index = repo.index().expect("index");
        index
            .remove_path(Path::new("a.txt"))
            .expect("remove from index");
        index.write().expect("write index");
        let tree_oid = index.write_tree().expect("write tree");
        drop(index);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(
            Some("HEAD"),
            &sig_at(BASE_TIME + 120),
            &sig_at(BASE_TIME + 120),
            "commit 2: delete a.txt",
            &tree,
            &[&parent],
        )
        .expect("commit delete");
    }

    // commit 3 (BASE_TIME + 180): rename moved1.txt -> moved2.txt
    {
        std::fs::remove_file(workdir.join("moved1.txt")).expect("remove moved1.txt");
        std::fs::write(workdir.join("moved2.txt"), "to be moved\n").expect("write moved2.txt");
        let mut index = repo.index().expect("index");
        index
            .remove_path(Path::new("moved1.txt"))
            .expect("remove moved1 from index");
        index
            .add_path(Path::new("moved2.txt"))
            .expect("add moved2 to index");
        index.write().expect("write index");
        let tree_oid = index.write_tree().expect("write tree");
        drop(index);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(
            Some("HEAD"),
            &sig_at(BASE_TIME + 180),
            &sig_at(BASE_TIME + 180),
            "commit 3: rename moved1.txt to moved2.txt",
            &tree,
            &[&parent],
        )
        .expect("commit rename");
    }

    // commit 4 (BASE_TIME + 240): modify moved2.txt
    commit_files(
        &repo,
        &sig_at(BASE_TIME + 240),
        "commit 4: modify moved2.txt",
        &[("moved2.txt", "to be moved\nmore content\n")],
    );

    (dir, repo)
}
