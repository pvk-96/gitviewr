use serde::{Deserialize, Serialize};

use super::repository::SourceType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentRepository {
    pub name: String,
    pub path: String,
    pub source_type: SourceType,
    pub last_opened: String,
}
