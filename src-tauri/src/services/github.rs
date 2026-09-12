use url::Url;

use crate::git::errors::{AnalysisError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubTarget {
    pub owner: String,
    pub repo: String,
}

/// Validate a GitHub repository URL and extract owner and repository name.
pub fn parse_github_url(url: &str) -> Result<GithubTarget> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AnalysisError::InvalidGithubUrl(
            "The URL is empty.".to_string(),
        ));
    }

    let parsed = Url::parse(trimmed).map_err(|e| {
        AnalysisError::InvalidGithubUrl(format!("The URL could not be parsed: {e}"))
    })?;

    if parsed.scheme() != "https" {
        return Err(AnalysisError::InvalidGithubUrl(
            "The URL must use https (http URLs are not allowed for cloning).".to_string(),
        ));
    }

    let host = parsed.host_str().unwrap_or("");
    if host != "github.com" && host != "www.github.com" {
        return Err(AnalysisError::InvalidGithubUrl(format!(
            "Only github.com repositories are supported (got host '{host}')."
        )));
    }

    let segments: Vec<String> = parsed
        .path_segments()
        .map(|s| s.filter(|x| !x.is_empty()).map(|x| x.to_string()).collect())
        .unwrap_or_default();

    if segments.is_empty() {
        return Err(AnalysisError::InvalidGithubUrl(
            "No repository was specified.".to_string(),
        ));
    }
    if segments.len() != 2 {
        return Err(AnalysisError::InvalidGithubUrl(
            format!(
                "Expected a URL of the form https://github.com/owner/repository (got {} path segments).",
                segments.len()
            ),
        ));
    }

    let owner = sanitize_component(&segments[0], "owner")?;
    let repo = sanitize_component(segments[1].trim_end_matches(".git"), "repository")?;

    Ok(GithubTarget { owner, repo })
}

fn sanitize_component(value: &str, what: &str) -> Result<String> {
    if value.is_empty() {
        return Err(AnalysisError::InvalidGithubUrl(format!(
            "The {what} name is empty."
        )));
    }
    if value == "." || value == ".." {
        return Err(AnalysisError::InvalidGithubUrl(format!(
            "The {what} name '{value}' is not allowed."
        )));
    }
    let valid = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if !valid {
        return Err(AnalysisError::InvalidGithubUrl(format!(
            "The {what} name '{value}' contains unsupported characters."
        )));
    }
    Ok(value.to_string())
}

/// Estimated clone destination for a target under a managed base directory.
pub fn clone_destination(target: &GithubTarget, base_dir: &std::path::Path) -> std::path::PathBuf {
    // GitHub owner/repo names are case-insensitive; the cache key must be
    // normalised to avoid duplicate clones for the same repository written
    // with different casing.
    base_dir.join(format!(
        "github_{}__{}",
        target.owner.to_lowercase(),
        target.repo.to_lowercase()
    ))
}

/// Clone a repository from the target into `base_dir` if not already present.
/// Returns the destination path.
pub fn clone_into(target: &GithubTarget, base_dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let dest = clone_destination(target, base_dir);
    if dest.join(".git").exists() {
        return Ok(dest);
    }

    if !base_dir.exists() {
        std::fs::create_dir_all(base_dir).map_err(|e| {
            AnalysisError::CloneFailed(format!(
                "Could not create the managed clone directory {}: {e}",
                base_dir.display()
            ))
        })?;
    }

    let clone_url = format!("https://github.com/{}/{}", target.owner, target.repo);
    git2::build::RepoBuilder::new()
        .clone(&clone_url, &dest)
        .map_err(|e| {
            AnalysisError::CloneFailed(format!("Git could not clone {clone_url}: {}", e.message()))
        })?;

    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_github_url() {
        let t = parse_github_url("https://github.com/owner/repository").unwrap();
        assert_eq!(
            t,
            GithubTarget {
                owner: "owner".into(),
                repo: "repository".into()
            }
        );
    }

    #[test]
    fn valid_with_git_suffix() {
        let t = parse_github_url("https://github.com/owner/repository.git").unwrap();
        assert_eq!(
            t,
            GithubTarget {
                owner: "owner".into(),
                repo: "repository".into()
            }
        );
    }

    #[test]
    fn www_host_allowed() {
        assert!(parse_github_url("https://www.github.com/foo/bar").is_ok());
    }

    #[test]
    fn rejects_non_github() {
        assert!(parse_github_url("https://gitlab.com/foo/bar").is_err());
        assert!(parse_github_url("https://github.com/").is_err());
    }

    #[test]
    fn rejects_bad_scheme() {
        assert!(parse_github_url("ftp://github.com/foo/bar").is_err());
    }

    #[test]
    fn rejects_http() {
        assert!(parse_github_url("http://github.com/foo/bar").is_err());
        assert!(parse_github_url("http://www.github.com/foo/bar").is_err());
    }

    #[test]
    fn accepts_https() {
        assert!(parse_github_url("https://github.com/foo/bar").is_ok());
        assert!(parse_github_url("https://www.github.com/foo/bar").is_ok());
    }

    #[test]
    fn rejects_path_traversal_attempts() {
        // ".." is not a valid GitHub owner/repo and must never resolve to a
        // path outside the managed clone directory.
        assert!(parse_github_url("https://github.com/../bar").is_err());
        assert!(parse_github_url("https://github.com/foo/..").is_err());
        assert!(parse_github_url("https://github.com/foo/bar/..").is_err());
        assert!(parse_github_url("https://github.com/%2e%2e/bar").is_err());
    }

    #[test]
    fn rejects_malicious_hostnames() {
        assert!(parse_github_url("https://github.com.evil.com/foo/bar").is_err());
        assert!(parse_github_url("https://evil-github.com/foo/bar").is_err());
        assert!(parse_github_url("https://github.com:8080@evil.com/foo/bar").is_err());
        assert!(parse_github_url("https://123.123.123.123/foo/bar").is_err());
    }

    #[test]
    fn rejects_too_many_segments() {
        assert!(parse_github_url("https://github.com/foo/bar/tree/main").is_err());
    }
}
