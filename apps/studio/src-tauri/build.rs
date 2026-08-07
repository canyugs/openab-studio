fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["workspace_bootstrap"])),
    )
    .expect("failed to build OpenAB Studio's Tauri permissions");
}
