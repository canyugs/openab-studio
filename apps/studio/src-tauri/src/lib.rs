use studio_core::StudioCore;
use studio_protocol::{CoreErrorEnvelope, WorkspaceBootstrapResult};

/// The platform shell only adapts Tauri arguments to the trusted core command boundary.
#[tauri::command(rename_all = "camelCase")]
fn workspace_bootstrap(
    core: tauri::State<'_, StudioCore>,
    request_id: String,
) -> Result<WorkspaceBootstrapResult, Box<CoreErrorEnvelope>> {
    execute_workspace_bootstrap(&core, request_id)
}

fn execute_workspace_bootstrap(
    core: &StudioCore,
    request_id: String,
) -> Result<WorkspaceBootstrapResult, Box<CoreErrorEnvelope>> {
    core.workspace_bootstrap(request_id)
        .map_err(|error| Box::new(error.into_public()))
}

/// Runs the same Tauri shell for desktop and mobile target families.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(StudioCore::default())
        .invoke_handler(tauri::generate_handler![workspace_bootstrap])
        .run(tauri::generate_context!())
        .expect("error while running OpenAB Studio");
}

#[cfg(test)]
mod tests {
    use studio_core::StudioCore;

    use super::execute_workspace_bootstrap;

    #[test]
    fn shell_adapter_returns_the_core_produced_bootstrap_result() {
        let result =
            execute_workspace_bootstrap(&StudioCore::default(), "req_shell_bootstrap".to_owned())
                .expect("shell bootstrap must succeed");

        assert_eq!(result.request_id, "req_shell_bootstrap");
        assert_eq!(result.workspace_bootstrap.protocol_version, 1);
        assert_eq!(result.workspace_bootstrap.status, "ready");
        assert_eq!(result.events[0].operation_id, result.operation_id);
    }
}
