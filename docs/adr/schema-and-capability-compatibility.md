# ADR: Schema and Capability Compatibility Policy

- **Status:** Proposed
- **Date:** 2026-08-07
- **Related:** [System architecture](../architecture.md),
  [Cross-platform client](./remote-first-client.md),
  [Plugin platform](./plugin-platform.md),
  [issue #3](https://github.com/canyugs/openab-studio/issues/3)

## Context

Studio, the Fleet Management API, ACP-over-WebSocket, and the Studio Plugin Spec will evolve at
different rates and may be deployed independently. A product release label is useful for support,
but it does not say which schema a peer can safely read, which ACP session behavior is available, or
whether a plugin can receive a permission safely.

Compatibility errors at these boundaries are security and data-integrity risks. Silently ignoring a
new grant condition can widen authority; treating a required persistence field as optional can corrupt
shared state; and inferring behavior from a display version can enable an ACP feature a particular
instance did not advertise. The policy therefore needs a common decision model before schemas,
generators, or runtime implementations are introduced.

## Decision

Compatibility is negotiated per contract and operation, not inferred from a Studio, OpenAB, plugin,
or package display version. Every compatibility decision has a selected schema profile, an explicit
set of accepted capabilities, and a visible outcome: **supported**, **degraded**, or **rejected**.

The exact Fleet Management API encoding remains a separate decision. JSON, YAML, protobuf, or a
future generated binding may represent this policy differently, but each representation must preserve
the following normalized information and rules.

### Compatibility vocabulary

| Term | Meaning | Not a substitute for |
|---|---|---|
| Display version | Human-facing application or package release label, such as **0.4.0**. It is diagnostic metadata only. | A schema profile or capability grant. |
| Contract family | A stable reverse-DNS identifier for one independently versioned contract, such as **studio.openab.dev/fleet-management**. | A product family or repository name. |
| Schema profile | A contract family plus major and minor schema revision. A peer advertises the inclusive minor range it can correctly read and write for one major. | A promise that every optional feature is implemented. |
| Capability | A stable, opaque identifier for one behavior with defined semantics, such as **studio.openab.dev/fleet/grants-revisioned-mutations**. | A guessed behavior derived from a version string. |
| Extension | A namespaced, explicitly optional data contribution whose semantics are outside the base schema. | A way to bypass schema validation or change security semantics. |
| Required capability or extension | A capability or extension the sender needs for the operation to be safe and correct. | A UI preference that can be silently dropped. |

The normalized profile declaration is:

```text
family: <reverse-DNS contract identifier>
major: <positive integer>
minMinor: <lowest compatible minor revision>
maxMinor: <highest compatible minor revision>
```

Two stable profiles have a common profile only when their **family** and **major** match and their minor
ranges overlap. The selected profile is the highest minor in that overlap. Once selected, both peers
serialize only the fields defined by that profile plus extensions permitted by that profile. An
extension that requires behavior must also be negotiated. A sender must not send a newer base-schema
field merely because it understands it locally.

Stable minor revisions may add only optional, default-safe base fields. Removing a field, changing a
field's meaning, changing authorization or persistence semantics, or making an optional behavior
required requires a new major profile or a new capability identifier. Pre-release profiles (for
example, **v1alpha1**) are exact-match profiles unless their owning contract explicitly publishes an
interoperability mapping; an **alpha** or **beta** suffix is never treated as an automatically compatible
range.

### Negotiation rules

1. A peer advertises its supported schema profile ranges and capability identifiers before it sends
   contract data or attempts a privileged operation.
2. The responder selects one common profile for each needed contract family and explicitly reports
   each required and optional capability as accepted, unavailable, or rejected with a machine-readable
   reason.
3. An operation proceeds only against the selected profile and the capabilities accepted for that
   operation. A capability is not implied by a product, package, or server display version.
4. Required capabilities and extensions are evaluated before authorization, mutation, plugin enable,
   or state migration. If any is unavailable or unknown, the operation is rejected before side effects.
5. Optional capability absence may produce a degraded experience only when the remaining behavior is
   correct, authorized, and durable. It must be visible to the caller; it must not silently weaken a
   permission, consistency, audit, revocation, or migration guarantee.
6. A negotiated selection is scoped to the identified Fleet, instance, session, and plugin package
   digest as applicable. It is renegotiated when that scope changes or a peer reconnects; a selection
   from another Fleet or package must not be reused.

Capability identifiers are stable opaque names. An incompatible change receives a new identifier,
rather than requiring consumers to parse a suffix or compare a display version. Parameters associated
with a capability have their own selected schema profile and validation rules.

### Plane-specific application

| Plane | What is selected | Required rule |
|---|---|---|
| Studio ↔ Fleet Management API | Fleet API schema profile and management capabilities, including mutation, event, grant, and audit semantics. | The endpoint must reject a mutation if its selected profile or required safety capability is absent. |
| Studio ↔ OpenAB over ACP | ACP initialization result plus the ACP capabilities required for the requested session behavior. | Studio uses the structured ACP initialization/capability result; it never feature-gates a session by parsing an OpenAB release label. |
| Studio Plugin Spec ↔ plugin host | Plugin manifest schema profile, placement/runtime capabilities, and each required contribution or migration capability. | A package is validated before install or enable. Its package **version** and Studio display version do not prove compatibility. |
| Studio local core ↔ persisted state | The local data schema profile and migration capability/plan. | A migration is explicit, reversible or recoverable as specified by its owner, and never inferred from the UI build number. |

The existing **compatibility.studio** display-version range shown in the Plugin Platform ADR is a
pre-policy illustration, not permission to gate installation or enablement by a display version. The
Plugin Spec owner must replace it with contract-profile and capability requirements when the manifest
schema is accepted. Until then, an implementation must not claim that a package is compatible solely
because that range matches.

### Unknown fields and extensions

Base schemas are strict at the selected profile. An unknown field outside the schema's declared
extension container is rejected with **schema-unknown-field**; implementations must not silently ignore
it. Required base fields are validated before the operation is authorized or persisted.

Optional extensions live only in an **extensions** container keyed by a reverse-DNS namespace, for
example **io.example.telemetry**. An optional extension:

- declares its own schema/profile inside its namespace;
- cannot alter identity, authorization, grant evaluation, revocation, auditing, durability, or base
  resource semantics when ignored;
- may be ignored only when it is not named in the operation's required extensions or capabilities; and
- must be preserved as opaque data only where the selected base contract explicitly permits
  pass-through preservation. A component must reject rather than strip data when preservation is
  required for correctness.

An extension that changes security, shared-state, or lifecycle semantics is not optional. Its
namespace must be named as a required extension or capability before use. An unknown required
extension fails closed with **required-extension-unavailable**, even if its payload is syntactically
well-formed.

## Compatibility outcomes and combinations

| Outcome | Selected base profile | Required capabilities/extensions | Required behavior |
|---|---|---|---|
| **Supported** | A common profile is selected. | All are accepted and understood. | Perform the advertised operation and record the selected profile/capabilities in diagnostics or audit context where appropriate. |
| **Degraded** | A common profile is selected. | All safety- and correctness-critical requirements are accepted; only explicitly optional behavior is unavailable. | Perform only the safe subset, expose the unavailable feature and reason, and do not report the unavailable behavior as complete. |
| **Rejected** | No common profile exists, or validation fails. | At least one required item is unavailable, unknown, or semantically unsafe. | Stop before side effects and return a stable rejection reason; offer a migration or alternative only if one was explicitly declared. |

The following combinations are the minimum conformance matrix for later fixtures and tests. The
capability names are policy examples; a protocol owner may register different names only with the
same semantics and compatibility evidence.

| Participants and declaration | Result | Required observable behavior |
|---|---|---|
| Studio supports Fleet Management **v1** minors **2..4**; the Fleet endpoint supports **v1** minors **3..5**; both accept **studio.openab.dev/fleet/grants-revisioned-mutations** and **studio.openab.dev/fleet/event-cursor**. | **Supported** — select **v1.4**. | Revisioned grant mutation and event-cursor synchronization are available. Neither peer emits a **v1.5** base field. |
| The same Studio and Fleet endpoint select **v1.4**, but the endpoint marks **studio.openab.dev/fleet/event-cursor** unavailable and it is optional for the inventory read. | **Degraded**. | Inventory read succeeds without resumable event synchronization; Studio displays that live cursor sync is unavailable and does not imply it is current. |
| Studio requests an ACP session with its required session capability accepted; the instance does not advertise the optional resume capability. | **Degraded**. | A new session may run, but resume is disabled and presented as unavailable. Studio does not inspect the instance's OpenAB display version to override ACP negotiation. |
| A mobile Studio manages a plugin whose selected manifest profile and **remote** placement capability are accepted, while the device does not advertise local **mcp-stdio** execution. | **Degraded** for local execution, **supported** for remote management. | The user may manage the remote/instance placement; a local-execution action is unavailable rather than falsely shown as an authorization failure. |
| Studio supports Fleet Management **v1** minors **2..4**; the endpoint supports only **v2** minors **0..1**. | **Rejected** — **schema-major-mismatch**. | No management mutation or cache reconciliation begins on the assumption that the newer endpoint is compatible. |
| An ACP operation requires **studio.openab.dev/acp/revision-bound-session**, but the initialized ACP capability result does not accept it. | **Rejected** — **required-capability-unavailable**. | Studio does not create a session whose required revision binding cannot be honored. |
| A plugin requires an extension the host does not recognize. | **Rejected** — **required-extension-unavailable**. | The package is not installed or enabled; the host does not treat the extension as an optional unknown field. |

## Concrete fixture examples

These snippets use the normalized model and are suitable as input/output fixtures. They do not choose
the final Fleet Management API wire encoding.

### Valid Fleet Management negotiation

Studio requests a revisioned grant mutation and an optional event cursor:

```json
{
  "contracts": [
    {
      "family": "studio.openab.dev/fleet-management",
      "major": 1,
      "minMinor": 2,
      "maxMinor": 4
    }
  ],
  "requiredCapabilities": [
    "studio.openab.dev/fleet/grants-revisioned-mutations"
  ],
  "optionalCapabilities": [
    "studio.openab.dev/fleet/event-cursor"
  ]
}
```

The Fleet endpoint accepts the common profile and both capabilities:

```json
{
  "selectedContracts": [
    {
      "family": "studio.openab.dev/fleet-management",
      "major": 1,
      "minor": 4
    }
  ],
  "acceptedCapabilities": [
    "studio.openab.dev/fleet/grants-revisioned-mutations",
    "studio.openab.dev/fleet/event-cursor"
  ],
  "unavailableCapabilities": []
}
```

Expected result: **supported**. A **v1.5** field is invalid in a message sent under this selection.

### Valid optional extension

This post-policy Plugin Spec fragment has an optional, namespaced telemetry extension. A host that
does not understand **io.example.telemetry** may ignore it because it is absent from the required list
and has no authority or persistence effect.

```yaml
schemaVersion: studio.openab.dev/v1alpha1
compatibility:
  contracts:
    - family: studio.openab.dev/plugin-manifest
      major: 1
      minMinor: 0
      maxMinor: 0
  requiredCapabilities:
    - studio.openab.dev/plugin/mcp-http
extensions:
  io.example.telemetry:
    schemaVersion: 1
    samplingRate: 0.1
```

Expected result: **supported** when the selected Plugin Spec profile and **mcp-http** capability are
accepted; otherwise the result is rejected for the missing base requirement, not because an optional
extension is present.

### Invalid: unnamespaced unknown base field

```yaml
schemaVersion: studio.openab.dev/v1alpha1
id: io.example.echo
dangerouslyBypassGrantChecks: true
```

Expected result: **rejected** with **schema-unknown-field**. The field is not inside **extensions**, and
the validator must not ignore a possible authorization-affecting instruction.

### Invalid: unknown required extension

```yaml
schemaVersion: studio.openab.dev/v1alpha1
compatibility:
  requiredExtensions:
    - io.example.encrypted-state
extensions:
  io.example.encrypted-state:
    schemaVersion: 1
    keyReference: fleet-managed
```

Expected result for a host that did not negotiate **io.example.encrypted-state**: **rejected** with
**required-extension-unavailable**. It must not install, enable, migrate, or persist the package as
though encryption were optional.

### Invalid: display-version parsing

```text
If OpenAB display version is at least 0.4.0, enable session resume.
```

Expected result: **invalid policy**. The correct condition is the ACP initialization result explicitly
accepting the resume capability for the current session. A display version may be shown in diagnostics,
but it cannot change compatibility or authorization behavior.

## Deprecation, migration, and fixture ownership

The workstream role owns the compatibility contract it changes; it may not delegate the migration
decision to a consumer that cannot define the contract's semantics.

| Contract or artifact | Accountable owner | Responsibilities before deprecation or removal |
|---|---|---|
| Fleet Management API profiles, endpoint behavior, and shared Fleet data migration | W2 Fleet/OpenAB contracts | Publish the successor profile/capability, define server-side and client-visible migration/rejection behavior, and retain compatible, migration, rollback/recovery, and rejection fixtures. |
| ACP protocol capability semantics | The upstream OpenAB ACP protocol owner; W2 owns the Studio mapping | Publish the ACP capability change upstream and use a paired cross-repository issue. W2 must prove Studio's mapping, negotiation, fallback, and rejection behavior against the accepted ACP contract. |
| Plugin Spec/manifest profiles, plugin package migrations, and extension registry | W4 Plugin platform | Define validator behavior, package migration/rollback ownership, namespace registration, and supported/rejected manifest fixtures. The plugin host must not infer this from package or Studio display versions. |
| Studio local persisted data, compatibility-selection enforcement, and recovery | W1 Trusted Studio core | Own explicit local migrations, cache invalidation, recovery/rollback behavior, and enforcement that a selected profile cannot be silently bypassed by the UI. |
| Shared cross-language compatibility corpus and conformance harness | W7 Dogfood and quality, with the semantic contract owner | Retain normalized request/response, manifest, migration, and rejection fixtures; ensure Rust and TypeScript make the same compatibility decision. W7 does not change a fixture's semantic outcome without the accountable contract owner. |

Before deprecating a profile, capability, field, or extension, its accountable owner must:

1. publish the successor or explicit terminal rejection behavior;
2. mark the old item deprecated in its contract metadata and documentation without changing current
   negotiated behavior silently;
3. provide an explicit migration path when persisted state, package configuration, grants, or shared
   Fleet data are affected, including recovery or rollback behavior where applicable;
4. add or retain fixtures for the oldest supported path, newest supported path, forward migration,
   recovery/rollback, and the eventual rejected legacy input; and
5. make removal a selected-major/profile or capability change, not an undocumented implementation
   change behind the same compatibility declaration.

Fixtures are retained for every supported and degraded selection while it is supported. When a path is
removed, its rejected-input and migration/recovery fixtures remain in the historical compatibility
corpus; they are not deleted merely because the old behavior is no longer enabled. Each fixture records
the peer declarations, selected profile, accepted/unavailable requirements, expected outcome, stable
reason code, and whether a side effect is permitted. This retention makes a future compatibility
decision auditable and prevents a removed safety guard from returning unnoticed.

## Consequences

Studio can distinguish an unavailable optional feature from an unsafe connection, rather than making
a broad claim that a server or plugin is "new enough." Fleet and plugin operators receive predictable
fail-closed behavior for unknown required semantics, while third parties retain a constrained path for
optional extensions.

This policy adds compatibility fixtures and explicit migration work to every contract change. That
cost is intentional: support for a schema or capability is a testable promise, not an assumption made
from a release name. The policy does not itself implement schemas, code generators, a release calendar,
or runtime negotiation.

## Acceptance criteria

- Supported, degraded, and rejected cross-plane combinations are defined with required outcomes.
- Required unknown fields and extensions fail closed; optional extensions are namespaced and constrained.
- Studio, Fleet Management API, ACP, and Plugin Spec compatibility are selected through profiles and
  capabilities rather than display-version parsing.
- Deprecation, migration, and fixture retention have accountable owners and explicit evidence rules.
- The valid and invalid examples above can become cross-language contract fixtures without selecting a
  final wire encoding.
