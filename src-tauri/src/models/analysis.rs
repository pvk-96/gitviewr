use serde::{Deserialize, Serialize};

use super::commit::{Commit, FileStatus};
use super::repository::{Repository, SourceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorSummary {
    pub name: String,
    pub email: String,
    pub commits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub name: String,
    pub path: String,
    pub source_type: SourceType,
    pub default_branch: Option<String>,
    pub remote_url: Option<String>,
    pub contributors: Vec<ContributorSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub branch: Option<String>,
    pub commit_count: usize,
    pub branch_count: usize,
    pub tracked_files: usize,
    pub contributor_count: usize,
    pub first_commit_date: Option<String>,
    pub latest_commit_date: Option<String>,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub net_change: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
    pub change_count: u64,
    pub last_committed: Option<String>,
    pub status: Option<FileStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorActivity {
    pub name: String,
    pub email: String,
    pub commits: usize,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBucket {
    pub period: String,
    pub commits: usize,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHotspot {
    pub path: String,
    pub changes: u64,
    pub additions: u64,
    pub deletions: u64,
    pub last_changed: Option<String>,
}

/// A point on the repository's cumulative growth curve: the running net line
/// count (additions minus deletions) up to and including `period`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthPoint {
    pub period: String,
    pub net: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics {
    pub total_commits: usize,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub net_change: i64,
    pub contributors: Vec<ContributorActivity>,
    /// Monthly commit counts ("YYYY-MM" -> commits).
    pub commits_over_time: Vec<TimeBucket>,
    /// Monthly line churn ("YYYY-MM" -> additions/deletions).
    pub churn_over_time: Vec<TimeBucket>,
    pub file_hotspots: Vec<FileHotspot>,
    /// Cumulative net lines added over the repository's life.
    pub timeline: Vec<GrowthPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisData {
    pub repository: Repository,
    pub metadata: RepositoryMetadata,
    pub stats: RepositoryStats,
    pub commits: Vec<Commit>,
    pub file_stats: Vec<FileStat>,
    pub analytics: Analytics,
}
