use std::sync::atomic::{AtomicU64, Ordering};

use studio_protocol::{
    CoreEventEnvelope, WorkspaceBootstrapCommand, WorkspaceBootstrapPayload,
    WorkspaceBootstrapResult,
};

use crate::{CoreError, ErrorCategory};

const WORKSPACE_BOOTSTRAP_KIND: &str = "workspace-bootstrap";

/// Every product operation has an explicit Rust variant before it can enter the trusted core.
#[derive(Debug, Clone, PartialEq)]
pub enum StudioCommand {
    WorkspaceBootstrap(WorkspaceBootstrapCommand),
}

/// Every successful operation leaves the trusted core as an explicit typed result.
#[derive(Debug, Clone, PartialEq)]
pub enum StudioCommandResult {
    WorkspaceBootstrap(WorkspaceBootstrapResult),
}

/// Process-local trusted core state shared by the platform shell.
///
/// Operation identifiers are opaque correlation values, not credentials. They are unique for the
/// lifetime of this core and must not be parsed for ordering or authorization.
#[derive(Debug, Default)]
pub struct StudioCore {
    next_operation: AtomicU64,
}

impl StudioCore {
    /// Constructs the typed bootstrap command used by every platform shell.
    pub fn workspace_bootstrap(
        &self,
        request_id: impl Into<String>,
    ) -> Result<WorkspaceBootstrapResult, CoreError> {
        let result = self.dispatch(StudioCommand::WorkspaceBootstrap(
            WorkspaceBootstrapCommand {
                kind: WORKSPACE_BOOTSTRAP_KIND.to_owned(),
                request_id: request_id.into(),
            },
        ))?;

        let StudioCommandResult::WorkspaceBootstrap(result) = result;
        Ok(result)
    }

    /// Dispatches one typed product operation through the trusted core boundary.
    pub fn dispatch(&self, command: StudioCommand) -> Result<StudioCommandResult, CoreError> {
        match command {
            StudioCommand::WorkspaceBootstrap(command) => {
                self.dispatch_workspace_bootstrap(command)
            }
        }
    }

    fn dispatch_workspace_bootstrap(
        &self,
        command: WorkspaceBootstrapCommand,
    ) -> Result<StudioCommandResult, CoreError> {
        let request_id = validate_request_id(command.request_id)?;
        if command.kind != WORKSPACE_BOOTSTRAP_KIND {
            return Err(CoreError::new(
                ErrorCategory::Unsupported,
                request_id,
                None,
                "core.command.unsupported",
                "unsupported command variant",
            ));
        }

        let operation_id = self.operation_id();
        let event = CoreEventEnvelope {
            command_kind: WORKSPACE_BOOTSTRAP_KIND.to_owned(),
            event_type: "operation-completed".to_owned(),
            operation_id: operation_id.clone(),
            outcome: "succeeded".to_owned(),
            request_id: request_id.clone(),
            sequence: 1,
        };
        let result = WorkspaceBootstrapResult {
            events: vec![event],
            operation_id,
            request_id,
            workspace_bootstrap: WorkspaceBootstrapPayload {
                protocol_version: 1,
                status: "ready".to_owned(),
            },
        };

        Ok(StudioCommandResult::WorkspaceBootstrap(result))
    }

    fn operation_id(&self) -> String {
        let sequence = self.next_operation.fetch_add(1, Ordering::Relaxed) + 1;
        format!("op_{sequence:016x}")
    }
}

fn validate_request_id(request_id: String) -> Result<String, CoreError> {
    let is_valid = !request_id.is_empty()
        && request_id.len() <= 128
        && request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if is_valid {
        return Ok(request_id);
    }

    Err(CoreError::new(
        ErrorCategory::InvalidInput,
        "req_invalid".to_owned(),
        None,
        "core.request-id.invalid",
        "request ID must be 1-128 ASCII letters, digits, dot, dash, or underscore",
    ))
}

#[cfg(test)]
mod tests {
    use studio_protocol::WorkspaceBootstrapCommand;

    use super::{StudioCommand, StudioCore};
    use crate::ErrorCategory;

    #[test]
    fn bootstrap_crosses_the_typed_boundary_with_correlated_event() {
        let core = StudioCore::default();
        let result = core
            .workspace_bootstrap("req_bootstrap_1")
            .expect("bootstrap must succeed");

        assert_eq!(result.request_id, "req_bootstrap_1");
        assert_eq!(result.operation_id, "op_0000000000000001");
        assert_eq!(result.workspace_bootstrap.protocol_version, 1);
        assert_eq!(result.workspace_bootstrap.status, "ready");
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].request_id, result.request_id);
        assert_eq!(result.events[0].operation_id, result.operation_id);
        assert_eq!(result.events[0].event_type, "operation-completed");
        assert_eq!(result.events[0].outcome, "succeeded");
    }

    #[test]
    fn operation_ids_are_core_generated_and_process_unique() {
        let core = StudioCore::default();
        let first = core
            .workspace_bootstrap("req_first")
            .expect("first bootstrap must succeed");
        let second = core
            .workspace_bootstrap("req_second")
            .expect("second bootstrap must succeed");

        assert_eq!(first.operation_id, "op_0000000000000001");
        assert_eq!(second.operation_id, "op_0000000000000002");
    }

    #[test]
    fn invalid_request_id_fails_before_an_operation_exists() {
        let error = StudioCore::default()
            .workspace_bootstrap("secret with spaces")
            .expect_err("invalid request ID must fail");

        assert_eq!(error.category(), ErrorCategory::InvalidInput);
        assert_eq!(error.public().request_id, "req_invalid");
        assert_eq!(error.public().operation_id, None);
        assert!(!error.public().message.contains("secret"));
    }

    #[test]
    fn a_forged_typed_variant_fails_closed() {
        let error = StudioCore::default()
            .dispatch(StudioCommand::WorkspaceBootstrap(
                WorkspaceBootstrapCommand {
                    kind: "future-command".to_owned(),
                    request_id: "req_forged".to_owned(),
                },
            ))
            .expect_err("unknown command variant must fail");

        assert_eq!(error.category(), ErrorCategory::Unsupported);
        assert_eq!(error.public().code, "core.command.unsupported");
        assert_eq!(error.public().operation_id, None);
    }
}
