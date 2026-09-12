//! Integration tests for the HTML/JSON/PDF exporters.

mod common;

use gitviewr_lib::models::analysis::AnalysisData;
use gitviewr_lib::services::exporter;

fn sample_data() -> AnalysisData {
    let (dir, _repo) = common::repo_with_history(3);
    common::analyze(&dir.path().to_string_lossy()).expect("analyze test repo")
}

#[test]
fn json_export_is_valid_and_contains_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("report.json");
    exporter::json::write(&sample_data(), &target.to_string_lossy()).expect("json export");
    let text = std::fs::read_to_string(&target).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert!(parsed["generated_at"].as_str().is_some());
    assert!(
        parsed["analysis"]["stats"]["commit_count"]
            .as_u64()
            .unwrap()
            >= 3
    );
    assert!(parsed["analysis"]["commits"].as_array().unwrap().len() >= 3);
    assert!(text.contains("\"commits\""));
}

#[test]
fn html_export_contains_repo_info() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("report.html");
    exporter::html::write(&sample_data(), &target.to_string_lossy()).expect("html export");
    let text = std::fs::read_to_string(&target).unwrap();
    assert!(text.starts_with("<!DOCTYPE html>"));
    assert!(text.contains("GitViewr Report"));
    assert!(text.contains("Commits Over Time"));
    assert!(text.contains("Contributors"));
}

#[test]
fn html_export_escapes_unsafe_content() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("escaped.html");
    let (temp, repo) = common::empty_repo();
    let sig = common::signature();
    common::commit_files(
        &repo,
        &sig,
        "message with <script>alert(1)</script>",
        &[("evil.txt", "<b>content</b>")],
    );
    let data = common::analyze(&temp.path().to_string_lossy()).expect("analyze");
    exporter::html::write(&data, &target.to_string_lossy()).expect("html export");
    let text = std::fs::read_to_string(&target).unwrap();
    assert!(!text.contains("<script>alert(1)</script>"));
    assert!(text.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn pdf_export_produces_valid_document() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("report.pdf");
    exporter::pdf::write(&sample_data(), &target.to_string_lossy()).expect("pdf export");
    let bytes = std::fs::read(&target).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    let text = String::from_utf8_lossy(&bytes);
    // Verify the xref table points at the last object (EOF parses).
    assert!(text.contains("startxref"));
    assert!(text.contains("%%EOF"));
    assert!(text.contains("/Type /Pages"));
}

#[test]
fn exporters_return_error_for_bad_path() {
    let data = sample_data();
    let bad_path = std::path::Path::new("/definitely/not/writable/dir/report.json");
    assert!(exporter::json::write(&data, &bad_path.to_string_lossy()).is_err());
}
