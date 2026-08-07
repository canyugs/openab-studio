/// The sole bootstrap command verifies the typed Rust-to-TypeScript boundary.
#[tauri::command]
fn workspace_bootstrap() -> studio_core::WorkspaceBootstrap {
    studio_core::workspace_bootstrap()
}

/// Runs the same Tauri shell for desktop and mobile target families.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![workspace_bootstrap])
        .run(tauri::generate_context!())
        .expect("error while running OpenAB Studio");
}
