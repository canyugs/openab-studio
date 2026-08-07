//! Versioned shared schema bindings and compatibility helpers for OpenAB Studio.
//!
//! The canonical source lives at `schemas/studio.shared.v1alpha1.schema.json`.
//! Generated bindings are deliberately committed with that source so downstream consumers can
//! review a contract change without relying on a locally installed generator.

pub mod compatibility;
pub mod generated;
pub mod validation;

pub use compatibility::decide_compatibility;
pub use generated::*;
pub use validation::{
    MigrationResult, ValidationError, migrate_plugin_manifest, parse_plugin_manifest,
    parse_shared_contract_document, parse_with_definition, validate_definition,
};
