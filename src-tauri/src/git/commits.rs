use std::collections::BTreeMap;

use super::errors::Result;
use crate::models::commit::{Commit, CommitFile, FileStatus};
use crate::utils::dates;

const PROGRESS_EVERY: usize = 2000;

/// Entry point for walking the full commit history with per-commit diffs.
pub fn collect_history(
    repo: &git2::Repository,
    progress: &mut dyn FnMut(usize),
) -> Result<HistoryAnalysis> {
    // A brand-new repository has no commits yet; return an empty history.
    if repo.is_empty().unwrap_or(true) {
        return Ok(HistoryAnalysis::default());
    }

    let mut walk = repo.revwalk()?;
    walk.set_sorting(git2::Sort::TIME)?;
    walk.push_head()?;

    let mut analysis = HistoryAnalysis::default();
    let mut index = 0usize;

    for oid in walk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        index += 1;
        if index.is_multiple_of(PROGRESS_EVERY) {
            progress(index);
        }
        process_commit(repo, &commit, index as CommitSeq, &mut analysis)?;
    }

    analysis.commit_count = index;
    Ok(analysis)
}

#[derive(Default)]
pub struct HistoryAnalysis {
    pub commits: Vec<Commit>,
    pub commit_count: usize,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub first_commit_ts: Option<i64>,
    pub latest_commit_ts: Option<i64>,
    /// email -> (name)
    pub contributor_names: BTreeMap<String, String>,
    /// email -> number of commits
    pub contributor_commits: BTreeMap<String, usize>,
    /// email -> (additions, deletions)
    pub contributor_lines: BTreeMap<String, (u64, u64)>,
    /// "YYYY-MM" -> (commits, additions, deletions)
    pub month_buckets: BTreeMap<String, (usize, u64, u64)>,
    /// path -> aggregate
    pub files: BTreeMap<String, FileAggregate>,
}

/// Sequence number assigned while walking history (newest commit first), used
/// to break author-timestamp ties deterministically.
pub(crate) type CommitSeq = u32;

pub struct FileAggregate {
    pub change_count: u64,
    pub additions: u64,
    pub deletions: u64,
    /// Author timestamp of the most recent commit that touched the file.
    pub last_ts: i64,
    /// Walk sequence (newest commit = 1) of the commit that won `status`.
    pub last_seq: CommitSeq,
    /// Status of the most recent commit that touched the file.
    ///
    /// Rule (deterministic for a given history): the status of the file as of
    /// the commit with the greatest author timestamp. Equal timestamps are
    /// broken by the walk sequence (the commit Git visited first — the newest
    /// one — wins), so the verdict never depends on map iteration or
    /// revwalk tie order. "Last operation wins": a file Added then Modified is
    /// Modified; Added then Deleted is Deleted.
    pub status: Option<FileStatus>,
}

fn process_commit(
    repo: &git2::Repository,
    commit: &git2::Commit,
    seq: CommitSeq,
    analysis: &mut HistoryAnalysis,
) -> Result<()> {
    let author = commit.author();
    let author_ts = author.when().seconds();
    let committer_ts = commit.committer().when().seconds();

    let (files, additions, deletions) = diff_commit(repo, commit)?;

    let email = author.email().ok().unwrap_or("").to_string();
    let name = author.name().ok().unwrap_or("Unknown").to_string();

    let bucket = dates::month_key(author_ts);

    let counts = analysis
        .contributor_commits
        .entry(email.clone())
        .or_insert(0);
    *counts += 1;
    analysis
        .contributor_names
        .entry(email.clone())
        .or_insert(name.clone());

    let lines = analysis
        .contributor_lines
        .entry(email.clone())
        .or_insert((0, 0));
    lines.0 += additions as u64;
    lines.1 += deletions as u64;

    let month = analysis.month_buckets.entry(bucket).or_insert((0, 0, 0));
    month.0 += 1;
    month.1 += additions as u64;
    month.2 += deletions as u64;

    for file in &files {
        let entry = analysis
            .files
            .entry(file.path.clone())
            .or_insert(FileAggregate {
                change_count: 0,
                additions: 0,
                deletions: 0,
                last_ts: 0,
                last_seq: 0,
                status: None,
            });
        entry.change_count += 1;
        entry.additions += file.additions as u64;
        entry.deletions += file.deletions as u64;
        // Seq grows while walking (newest commit = 1), so the newer commit has
        // the smaller seq. A change wins when it is strictly newer by author
        // timestamp, or ties the timestamp and is newer in walk order — this
        // keeps the most-recent-commit verdict independent of BTreeMap order.
        if entry.status.is_none()
            || author_ts > entry.last_ts
            || (author_ts == entry.last_ts && seq < entry.last_seq)
        {
            entry.last_ts = author_ts;
            entry.last_seq = seq;
            entry.status = Some(file.status.clone());
        }
    }

    analysis.total_additions += additions as u64;
    analysis.total_deletions += deletions as u64;

    match analysis.first_commit_ts {
        Some(current) if current <= author_ts => {}
        _ => analysis.first_commit_ts = Some(author_ts),
    }
    match analysis.latest_commit_ts {
        Some(current) if current >= author_ts => {}
        _ => analysis.latest_commit_ts = Some(author_ts),
    }

    analysis.commits.push(Commit {
        hash: commit.id().to_string(),
        subject: commit.summary().ok().flatten().unwrap_or("").to_string(),
        body: commit.body().ok().flatten().map(|s| s.to_string()),
        author_name: name,
        author_email: email,
        author_date: dates::iso_datetime(author_ts),
        committer_name: commit
            .committer()
            .name()
            .ok()
            .unwrap_or("Unknown")
            .to_string(),
        committer_date: dates::iso_datetime(committer_ts),
        parents: commit.parent_ids().map(|p| p.to_string()).collect(),
        files,
        additions,
        deletions,
    });

    Ok(())
}

fn diff_commit(
    repo: &git2::Repository,
    commit: &git2::Commit,
) -> Result<(Vec<CommitFile>, u32, u32)> {
    let tree = commit.tree()?;

    // Diff against the first parent; for a root commit, against an empty tree.
    let parent_tree = match commit.parent(0) {
        Ok(parent) => Some(parent.tree()?),
        Err(_) => None,
    };

    let empty_tree = repo.find_tree(repo.treebuilder(None)?.write()?)?;
    let base = parent_tree.as_ref().unwrap_or(&empty_tree);

    let mut diff_ops = git2::DiffOptions::new();
    diff_ops.include_typechange(true);
    let mut diff = repo.diff_tree_to_tree(Some(base), Some(&tree), Some(&mut diff_ops))?;

    let mut find_ops = git2::DiffFindOptions::new();
    find_ops.renames(true);
    find_ops.renames_from_rewrites(true);
    diff.find_similar(Some(&mut find_ops))?;

    let mut files = Vec::new();
    let mut total_additions = 0u32;
    let mut total_deletions = 0u32;

    for (idx, delta) in diff.deltas().enumerate() {
        let new_path = delta
            .new_file()
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let old_path = delta
            .old_file()
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let status = map_status(delta.status());

        let (additions, deletions) = match git2::Patch::from_diff(&diff, idx) {
            Ok(Some(patch)) => match patch.line_stats() {
                Ok((_context, a, d)) => {
                    let a = a.min(u32::MAX as usize) as u32;
                    let d = d.min(u32::MAX as usize) as u32;
                    total_additions = total_additions.saturating_add(a);
                    total_deletions = total_deletions.saturating_add(d);
                    (a, d)
                }
                Err(_) => (0, 0),
            },
            _ => (0, 0),
        };

        // For renames the file lives on under its new path. For deletions the
        // delta has no new file, so fall back to the old path — otherwise every
        // deleted file would be recorded under an empty path.
        let path = if !new_path.is_empty() {
            new_path
        } else if !old_path.is_empty() {
            old_path.clone()
        } else {
            String::new()
        };

        files.push(CommitFile {
            previous_path: if status == FileStatus::Renamed {
                Some(old_path)
            } else {
                None
            },
            path,
            status,
            additions,
            deletions,
        });
    }

    Ok((files, total_additions, total_deletions))
}

fn map_status(status: git2::Delta) -> FileStatus {
    match status {
        git2::Delta::Added => FileStatus::Added,
        git2::Delta::Deleted => FileStatus::Deleted,
        git2::Delta::Renamed | git2::Delta::Copied => FileStatus::Renamed,
        git2::Delta::Modified | git2::Delta::Typechange => FileStatus::Modified,
        _ => FileStatus::Modified,
    }
}
