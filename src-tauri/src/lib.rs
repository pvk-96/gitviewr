pub mod commands;
pub mod git;
pub mod models;
pub mod services;
pub mod storage;

pub mod utils {
    pub mod dates;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::repository::analyze_local_repository,
            commands::repository::analyze_github_repository,
            commands::recent::get_recent_repositories,
            commands::export::export_report,
            commands::system::open_external_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
