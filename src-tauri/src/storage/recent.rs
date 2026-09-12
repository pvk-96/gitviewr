use crate::models::recent::RecentRepository;
use crate::models::repository::SourceType;
use std::path::PathBuf;

/// Maximum number of recent repositories kept in persistent storage.
pub const MAX_RECENT: usize = 3;

pub struct RecentStore {
    path: PathBuf,
}

impl RecentStore {
    pub fn new(path: PathBuf) -> Self {
        RecentStore { path }
    }

    /// Read the stored list of recent repositories. Missing/corrupt files
    /// fall back to an empty list rather than failing.
    pub fn read(&self) -> Vec<RecentRepository> {
        let Ok(bytes) = std::fs::read(&self.path) else {
            return Vec::new();
        };
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    /// Record a recently opened repository, moving it to the front and
    /// keeping at most `MAX_RECENT` entries. Duplicates (same path + type)
    /// are removed so the repository appears only once.
    pub fn record(&self, name: String, path: String, source_type: SourceType, last_opened: String) {
        let mut entries = self.read();

        entries.retain(|e| e.path != path || e.source_type != source_type);

        let newest = RecentRepository {
            name,
            path,
            source_type,
            last_opened,
        };
        entries.insert(0, newest);
        entries.truncate(MAX_RECENT);

        let _ = self.write(&entries);
    }

    fn write(&self, entries: &[RecentRepository]) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(entries)?;
        std::fs::write(&self.path, json)
    }
}

pub fn app_data_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> std::io::Result<PathBuf> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map_err(|e| std::io::Error::other(e.to_string()))
}

pub fn recent_store_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> std::io::Result<PathBuf> {
    Ok(app_data_dir(app)?.join("recent.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_store() -> (tempfile::TempDir, RecentStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = RecentStore::new(dir.path().join("recent.json"));
        (dir, store)
    }

    #[test]
    fn empty_by_default() {
        let (_dir, store) = new_store();
        assert!(store.read().is_empty());
    }

    #[test]
    fn persists_across_store_instances() {
        let (_dir, store) = new_store();
        store.record("A".into(), "A".into(), SourceType::Local, "t".into());
        let reread = store.read();
        assert_eq!(reread.len(), 1);
        assert_eq!(reread[0].name, "A");
    }

    #[test]
    fn enforces_three_item_limit() {
        let (_dir, store) = new_store();
        for name in ["A", "B", "C", "D", "E"] {
            store.record(name.into(), name.into(), SourceType::Local, "t".into());
        }
        let entries = store.read();
        assert_eq!(entries.len(), MAX_RECENT);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["E", "D", "C"]);
    }

    #[test]
    fn reopens_move_to_front_without_duplicates() {
        let (_dir, store) = new_store();
        for name in ["A", "B", "C"] {
            store.record(name.into(), name.into(), SourceType::Local, "t".into());
        }
        // Re-open C again.
        store.record("C".into(), "C".into(), SourceType::Local, "t2".into());
        let entries = store.read();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["C", "B", "A"]);
        assert_eq!(entries.len(), 3, "no duplicates");
    }

    #[test]
    fn corrupt_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("recent.json");
        std::fs::write(&path, "not json{{").unwrap();
        let store = RecentStore::new(path);
        assert!(store.read().is_empty());
    }
}
