/// Opens a URL in the user's default browser via the OS.
#[tauri::command]
pub fn open_external_url(url: String) -> std::result::Result<(), String> {
    // Only allow opening http(s) links through this command.
    let parsed = url::Url::parse(&url).map_err(|e| format!("The URL could not be parsed: {e}"))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Only http(s) URLs can be opened.".to_string());
    }
    open::that(url).map_err(|e| format!("Could not open the URL in your browser: {e}"))
}
