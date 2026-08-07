use std::{fmt, sync::OnceLock};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::generated::{PluginManifest, SharedContractDocument};

const SCHEMA_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../schemas/studio.shared.v1alpha1.schema.json"
));

static SHARED_SCHEMA: OnceLock<Value> = OnceLock::new();

/// Stable, machine-readable reason returned by the shared schema validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str,
    pub path: String,
}

impl ValidationError {
    fn at(code: &'static str, path: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}", self.code, self.path)
    }
}

impl std::error::Error for ValidationError {}

/// The result of applying a schema-defined, forward-only manifest migration.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationResult {
    pub migrated: bool,
    pub value: Value,
}

/// Validates a value against one named definition from the canonical schema source.
pub fn validate_definition(
    definition: &str,
    value: &Value,
    known_extensions: &[&str],
) -> Result<(), ValidationError> {
    let schema = shared_schema();
    let definition_schema = schema
        .pointer(&format!("/$defs/{definition}"))
        .ok_or_else(|| ValidationError::at("schema-definition-unavailable", "$"))?;

    validate_node(schema, definition_schema, value, "$")?;

    if definition == "PluginManifest" {
        validate_required_extensions(value, known_extensions)?;
    }

    Ok(())
}

/// Validates and deserializes a value into one of the generated Rust bindings.
pub fn parse_with_definition<T>(
    definition: &str,
    value: Value,
    known_extensions: &[&str],
) -> Result<T, ValidationError>
where
    T: DeserializeOwned,
{
    validate_definition(definition, &value, known_extensions)?;
    serde_json::from_value(value)
        .map_err(|_| ValidationError::at("schema-deserialization-failed", "$"))
}

/// Validates and deserializes a generated plugin manifest binding.
pub fn parse_plugin_manifest(
    value: Value,
    known_extensions: &[&str],
) -> Result<PluginManifest, ValidationError> {
    parse_with_definition("PluginManifest", value, known_extensions)
}

/// Validates and deserializes the fixture document spanning all shared contract types.
pub fn parse_shared_contract_document(
    value: Value,
    known_extensions: &[&str],
) -> Result<SharedContractDocument, ValidationError> {
    parse_with_definition("SharedContractDocument", value, known_extensions)
}

/// Applies the migration declared by the canonical source, then validates the resulting manifest.
pub fn migrate_plugin_manifest(
    value: Value,
    known_extensions: &[&str],
) -> Result<MigrationResult, ValidationError> {
    let mut migrated_value = value;
    let schema_version = migrated_value
        .get("schemaVersion")
        .and_then(Value::as_str)
        .ok_or_else(|| ValidationError::at("schema-required-field", "$.schemaVersion"))?
        .to_owned();

    let migrations = shared_schema()
        .get("x-openab-migrations")
        .and_then(Value::as_array)
        .expect("canonical schema must contain x-openab-migrations");

    let migration = migrations.iter().find(|candidate| {
        candidate.get("definition").and_then(Value::as_str) == Some("PluginManifest")
            && candidate.get("fromSchemaVersion").and_then(Value::as_str) == Some(&schema_version)
    });

    if let Some(migration) = migration {
        let manifest = migrated_value
            .as_object_mut()
            .ok_or_else(|| ValidationError::at("schema-type-mismatch", "$"))?;
        let target = migration
            .get("toSchemaVersion")
            .and_then(Value::as_str)
            .expect("canonical migration must name a target schema version");

        for (from, to) in migration
            .get("rename")
            .and_then(Value::as_object)
            .expect("canonical migration must declare rename instructions")
        {
            let target_field = to
                .as_str()
                .expect("canonical migration rename target must be a string");
            if let Some(value) = manifest.remove(from) {
                if manifest.contains_key(target_field) {
                    return Err(ValidationError::at("schema-migration-conflict", "$"));
                }
                manifest.insert(target_field.to_owned(), value);
            }
        }
        manifest.insert("schemaVersion".to_owned(), Value::String(target.to_owned()));

        validate_definition("PluginManifest", &migrated_value, known_extensions)?;
        return Ok(MigrationResult {
            migrated: true,
            value: migrated_value,
        });
    }

    let current_version = shared_schema()
        .get("x-openab-schema-version")
        .and_then(Value::as_str)
        .expect("canonical schema must declare x-openab-schema-version");
    if schema_version != current_version {
        return Err(ValidationError::at(
            "schema-migration-unavailable",
            "$.schemaVersion",
        ));
    }

    validate_definition("PluginManifest", &migrated_value, known_extensions)?;
    Ok(MigrationResult {
        migrated: false,
        value: migrated_value,
    })
}

fn shared_schema() -> &'static Value {
    SHARED_SCHEMA.get_or_init(|| {
        serde_json::from_str(SCHEMA_SOURCE)
            .expect("canonical shared schema source must be valid JSON")
    })
}

fn validate_node(
    root: &Value,
    node: &Value,
    value: &Value,
    path: &str,
) -> Result<(), ValidationError> {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        let referenced = root
            .pointer(reference.trim_start_matches('#'))
            .ok_or_else(|| ValidationError::at("schema-reference-unavailable", path))?;
        return validate_node(root, referenced, value, path);
    }

    if let Some(constant) = node.get("const") {
        if value != constant {
            return Err(ValidationError::at("schema-const-mismatch", path));
        }
    }

    if let Some(allowed) = node.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return Err(ValidationError::at("schema-enum-mismatch", path));
        }
    }

    match node.get("type").and_then(Value::as_str) {
        Some("object") => validate_object(root, node, value, path),
        Some("array") => validate_array(root, node, value, path),
        Some("string") => validate_string(node, value, path),
        Some("integer") => validate_integer(node, value, path),
        Some("number") => validate_number(node, value, path),
        Some("boolean") if value.is_boolean() => Ok(()),
        Some("boolean") => Err(ValidationError::at("schema-type-mismatch", path)),
        Some(_) | None => Err(ValidationError::at("schema-type-unsupported", path)),
    }
}

fn validate_object(
    root: &Value,
    node: &Value,
    value: &Value,
    path: &str,
) -> Result<(), ValidationError> {
    let object = value
        .as_object()
        .ok_or_else(|| ValidationError::at("schema-type-mismatch", path))?;
    let properties = node
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for required in node
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let property = required
            .as_str()
            .expect("canonical required entry must be a string");
        if !object.contains_key(property) {
            return Err(ValidationError::at(
                "schema-required-field",
                child_path(path, property),
            ));
        }
    }

    for (property, child_value) in object {
        if let Some(property_schema) = properties.get(property) {
            validate_node(
                root,
                property_schema,
                child_value,
                &child_path(path, property),
            )?;
            continue;
        }

        match node.get("additionalProperties") {
            Some(Value::Bool(false)) => {
                return Err(ValidationError::at(
                    "schema-unknown-field",
                    child_path(path, property),
                ));
            }
            Some(Value::Bool(true)) | None => {}
            Some(schema) => validate_node(root, schema, child_value, &child_path(path, property))?,
        }
    }

    if let Some(pattern) = node
        .get("propertyNames")
        .and_then(|property_names| property_names.get("pattern"))
        .and_then(Value::as_str)
    {
        for property in object.keys() {
            if !matches_supported_pattern(property, pattern) {
                return Err(ValidationError::at(
                    "schema-invalid-extension-namespace",
                    child_path(path, property),
                ));
            }
        }
    }

    Ok(())
}

fn validate_array(
    root: &Value,
    node: &Value,
    value: &Value,
    path: &str,
) -> Result<(), ValidationError> {
    let items = value
        .as_array()
        .ok_or_else(|| ValidationError::at("schema-type-mismatch", path))?;
    if let Some(minimum) = node.get("minItems").and_then(Value::as_u64) {
        if items.len() < minimum as usize {
            return Err(ValidationError::at("schema-min-items", path));
        }
    }

    let item_schema = node
        .get("items")
        .expect("canonical array schema must declare items");
    for (index, item) in items.iter().enumerate() {
        validate_node(root, item_schema, item, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

fn validate_string(node: &Value, value: &Value, path: &str) -> Result<(), ValidationError> {
    let string = value
        .as_str()
        .ok_or_else(|| ValidationError::at("schema-type-mismatch", path))?;
    if let Some(minimum) = node.get("minLength").and_then(Value::as_u64) {
        if string.chars().count() < minimum as usize {
            return Err(ValidationError::at("schema-min-length", path));
        }
    }
    Ok(())
}

fn validate_integer(node: &Value, value: &Value, path: &str) -> Result<(), ValidationError> {
    let integer = value
        .as_i64()
        .ok_or_else(|| ValidationError::at("schema-type-mismatch", path))?;
    if let Some(minimum) = node.get("minimum").and_then(Value::as_i64) {
        if integer < minimum {
            return Err(ValidationError::at("schema-minimum", path));
        }
    }
    if let Some(maximum) = node.get("maximum").and_then(Value::as_i64) {
        if integer > maximum {
            return Err(ValidationError::at("schema-maximum", path));
        }
    }
    Ok(())
}

fn validate_number(node: &Value, value: &Value, path: &str) -> Result<(), ValidationError> {
    let number = value
        .as_f64()
        .ok_or_else(|| ValidationError::at("schema-type-mismatch", path))?;
    if let Some(minimum) = node.get("minimum").and_then(Value::as_f64) {
        if number < minimum {
            return Err(ValidationError::at("schema-minimum", path));
        }
    }
    if let Some(maximum) = node.get("maximum").and_then(Value::as_f64) {
        if number > maximum {
            return Err(ValidationError::at("schema-maximum", path));
        }
    }
    Ok(())
}

fn validate_required_extensions(
    value: &Value,
    known_extensions: &[&str],
) -> Result<(), ValidationError> {
    let required_extensions = value
        .pointer("/compatibility/requiredExtensions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    let declared_extensions = value.get("extensions").and_then(Value::as_object);

    for (index, required_extension) in required_extensions.enumerate() {
        let namespace = required_extension
            .as_str()
            .expect("canonical required extension must be a string");
        let is_declared =
            declared_extensions.is_some_and(|extensions| extensions.contains_key(namespace));
        let is_known = known_extensions.contains(&namespace);
        if !is_declared || !is_known {
            return Err(ValidationError::at(
                "required-extension-unavailable",
                format!("$.compatibility.requiredExtensions[{index}]"),
            ));
        }
    }
    Ok(())
}

fn matches_supported_pattern(value: &str, pattern: &str) -> bool {
    match pattern {
        "^[a-z0-9]+(?:[.-][a-z0-9]+)+$" => {
            let mut segment_has_character = false;
            let mut separator_seen = false;
            for character in value.chars() {
                if character.is_ascii_lowercase() || character.is_ascii_digit() {
                    segment_has_character = true;
                } else if (character == '.' || character == '-') && segment_has_character {
                    separator_seen = true;
                    segment_has_character = false;
                } else {
                    return false;
                }
            }
            separator_seen && segment_has_character
        }
        _ => false,
    }
}

fn child_path(path: &str, property: &str) -> String {
    format!("{path}.{property}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{matches_supported_pattern, validate_definition};

    #[test]
    fn extension_namespace_pattern_is_strict() {
        assert!(matches_supported_pattern(
            "io.example.telemetry",
            "^[a-z0-9]+(?:[.-][a-z0-9]+)+$"
        ));
        assert!(!matches_supported_pattern(
            "io.example..telemetry",
            "^[a-z0-9]+(?:[.-][a-z0-9]+)+$"
        ));
    }

    #[test]
    fn strict_base_schema_rejects_unknown_fields() {
        let result = validate_definition(
            "FleetIdentity",
            &json!({
                "id": "flt_1",
                "displayName": "Fleet",
                "revision": "rev_1",
                "untrusted": true
            }),
            &[],
        );
        assert_eq!(
            result.expect_err("must reject unknown base field").code,
            "schema-unknown-field"
        );
    }
}
