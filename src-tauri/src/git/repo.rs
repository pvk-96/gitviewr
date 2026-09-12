use std::path::{Path, PathBuf};

use super::errors::{AnalysisError, Result};

/// Lightweight handle around an opened Git repository plus basic info that
/// does not require walking the full history.
pub struct RepoInfo {
    pub repo: git2::Repository,
    pub repo_root: PathBuf,
    pub name: String,
    pub default_branch: Option<String>,
    pub current_branch: Option<String>,
    pub branch_count: usize,
    pub tracked_files: usize,
    pub remote_url: Option<String>,
}

impl RepoInfo {
    pub fn open(path: &str) -> Result<Self> {
        let p = Path::new(path);
        let metadata = std::fs::metadata(p).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => AnalysisError::InvalidPath(path.to_string()),
            std::io::ErrorKind::PermissionDenied => AnalysisError::Inaccessible(path.to_string()),
            _ => AnalysisError::Inaccessible(path.to_string()),
        })?;

        if !metadata.is_dir() {
            return Err(AnalysisError::NotGitRepository(path.to_string()));
        }

        let repo = git2::Repository::open(p).map_err(|e| match e.code() {
            git2::ErrorCode::NotFound => AnalysisError::NotGitRepository(path.to_string()),
            _ => AnalysisError::ReadFailed(format!("{path}: {}", e.message())),
        })?;

        let repo_root = repo
            .workdir()
            .map(|w| w.to_path_buf())
            .unwrap_or_else(|| p.join(".git").to_path_buf());

        let name = repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        Ok(RepoInfo {
            current_branch: head_shorthand(&repo),
            default_branch: default_branch(&repo),
            branch_count: count_local_branches(&repo),
            tracked_files: count_tracked_files(&repo),
            remote_url: origin_url(&repo),
            repo,
            repo_root,
            name,
        })
    }
}

fn head_shorthand(repo: &git2::Repository) -> Option<String> {
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().ok().map(|s| s.to_string()))
}

fn default_branch(repo: &git2::Repository) -> Option<String> {
    // Prefer origin/HEAD when available (common in cloned repos).
    if let Ok(reference) = repo.find_reference("refs/remotes/origin/HEAD") {
        if let Ok(resolved) = reference.resolve() {
            if let Ok(name) = resolved.name() {
                let short = name.strip_prefix("refs/remotes/").map(|s| s.to_string());
                if let Some(short) = short {
                    // strip the leading "origin/" to leave just the branch name
                    if let Some(branch) = short.split_once('/').map(|(_, b)| b.to_string()) {
                        return Some(branch);
                    }
                }
            }
        }
    }
    head_shorthand(repo)
}

fn count_local_branches(repo: &git2::Repository) -> usize {
    repo.branches(Some(git2::BranchType::Local))
        .map(|branches| branches.filter(|b| b.is_ok()).count())
        .unwrap_or(0)
}

fn count_tracked_files(repo: &git2::Repository) -> usize {
    let tree = match repo.head().and_then(|h| h.peel_to_tree()) {
        Ok(t) => t,
        Err(_) => return 0,
    };
    let mut count = 0usize;
    let _ = tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            count += 1;
        }
        0
    });
    count
}

fn origin_url(repo: &git2::Repository) -> Option<String> {
    repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().ok().map(|u| u.to_string()))
}
