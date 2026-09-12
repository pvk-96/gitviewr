//! Exercises the real Tauri command layer through the full IPC path, exactly as
//! the frontend invokes it, using a mocked `tauri` app with a `MockRuntime`.

mod common;

use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::INVOKE_KEY;
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindowBuilder};

fn build_handler_app() -> tauri::App<tauri::test::MockRuntime> {
    tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![
            gitviewr_lib::commands::repository::analyze_local_repository,
            gitviewr_lib::commands::repository::analyze_github_repository,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app")
}

fn build_full_handler_app() -> tauri::App<tauri::test::MockRuntime> {
    tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![
            gitviewr_lib::commands::export::export_report,
            gitviewr_lib::commands::recent::get_recent_repositories,
            gitviewr_lib::commands::system::open_external_url,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app")
}

fn invoke(
    _app: &tauri::App<tauri::test::MockRuntime>,
    window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    let request = InvokeRequest {
        cmd: cmd.to_string(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "tauri://localhost".parse().unwrap(),
        body: InvokeBody::Json(body),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    };
    let response = tauri::test::get_ipc_response(&window, request)?;
    Ok(response.deserialize().expect("json response"))
}

#[test]
fn analyze_local_repository_command_reports_analysis() {
    let (dir, _repo) = common::repo_with_history(3);
    let app = build_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    let res = invoke(
        &app,
        &window,
        "analyze_local_repository",
        serde_json::json!({ "path": dir.path().to_string_lossy().to_string() }),
    );
    let data = res.expect("command succeeds");
    assert_eq!(data["stats"]["commit_count"], 3);
    assert_eq!(data["stats"]["total_additions"], 4);
    assert_eq!(data["commits"].as_array().map(|a| a.len()), Some(3));

    window.close().expect("close");
}

#[test]
fn analyze_local_repository_command_errors_on_non_git() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("x.txt"), "hi").unwrap();
    let app = build_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    let res = invoke(
        &app,
        &window,
        "analyze_local_repository",
        serde_json::json!({ "path": dir.path().to_string_lossy().to_string() }),
    );
    let err = res.expect_err("command fails for non-git folder");
    let message = err.as_str().unwrap_or_default();
    assert!(
        message.contains("Git repository"),
        "message should guide the user, got {message}"
    );

    window.close().expect("close");
}

#[test]
fn analyze_local_repository_command_records_recent() {
    // Sanity: successful command must not panic while recording the recent entry.
    let (dir, _repo) = common::repo_with_history(2);
    let app = build_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    invoke(
        &app,
        &window,
        "analyze_local_repository",
        serde_json::json!({ "path": dir.path().to_string_lossy().to_string() }),
    )
    .expect("analyze ok");

    // Recent-store persistence (limit, dedupe, move-to-front) is covered by the
    // storage module's own unit tests, so here we only assert a file exists.
    let recent_path = app
        .path()
        .app_data_dir()
        .expect("app data dir")
        .join("recent.json");
    assert!(
        recent_path.exists()
            || std::fs::read_to_string(&recent_path)
                .unwrap_or_default()
                .is_empty(),
        "analysis should have written a recent entry"
    );

    window.close().expect("close");
}

#[test]
fn export_report_command_writes_json_file() {
    let (dir, _repo) = common::repo_with_history(2);
    let data = common::analyze(dir.path().to_str().unwrap()).unwrap();
    let analysis_json = serde_json::to_string(&data).unwrap();
    let target = dir.path().join("report.json");

    let app = build_full_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    invoke(
        &app,
        &window,
        "export_report",
        serde_json::json!({
            "format": "json",
            "targetPath": target.to_string_lossy().to_string(),
            "analysisJson": analysis_json,
        }),
    )
    .expect("export succeeds");

    let written = std::fs::read_to_string(&target).expect("report file written");
    let parsed: serde_json::Value = serde_json::from_str(&written).expect("report is valid JSON");
    assert!(
        parsed["analysis"]["stats"]["commit_count"] == 2,
        "report must carry the analysis payload, got: {written}"
    );

    window.close().expect("close");
}

#[test]
fn export_report_command_rejects_unknown_format() {
    let (dir, _repo) = common::repo_with_history(1);
    let data = common::analyze(dir.path().to_str().unwrap()).unwrap();
    let analysis_json = serde_json::to_string(&data).unwrap();
    let target = dir.path().join("report.xyz");

    let app = build_full_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    let res = invoke(
        &app,
        &window,
        "export_report",
        serde_json::json!({
            "format": "xyz",
            "targetPath": target.to_string_lossy().to_string(),
            "analysisJson": analysis_json,
        }),
    );
    let err = res.expect_err("unknown format fails");
    let message = err.as_str().unwrap_or_default();
    assert!(message.contains("Unknown export format"), "got {message}");
    assert!(!target.exists());

    window.close().expect("close");
}

#[test]
fn get_recent_repositories_command_returns_list() {
    let app = build_full_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    let res = invoke(
        &app,
        &window,
        "get_recent_repositories",
        serde_json::Value::Null,
    );
    let list = res.expect("command succeeds");
    assert!(
        list.is_array(),
        "expected an array of recent repositories, got {list}"
    );

    window.close().expect("close");
}

#[test]
fn open_external_url_rejects_unsafe_schemes() {
    let app = build_full_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    for bad in [
        "javascript:alert(1)",
        "ftp://example.com/file",
        "file:///etc/passwd",
        "not a url",
    ] {
        let res = invoke(
            &app,
            &window,
            "open_external_url",
            serde_json::json!({ "url": bad }),
        );
        assert!(
            res.is_err(),
            "open_external_url must reject unsafe/odd URL {bad:?}"
        );
    }

    window.close().expect("close");
}

#[test]
fn analyze_github_repository_command_rejects_http_url() {
    // The command must refuse plain-http GitHub URLs before any network access.
    let app = build_handler_app();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("window");

    let res = invoke(
        &app,
        &window,
        "analyze_github_repository",
        serde_json::json!({ "url": "http://github.com/octocat/Hello-World" }),
    );
    let err = res.expect_err("http GitHub URL must be rejected");
    let message = err.as_str().unwrap_or_default().to_lowercase();
    assert!(
        message.contains("https"),
        "error should mention https, got {message}"
    );

    window.close().expect("close");
}
