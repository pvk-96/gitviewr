use crate::git::commits::collect_history;
use crate::git::errors::Result;
use crate::git::repo::RepoInfo;
use crate::models::analysis::{
    AnalysisData, Analytics, ContributorActivity, ContributorSummary, FileHotspot, FileStat,
    GrowthPoint, RepositoryMetadata, RepositoryStats, TimeBucket,
};
use crate::models::repository::{Repository, SourceType};
use crate::services::github;
use crate::utils::dates;
use std::cmp::Reverse;

/// Analyzes repositories and builds structured `AnalysisData`.
pub struct Analyzer;

impl Analyzer {
    /// Analyze a local repository at the given path.
    pub fn analyze_local(path: &str) -> Result<AnalysisData> {
        let info = RepoInfo::open(path)?;
        build(&info, path, SourceType::Local, &mut None)
    }

    /// Analyze local with progress reporting.
    pub fn analyze_local_with_progress(
        path: &str,
        progress: &mut dyn FnMut(&str),
    ) -> Result<AnalysisData> {
        let info = RepoInfo::open(path)?;
        build(&info, path, SourceType::Local, &mut Some(progress))
    }

    /// Clone (if necessary) and analyze a public GitHub repository URL.
    pub fn analyze_github(url: &str, clone_base: &std::path::Path) -> Result<AnalysisData> {
        let target = github::parse_github_url(url)?;
        let dest = github::clone_into(&target, clone_base)?;
        let info = RepoInfo::open(&dest.to_string_lossy())?;
        let mut data = build(&info, url, SourceType::GitHub, &mut None)?;
        data.metadata.name = target.repo.clone();
        data.repository.name = target.repo;
        Ok(data)
    }
}

fn emit_progress(progress: &mut Option<&mut dyn FnMut(&str)>, msg: &str) {
    if let Some(cb) = progress.as_mut() {
        cb(msg);
    }
}

fn build(
    info: &RepoInfo,
    display_path: &str,
    source_type: SourceType,
    progress: &mut Option<&mut dyn FnMut(&str)>,
) -> Result<AnalysisData> {
    emit_progress(progress, "Collecting commit history…");
    let mut on_commits = |count: usize| {
        emit_progress(
            progress,
            &format!("Collecting commit history… ({count} commits)"),
        );
    };
    let history = collect_history(&info.repo, &mut on_commits)?;

    emit_progress(progress, "Building statistics…");
    let repository = Repository {
        name: info.name.clone(),
        path: display_path.to_string(),
        source_type: source_type.clone(),
    };

    let mut contributor_rows: Vec<(String, String, usize, u64, u64)> = history
        .contributor_commits
        .iter()
        .map(|(email, &commits)| {
            let name = history
                .contributor_names
                .get(email)
                .cloned()
                .unwrap_or_else(|| email.clone());
            let (additions, deletions) = history
                .contributor_lines
                .get(email)
                .copied()
                .unwrap_or((0, 0));
            (email.clone(), name, commits, additions, deletions)
        })
        .collect();
    contributor_rows.sort_by_key(|row| Reverse(row.2));

    let contributors: Vec<ContributorSummary> = contributor_rows
        .iter()
        .map(|(email, name, commits, _, _)| ContributorSummary {
            name: name.clone(),
            email: email.clone(),
            commits: *commits,
        })
        .collect();

    let contributor_activity: Vec<ContributorActivity> = contributor_rows
        .iter()
        .map(
            |(email, name, commits, additions, deletions)| ContributorActivity {
                name: name.clone(),
                email: email.clone(),
                commits: *commits,
                additions: *additions,
                deletions: *deletions,
            },
        )
        .collect();

    let metadata = RepositoryMetadata {
        name: info.name.clone(),
        path: info.repo_root.display().to_string(),
        source_type: source_type.clone(),
        default_branch: info.default_branch.clone(),
        remote_url: info.remote_url.clone(),
        contributors,
    };

    let net_change = history.total_additions as i64 - history.total_deletions as i64;

    let stats = RepositoryStats {
        branch: info.current_branch.clone(),
        commit_count: history.commit_count,
        branch_count: info.branch_count,
        tracked_files: info.tracked_files,
        contributor_count: contributor_activity.len(),
        first_commit_date: history.first_commit_ts.map(dates::iso_datetime),
        latest_commit_date: history.latest_commit_ts.map(dates::iso_datetime),
        total_additions: history.total_additions,
        total_deletions: history.total_deletions,
        net_change,
    };

    // `status` carries the status of the most recent commit that touched the
    // file (see `FileAggregate.status` in git/commits.rs for the rule).
    let mut file_rows: Vec<(String, FileStat)> = history
        .files
        .iter()
        .map(|(path, agg)| {
            let stat = FileStat {
                path: path.clone(),
                additions: agg.additions,
                deletions: agg.deletions,
                change_count: agg.change_count,
                last_committed: Some(dates::iso_date(agg.last_ts)),
                status: agg.status.clone(),
            };
            (path.clone(), stat)
        })
        .collect();
    file_rows.sort_by(|a, b| {
        b.1.change_count
            .cmp(&a.1.change_count)
            .then_with(|| a.0.cmp(&b.0))
    });
    let file_stats: Vec<FileStat> = file_rows.into_iter().map(|(_, s)| s).collect();

    // Monthly buckets, ascending by "YYYY-MM" (BTreeMap is key-sorted).
    let time_buckets: Vec<TimeBucket> = history
        .month_buckets
        .iter()
        .map(|(period, (commits, additions, deletions))| TimeBucket {
            period: period.clone(),
            commits: *commits,
            additions: *additions,
            deletions: *deletions,
        })
        .collect();

    let mut running_net = 0i64;
    let timeline: Vec<GrowthPoint> = time_buckets
        .iter()
        .map(|bucket| {
            running_net += bucket.additions as i64 - bucket.deletions as i64;
            GrowthPoint {
                period: bucket.period.clone(),
                net: running_net,
            }
        })
        .collect();

    let mut file_hotspots: Vec<FileHotspot> = history
        .files
        .iter()
        .map(|(path, agg)| FileHotspot {
            path: path.clone(),
            changes: agg.change_count,
            additions: agg.additions,
            deletions: agg.deletions,
            last_changed: Some(dates::iso_date(agg.last_ts)),
        })
        .collect();
    file_hotspots.sort_by(|a, b| b.changes.cmp(&a.changes).then_with(|| a.path.cmp(&b.path)));

    let analytics = Analytics {
        total_commits: history.commit_count,
        total_additions: history.total_additions,
        total_deletions: history.total_deletions,
        net_change,
        contributors: contributor_activity,
        commits_over_time: time_buckets.clone(),
        churn_over_time: time_buckets,
        file_hotspots,
        timeline,
    };

    emit_progress(progress, "Done.");
    Ok(AnalysisData {
        repository,
        metadata,
        stats,
        commits: history.commits,
        file_stats,
        analytics,
    })
}
