use std::path::PathBuf;

use tauri::Emitter;

use super::ProgressEvent;
use crate::git::errors::{AnalysisError, Result};
use crate::models::analysis::AnalysisData;
use crate::services::analyzer::Analyzer;
use crate::storage::recent::{self, RecentStore};
use crate::utils::dates;

fn report_progress<R: tauri::Runtime>(app: &tauri::AppHandle<R>, phase: &str) {
    let _ = app.emit(
        "analysis-progress",
        ProgressEvent {
            phase: phase.to_string(),
        },
    );
}

fn store_recent<R: tauri::Runtime>(app: &tauri::AppHandle<R>, data: &AnalysisData) {
    if let Ok(path) = recent::recent_store_path(app) {
        let store = RecentStore::new(path);
        store.record(
            data.repository.name.clone(),
            data.repository.path.clone(),
            data.repository.source_type.clone(),
            dates::now_iso(),
        );
    }
}

#[tauri::command]
pub async fn analyze_local_repository<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
) -> std::result::Result<AnalysisData, String> {
    let normalized = path.trim().to_string();
    let backend = app.clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<AnalysisData> {
        let mut progress = |phase: &str| report_progress(&backend, phase);
        let data = Analyzer::analyze_local_with_progress(&normalized, &mut progress)?;
        store_recent(&backend, &data);
        Ok(data)
    })
    .await
    .map_err(|e| AnalysisError::AnalysisFailed(e.to_string()).to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn analyze_github_repository<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
) -> std::result::Result<AnalysisData, String> {
    let backend = app.clone();
    let normalized_url = url.trim().to_string();

    tauri::async_runtime::spawn_blocking(move || -> Result<AnalysisData> {
        report_progress(&backend, "Validating URL…");

        let data = {
            let clone_base: PathBuf = recent::app_data_dir(&backend)
                .map_err(|e| {
                    AnalysisError::AnalysisFailed(format!(
                        "Could not determine app data directory: {e}"
                    ))
                })?
                .join("repositories");

            Analyzer::analyze_github(&normalized_url, &clone_base)?
        };

        store_recent(&backend, &data);
        Ok(data)
    })
    .await
    .map_err(|e| AnalysisError::AnalysisFailed(e.to_string()).to_string())?
    .map_err(|e| e.to_string())
}
