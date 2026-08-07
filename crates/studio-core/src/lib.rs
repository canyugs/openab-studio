//! Minimal trusted-core seam for the workspace bootstrap.
//!
//! Product operations, storage, transport, identity, and plugins deliberately
//! live outside this crate until their contracts are accepted.

use serde::Serialize;

/// Typed response returned through the initial Tauri command boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBootstrap {
    /// Version of this intentionally tiny Rust-to-TypeScript boundary.
    pub protocol_version: u16,
    /// Static readiness marker; this command intentionally has no side effects.
    pub status: BootstrapStatus,
}

/// State reported by the initial workspace bootstrap command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapStatus {
    Ready,
}

/// Returns the typed, side-effect-free response used to verify the app boundary.
#[must_use]
pub const fn workspace_bootstrap() -> WorkspaceBootstrap {
    WorkspaceBootstrap {
        protocol_version: 1,
        status: BootstrapStatus::Ready,
    }
}

#[cfg(test)]
mod tests {
    use super::{BootstrapStatus, WorkspaceBootstrap, workspace_bootstrap};

    #[test]
    fn bootstrap_response_is_stable() {
        assert_eq!(
            workspace_bootstrap(),
            WorkspaceBootstrap {
                protocol_version: 1,
                status: BootstrapStatus::Ready,
            }
        );
    }
}
