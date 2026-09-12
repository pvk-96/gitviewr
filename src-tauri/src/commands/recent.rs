use tauri::AppHandle;

use crate::models::recent::RecentRepository;
use crate::storage::recent::{self, RecentStore};

#[tauri::command]
pub fn get_recent_repositories<R: tauri::Runtime>(
    app: AppHandle<R>,
) -> std::result::Result<Vec<RecentRepository>, String> {
    let path = recent::recent_store_path(&app).map_err(|e| e.to_string())?;
    Ok(RecentStore::new(path).read())
}
