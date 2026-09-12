use crate::git::errors::{AnalysisError, Result};
use crate::models::analysis::AnalysisData;
use crate::utils::dates;

#[derive(serde::Serialize)]
struct Report {
    generated_at: String,
    gitviewr_version: &'static str,
    analysis: AnalysisData,
}

pub fn write(data: &AnalysisData, path: &str) -> Result<()> {
    let report = Report {
        generated_at: dates::now_iso(),
        gitviewr_version: env!("CARGO_PKG_VERSION"),
        analysis: data.clone(),
    };
    let json = serde_json::to_string_pretty(&report).map_err(|e| {
        AnalysisError::ExportFailed(format!("Could not serialize analysis data: {e}"))
    })?;
    std::fs::write(path, json)
        .map_err(|e| AnalysisError::ExportFailed(format!("Could not write the JSON file: {e}")))?;
    Ok(())
}
