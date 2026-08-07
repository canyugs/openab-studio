//! Trusted command boundary for OpenAB Studio.
//!
//! Device shells are deliberately thin: they construct a typed command, dispatch it here, and
//! return the core-produced result. Product state, authorization, transport, and persistence are
//! added behind this boundary rather than implemented in TypeScript or platform adapters.

mod command;
mod error;

pub use command::{StudioCommand, StudioCommandResult, StudioCore};
pub use error::{CoreError, ErrorCategory};
