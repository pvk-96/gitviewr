use crate::git::errors::{AnalysisError, Result};
use crate::models::analysis::AnalysisData;
use crate::services::exporter;

#[tauri::command]
pub async fn export_report(
    format: String,
    target_path: String,
    analysis_json: String,
) -> std::result::Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<()> {
        let data: AnalysisData = serde_json::from_str(&analysis_json).map_err(|e| {
            AnalysisError::ExportFailed(format!(
                "Could not parse the analysis data sent to the backend: {e}"
            ))
        })?;

        match format.as_str() {
            "html" => exporter::html::write(&data, &target_path),
            "json" => exporter::json::write(&data, &target_path),
            "pdf" => exporter::pdf::write(&data, &target_path),
            other => Err(AnalysisError::ExportFailed(format!(
                "Unknown export format '{other}'. Expected 'html', 'json' or 'pdf'."
            ))),
        }
    })
    .await
    .map_err(|e| AnalysisError::ExportFailed(e.to_string()).to_string())?
    .map_err(|e| e.to_string())
}
