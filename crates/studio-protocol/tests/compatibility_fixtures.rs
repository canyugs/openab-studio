use std::{fs, path::PathBuf};

use serde_json::{Map, Value};
use studio_protocol::{
    CompatibilityPeer, CompatibilityRequest, decide_compatibility, migrate_plugin_manifest,
    parse_plugin_manifest, parse_shared_contract_document, parse_with_definition,
    validate_definition,
};

#[test]
fn compatibility_and_validation_fixtures_match_the_policy() {
    let fixtures = fixture_files("compatibility");
    assert!(
        !fixtures.is_empty(),
        "compatibility fixture corpus must not be empty"
    );

    for fixture_path in fixtures {
        let fixture = read_fixture(&fixture_path);
        let fixture_name = fixture_name(&fixture);
        match fixture.get("kind").and_then(Value::as_str) {
            Some("compatibility") => {
                let request: CompatibilityRequest =
                    parse_with_definition("CompatibilityRequest", fixture["request"].clone(), &[])
                        .expect("compatibility request fixture must validate");
                let peer: CompatibilityPeer =
                    parse_with_definition("CompatibilityPeer", fixture["peer"].clone(), &[])
                        .expect("compatibility peer fixture must validate");
                let actual = serde_json::to_value(decide_compatibility(&request, &peer))
                    .expect("decision must serialize");

                assert_eq!(
                    canonicalize(actual),
                    canonicalize(fixture["expect"].clone()),
                    "{fixture_name}"
                );
            }
            Some("validation") => {
                let definition = fixture["definition"]
                    .as_str()
                    .expect("validation fixture definition must be a string");
                let known_extensions = known_extensions(&fixture);
                let expected = fixture["expect"]
                    .as_object()
                    .expect("validation fixture expectation must be an object");
                let result = validate_definition(definition, &fixture["input"], &known_extensions);

                if expected.get("valid") == Some(&Value::Bool(true)) {
                    result.expect("valid fixture must validate");
                    if definition == "PluginManifest" {
                        parse_plugin_manifest(fixture["input"].clone(), &known_extensions).expect(
                            "valid plugin manifest must deserialize into the generated binding",
                        );
                    }
                } else {
                    let error = result.expect_err("invalid fixture must fail validation");
                    assert_eq!(
                        error.code,
                        expected["code"]
                            .as_str()
                            .expect("invalid fixture must name a stable error code"),
                        "{fixture_name}"
                    );
                }
            }
            Some("roundtrip") => {
                let known_extensions = known_extensions(&fixture);
                let parsed =
                    parse_shared_contract_document(fixture["input"].clone(), &known_extensions)
                        .expect("round-trip fixture must validate and deserialize");
                let actual =
                    serde_json::to_value(parsed).expect("generated binding must serialize");

                assert_eq!(
                    canonicalize(actual),
                    canonicalize(fixture["input"].clone()),
                    "{fixture_name}"
                );
            }
            other => panic!("unsupported compatibility fixture kind {other:?} in {fixture_name}"),
        }
    }
}

#[test]
fn migration_fixtures_are_forward_only_and_explicit() {
    let fixtures = fixture_files("migrations");
    assert!(
        !fixtures.is_empty(),
        "migration fixture corpus must not be empty"
    );

    for fixture_path in fixtures {
        let fixture = read_fixture(&fixture_path);
        let fixture_name = fixture_name(&fixture);
        let known_extensions = known_extensions(&fixture);
        let expected = fixture["expect"]
            .as_object()
            .expect("migration expectation must be an object");

        if let Some(expected_error) = expected.get("error").and_then(Value::as_str) {
            let error = migrate_plugin_manifest(fixture["input"].clone(), &known_extensions)
                .expect_err("rejected migration fixture must fail");
            assert_eq!(error.code, expected_error, "{fixture_name}");
            continue;
        }

        let actual = migrate_plugin_manifest(fixture["input"].clone(), &known_extensions)
            .expect("migration fixture must succeed");
        assert_eq!(
            actual.migrated,
            expected["migrated"]
                .as_bool()
                .expect("migration expectation must name migrated state"),
            "{fixture_name}"
        );
        assert_eq!(
            canonicalize(actual.value),
            canonicalize(expected["value"].clone()),
            "{fixture_name}"
        );
    }
}

fn fixture_files(directory: &str) -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/fixtures")
        .join(directory);
    let mut fixtures = fs::read_dir(root)
        .expect("fixture directory must exist")
        .map(|entry| {
            entry
                .expect("fixture directory entry must be readable")
                .path()
        })
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    fixtures.sort();
    fixtures
}

fn read_fixture(path: &PathBuf) -> Value {
    serde_json::from_slice(&fs::read(path).expect("fixture file must be readable"))
        .expect("fixture file must contain valid JSON")
}

fn fixture_name(fixture: &Value) -> &str {
    fixture
        .get("name")
        .and_then(Value::as_str)
        .expect("fixture must have a name")
}

fn known_extensions(fixture: &Value) -> Vec<&str> {
    fixture
        .get("knownExtensions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        Value::Object(values) => {
            let mut sorted = values.into_iter().collect::<Vec<_>>();
            sorted.sort_by(|(left, _), (right, _)| left.cmp(right));
            Value::Object(
                sorted
                    .into_iter()
                    .map(|(key, value)| (key, canonicalize(value)))
                    .collect::<Map<_, _>>(),
            )
        }
        scalar => scalar,
    }
}
