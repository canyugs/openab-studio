# ADR: Public OpenAB Studio Plugin Platform

- **Status:** Proposed
- **Date:** 2026-08-07
- **Related:** [System architecture](../architecture.md),
  [Cross-platform client](./remote-first-client.md)

## Context

Studio needs integrations, provisioning, resources, memory providers, workflows, and eventually UI
extensions. First-party-only extension hooks would make Studio a growing monolith and prevent users
from connecting their own systems. Treating raw MCP configuration as the complete plugin model would
omit package identity, placement, lifecycle, compatibility, permissions, secrets, audit, and recovery.

The product must also work across desktop and mobile, where available execution environments differ.

## Decision

Adopt **kernel core-first, product features plugin-first**:

- the trusted kernel owns invariants, authority, lifecycle, recovery, and protocol mediation;
- integrations and replaceable domain features use a public Plugin SDK; and
- first-party plugins receive no private authority unavailable to third-party plugins.

MCP is the preferred tool/data protocol. The OpenAB Studio Plugin Spec wraps MCP and other supported
runtimes in a versioned package, permission, and lifecycle contract.

## Manifest

The initial manifest shape is declarative and has no installation-time executable hook:

```yaml
schemaVersion: studio.openab.dev/v1alpha1
id: dev.openab.echo
name: Echo
version: 0.1.0
publisher: openab
compatibility:
  studio: ">=0.1.0 <0.2.0"
platforms: [desktop, mobile]
runtime:
  placement: remote
  transport: mcp-http
  entrypoint: https://example.invalid/mcp
contributes:
  tools:
    - echo
permissions:
  network:
    allow: [example.invalid]
  resources:
    read: []
  secrets:
    use: []
```

The accepted schema must additionally define package digest/signature, runtime health, configuration
schema, migration compatibility, resource limits, contribution identifiers, and unambiguous platform
selectors. Unknown required fields fail closed; optional extensions remain namespaced.

## Extension stages

Extension points are introduced in this order:

1. **V0 tools/connectors:** MCP tools with explicit input/output and network/secret grants.
2. **V1 resource providers:** typed resources, actions, health, and revisioned synchronization.
3. **V2 memory providers:** scoped memory, provenance, search, retention, and deletion jobs.
4. **V3 UI contributions:** reviewed routes, panels, commands, and renderers with a constrained UI API.

UI extensions are intentionally last because arbitrary rendering increases phishing, accessibility,
state-isolation, and cross-platform compatibility risks.

## Runtime placements

| Placement/transport | Host | Target support | Intended use |
|---|---|---|---|
| `remote` / `mcp-http` | External service | All platforms | SaaS and network connectors |
| `instance` / MCP | OpenAB instance or fleet service | All clients | Fleet-local tools/providers |
| `device` / `mcp-stdio` | Desktop Studio host | Desktop only | Reviewed local command providers |
| `device` / native provider | Narrow Studio host API | Per platform | OS-specific capabilities |
| `device` / WASM-WASI | Studio sandbox | Later | Portable logic with bounded host calls |

Installation and execution are separate. A phone may install, configure, enable, disable, or roll
back a remote/instance plugin even though it cannot execute the plugin on-device.

## Lifecycle

The kernel implements a state machine with auditable transitions:

```text
discovered -> validated -> installed -> enabled -> running
                       \-> quarantined
running -> disabled -> upgraded/rolled-back/uninstalled
```

Validation covers manifest schema, digest/signature, compatibility, placement, required permissions,
configuration, and conflicts. Enablement grants only the intersection of requested permissions and
administrator-approved policy. Failure health checks trigger bounded restart or quarantine, never an
unbounded crash loop. Safe mode starts Studio with third-party plugins disabled.

Upgrade stages the new package, validates migrations, retains a rollback point, and changes the active
version atomically. Uninstall revokes grants and secret handles before package removal and records
provider-owned data that remains elsewhere.

## Permission and secret model

Plugins start with no ambient authority. They request named capabilities for:

- network destinations and methods;
- resource types, selectors, and actions;
- secret use through opaque handles;
- filesystem roots or native device functions where supported;
- agent/session invocation; and
- storage, CPU, memory, result-size, and duration limits.

The trusted kernel evaluates grants at call time. UI consent is an input to that decision, not the
enforcement mechanism. Secret values should remain inside the secret broker; plugins receive a handle
or brokered request whenever the target protocol permits it. Arguments, results, and errors are
redacted according to declared schemas before audit or diagnostics.

## Distribution

Support three channels in order:

1. local development packages with explicit developer mode;
2. private fleet catalogs controlled by fleet administrators; and
3. an optional public registry with publisher verification and revocation.

Package identity is the reverse-DNS plugin ID plus publisher identity. Versions are immutable.
Compatibility is evaluated against Plugin Spec/schema and required capabilities, not only a Studio
display version. Registry metadata is not trusted in place of the verified package digest.

## SDK and author experience

The public SDK must include:

- versioned manifest schemas and generated Rust/TypeScript types;
- a validator and packaging CLI;
- a local test host with permission and failure simulation;
- contract fixtures for tools, resources, memory, lifecycle, and audit redaction;
- upgrade/rollback and compatibility tests; and
- documentation for every runtime placement and platform limitation.

The `echo` reference plugin proves the smallest complete lifecycle. The `studio-dev` dogfood plugin
then proves a real connector without private hooks.

## Consequences

This decision makes early kernel work larger because authority, lifecycle, and recovery must exist
before a plugin ecosystem. It reduces later core growth, gives users a supported extension path, and
forces first-party features to exercise public contracts.

Remote and instance placements provide immediate cross-platform reach. Local desktop plugins remain
valuable but are not the definition of plugin support. WASM/WASI remains an option, not a prerequisite
or a claim that the OpenAB runtime should run in a browser sandbox.

## Non-goals

- Loading arbitrary shared libraries into the Studio process.
- Giving plugins ambient user, filesystem, environment, or network authority.
- Defining marketplace payment or revenue sharing.
- Allowing installation scripts outside the declared lifecycle.
- Shipping UI extensions before tool/resource/memory boundaries stabilize.
- Treating an MCP server configuration file as a complete Studio plugin package.

## Acceptance criteria

- A third-party author can build the `echo` equivalent from public documentation only.
- The same package can be managed from desktop and mobile when its execution placement is remote.
- Permission denial is enforced by the trusted core and produces a redacted audit event.
- Disable/revocation prevents new calls within a documented bound.
- Failed install, upgrade, migration, or runtime startup has a tested rollback/recovery path.
- Studio starts in safe mode when all third-party plugin code is unavailable.

## Follow-up decisions

Separate decisions are required for package format and signing, registry governance, WASM/WASI host
calls, UI extension isolation, and provider data migration ownership.
