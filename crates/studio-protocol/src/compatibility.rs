use crate::generated::{
    CompatibilityDecision, CompatibilityPeer, CompatibilityRequest, SchemaProfile,
    SelectedSchemaProfile,
};

/// Selects schema profiles and capabilities without inspecting a display version.
#[must_use]
pub fn decide_compatibility(
    request: &CompatibilityRequest,
    peer: &CompatibilityPeer,
) -> CompatibilityDecision {
    let mut selected_contracts = Vec::with_capacity(request.contracts.len());
    for requested in &request.contracts {
        let same_family = peer
            .contracts
            .iter()
            .filter(|candidate| candidate.family == requested.family)
            .collect::<Vec<_>>();
        let same_major = same_family
            .iter()
            .copied()
            .filter(|candidate| candidate.major == requested.major)
            .collect::<Vec<_>>();

        if same_major.is_empty() {
            return rejected(
                if same_family.is_empty() {
                    "schema-profile-unavailable"
                } else {
                    "schema-major-mismatch"
                },
                selected_contracts,
                Vec::new(),
                Vec::new(),
            );
        }

        let compatible_minor = same_major
            .into_iter()
            .filter_map(|candidate| highest_common_minor(requested, candidate))
            .max();
        let Some(minor) = compatible_minor else {
            return rejected(
                "schema-profile-unavailable",
                selected_contracts,
                Vec::new(),
                Vec::new(),
            );
        };

        selected_contracts.push(SelectedSchemaProfile {
            family: requested.family.clone(),
            major: requested.major,
            minor,
        });
    }

    let mut accepted_capabilities = Vec::new();
    for capability in &request.required_capabilities {
        if !peer.capabilities.contains(capability) {
            return rejected(
                "required-capability-unavailable",
                selected_contracts,
                accepted_capabilities,
                Vec::new(),
            );
        }
        push_unique(&mut accepted_capabilities, capability);
    }

    for extension in request.required_extensions.as_deref().unwrap_or_default() {
        if !peer.extensions.contains(extension) {
            return rejected(
                "required-extension-unavailable",
                selected_contracts,
                accepted_capabilities,
                Vec::new(),
            );
        }
    }

    let mut unavailable_capabilities = Vec::new();
    for capability in &request.optional_capabilities {
        if peer.capabilities.contains(capability) {
            push_unique(&mut accepted_capabilities, capability);
        } else {
            push_unique(&mut unavailable_capabilities, capability);
        }
    }

    CompatibilityDecision {
        outcome: if unavailable_capabilities.is_empty() {
            "supported".to_owned()
        } else {
            "degraded".to_owned()
        },
        reason: None,
        selected_contracts,
        accepted_capabilities,
        unavailable_capabilities,
        side_effect_permitted: true,
    }
}

fn highest_common_minor(requested: &SchemaProfile, peer: &SchemaProfile) -> Option<i64> {
    let lower = requested.min_minor.max(peer.min_minor);
    let upper = requested.max_minor.min(peer.max_minor);
    (lower <= upper).then_some(upper)
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|current| current == value) {
        values.push(value.to_owned());
    }
}

fn rejected(
    reason: &str,
    selected_contracts: Vec<SelectedSchemaProfile>,
    accepted_capabilities: Vec<String>,
    unavailable_capabilities: Vec<String>,
) -> CompatibilityDecision {
    CompatibilityDecision {
        outcome: "rejected".to_owned(),
        reason: Some(reason.to_owned()),
        selected_contracts,
        accepted_capabilities,
        unavailable_capabilities,
        side_effect_permitted: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::generated::{CompatibilityPeer, CompatibilityRequest, SchemaProfile};

    use super::decide_compatibility;

    #[test]
    fn selects_the_highest_minor_in_the_intersection() {
        let decision = decide_compatibility(
            &CompatibilityRequest {
                contracts: vec![SchemaProfile {
                    family: "studio.openab.dev/fleet-management".to_owned(),
                    major: 1,
                    min_minor: 2,
                    max_minor: 4,
                }],
                required_capabilities: vec!["revisioned-mutations".to_owned()],
                optional_capabilities: Vec::new(),
                required_extensions: None,
            },
            &CompatibilityPeer {
                contracts: vec![SchemaProfile {
                    family: "studio.openab.dev/fleet-management".to_owned(),
                    major: 1,
                    min_minor: 3,
                    max_minor: 5,
                }],
                capabilities: vec!["revisioned-mutations".to_owned()],
                extensions: Vec::new(),
            },
        );

        assert_eq!(decision.outcome, "supported");
        assert_eq!(decision.selected_contracts[0].minor, 4);
    }
}
