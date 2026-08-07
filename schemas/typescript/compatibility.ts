import type {
  CompatibilityDecision,
  CompatibilityPeer,
  CompatibilityRequest,
  SchemaProfile,
  SelectedSchemaProfile,
} from "../generated/typescript/studio-protocol.js";

/** Selects schema profiles and capabilities without inspecting a display version. */
export function decideCompatibility(
  request: CompatibilityRequest,
  peer: CompatibilityPeer,
): CompatibilityDecision {
  const selectedContracts: SelectedSchemaProfile[] = [];
  for (const requested of request.contracts) {
    const sameFamily = peer.contracts.filter(
      (candidate) => candidate.family === requested.family,
    );
    const sameMajor = sameFamily.filter(
      (candidate) => candidate.major === requested.major,
    );
    if (sameMajor.length === 0) {
      return rejected(
        sameFamily.length === 0
          ? "schema-profile-unavailable"
          : "schema-major-mismatch",
        selectedContracts,
        [],
        [],
      );
    }

    const commonMinors = sameMajor
      .map((candidate) => highestCommonMinor(requested, candidate))
      .filter((minor): minor is number => minor !== undefined);
    if (commonMinors.length === 0) {
      return rejected("schema-profile-unavailable", selectedContracts, [], []);
    }
    selectedContracts.push({
      family: requested.family,
      major: requested.major,
      minor: Math.max(...commonMinors),
    });
  }

  const acceptedCapabilities: string[] = [];
  for (const capability of request.requiredCapabilities) {
    if (!peer.capabilities.includes(capability)) {
      return rejected(
        "required-capability-unavailable",
        selectedContracts,
        acceptedCapabilities,
        [],
      );
    }
    pushUnique(acceptedCapabilities, capability);
  }

  for (const extension of request.requiredExtensions ?? []) {
    if (!peer.extensions.includes(extension)) {
      return rejected(
        "required-extension-unavailable",
        selectedContracts,
        acceptedCapabilities,
        [],
      );
    }
  }

  const unavailableCapabilities: string[] = [];
  for (const capability of request.optionalCapabilities) {
    if (peer.capabilities.includes(capability)) {
      pushUnique(acceptedCapabilities, capability);
    } else {
      pushUnique(unavailableCapabilities, capability);
    }
  }

  return {
    outcome: unavailableCapabilities.length === 0 ? "supported" : "degraded",
    selectedContracts,
    acceptedCapabilities,
    unavailableCapabilities,
    sideEffectPermitted: true,
  };
}

function highestCommonMinor(
  requested: SchemaProfile,
  peer: SchemaProfile,
): number | undefined {
  const lower = Math.max(requested.minMinor, peer.minMinor);
  const upper = Math.min(requested.maxMinor, peer.maxMinor);
  return lower <= upper ? upper : undefined;
}

function pushUnique(values: string[], value: string): void {
  if (!values.includes(value)) {
    values.push(value);
  }
}

function rejected(
  reason: string,
  selectedContracts: SelectedSchemaProfile[],
  acceptedCapabilities: string[],
  unavailableCapabilities: string[],
): CompatibilityDecision {
  return {
    outcome: "rejected",
    reason,
    selectedContracts,
    acceptedCapabilities,
    unavailableCapabilities,
    sideEffectPermitted: false,
  };
}
