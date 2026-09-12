use std::fmt;

#[derive(Debug)]
pub enum AnalysisError {
    InvalidPath(String),
    NotGitRepository(String),
    Inaccessible(String),
    GitUnavailable(String),
    ReadFailed(String),
    AnalysisFailed(String),
    CloneFailed(String),
    InvalidGithubUrl(String),
    ExportFailed(String),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::InvalidPath(p) => write!(
                f,
                "Unable to open repository.\n\nThe selected path does not exist.\nPath: {p}"
            ),
            AnalysisError::NotGitRepository(p) => write!(
                f,
                "Unable to open repository.\n\nThe selected directory does not appear to contain a Git repository.\nPath: {p}"
            ),
            AnalysisError::Inaccessible(p) => write!(
                f,
                "Unable to open repository.\n\nThe selected folder could not be accessed.\nPath: {p}\n\nCheck that you have permission to read this folder."
            ),
            AnalysisError::GitUnavailable(msg) => write!(
                f,
                "Git functionality is unavailable.\n\n{msg}\n\nGitViewr uses libgit2 for repository access. If this persists, please report the issue."
            ),
            AnalysisError::ReadFailed(msg) => write!(
                f,
                "The repository could not be read.\n\n{msg}"
            ),
            AnalysisError::AnalysisFailed(msg) => write!(
                f,
                "Repository analysis failed.\n\n{msg}"
            ),
            AnalysisError::CloneFailed(msg) => write!(
                f,
                "Unable to clone repository.\n\nPlease check the GitHub URL and your network connection.\n\n{msg}"
            ),
            AnalysisError::InvalidGithubUrl(msg) => write!(
                f,
                "Invalid GitHub repository URL.\n\nUse a URL like https://github.com/owner/repository\n\n{msg}"
            ),
            AnalysisError::ExportFailed(msg) => write!(
                f,
                "Export failed.\n\n{msg}"
            ),
        }
    }
}

impl std::error::Error for AnalysisError {}

impl From<git2::Error> for AnalysisError {
    fn from(err: git2::Error) -> Self {
        match err.code() {
            git2::ErrorCode::NotFound => AnalysisError::NotGitRepository(err.message().to_string()),
            _ => AnalysisError::ReadFailed(err.message().to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, AnalysisError>;
