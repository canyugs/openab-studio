# ADR: Cross-Platform, Remote-First OpenAB Studio Client

- **Status:** Proposed
- **Date:** 2026-08-07
- **Related:** [System architecture](../architecture.md),
  [Plugin platform](./plugin-platform.md),
  [ACP Server over WebSocket — Base](https://github.com/openabdev/openab/blob/main/docs/adr/acp-server-websocket-base.md)

## Context

OpenAB Studio must manage local or remote OpenAB instances across macOS, Windows, Linux, iPadOS,
iOS, and Android. It is a client and control surface, not a required copy of the OpenAB runtime.

The product also needs multi-fleet governance, resources and grants, memory, plugins, audit, and
cross-device state. ACP-over-WebSocket is the key portable session transport, but it is not a durable
fleet-management protocol. Mobile operating systems also cannot be assumed to execute arbitrary
local plugins or agent CLIs.

The browser-hosted Web App was considered initially and is now paused. The shared UI may remain based
on web technology inside native shells, but active releases target installed applications.

## Decision

Build Studio as a remote-first installed client with:

- a Rust `studio-core` owning storage, sync, transport, policy, secrets, audit, and plugin lifecycle;
- a TypeScript adaptive UI owning presentation and ephemeral interaction state;
- Tauri 2 shells for desktop and mobile;
- a Fleet Management API for durable management state and mutations;
- ACP-over-WebSocket for live agent sessions; and
- MCP, mediated by the Plugin Spec, for plugin tools and data.

The first releases connect to existing OpenAB instances. Studio does not bundle, install, or own an
OpenAB runtime by default. A later provisioning plugin may offer that workflow without changing the
client/instance boundary.

### Why the trusted core is Rust

The cross-platform core has responsibilities that must behave consistently on every device: database
migrations, protocol state machines, authorization checks, audit generation, secret-handle mediation,
plugin validation, and recovery. Keeping them in one Rust core avoids reimplementing security
decisions in each UI or device shell. Generated bindings expose narrow commands and events to the
TypeScript UI.

### Why Tauri 2

Tauri 2 supports the target desktop and mobile families while allowing a shared TypeScript UI and
Rust core. It matches existing Rust expertise and provides a narrow command boundary for native
integration. The choice accepts that platform WebViews, packaging, signing, background lifecycle,
and store distribution still require independent OS evidence.

Electron remains a desktop fallback if Chromium-identical rendering or extensive local process
supervision later dominates the product. Flutter or separate native applications remain alternatives
if Tauri mobile evidence fails a P0 spike; they are not the default architecture.

## Protocol responsibilities

| Concern | Contract |
|---|---|
| Fleet/instance inventory, users, devices | Fleet Management API |
| Resources, roles, grants, audit | Fleet Management API |
| Plugin placement, configuration, rollout | Fleet Management API + Plugin Spec |
| Memory catalog, governance, provider jobs | Fleet Management API + provider SPI |
| Interactive agent session | ACP-over-WebSocket |
| Plugin tools and provider data | MCP through the plugin host |
| OS secrets and native device functions | Rust core and narrow Tauri host commands |

ACP capability negotiation remains authoritative for session features. Studio does not feature-gate
by parsing an OpenAB version string.

## Client and instance boundary

Studio stores connection profiles, device state, offline caches, and display state. The fleet or
provider remains authoritative for shared policy and provider-backed data. The OpenAB instance owns
the active agent session context unless a future history service defines another contract.

Current ACP limitations must be represented honestly:

- resume can continue context without replaying a transcript;
- cancellation may stop the client waiter without stopping downstream agent work;
- rich structured events may not be available; and
- a transport bearer is not a sufficient multi-user fleet identity.

Public shared-fleet use therefore depends on P3 identity, revocation, and authorization work. Private
development can use the current OpenAB transport authentication within its documented trust boundary.

## Platform behavior

Desktop and mobile expose the same management capabilities. Their execution placements differ:

- desktop can host reviewed local `mcp-stdio` plugins and narrow native providers;
- mobile manages remote and instance-hosted plugins but does not execute arbitrary child processes;
- both can run ACP sessions and all Fleet Management API operations; and
- WASM/WASI is a later placement only if host-call permissions and lifecycle work consistently across
  target systems.

The UI adapts density, navigation, approvals, and interruption recovery to the device. It does not
label a feature absent merely because the current device cannot execute that plugin locally.

## Consequences

Positive consequences:

- one security kernel and data model serve all installed platforms;
- remote OpenAB instances remain first-class;
- mobile can provide complete management without pretending to be a desktop runtime;
- the UI, core, instance, and plugin ecosystem can evolve through versioned contracts; and
- a managed-local experience can be added by a provisioning plugin later.

Costs and risks:

- Tauri support does not remove OS-specific release engineering;
- the Rust/TypeScript binding surface requires disciplined schema generation and compatibility tests;
- a new Fleet Management API/control-plane capability must be designed and implemented;
- offline mutation and multi-device conflicts require durable revisions and reconciliation; and
- cross-device conversation history remains a separate ownership decision.

## Non-goals

- Shipping a browser-hosted Web App in the active roadmap.
- Replacing existing Discord, Slack, LINE, or other OpenAB adapters.
- Bundling an OpenAB instance in the first release.
- Treating ACP as the fleet database or authorization API.
- Executing arbitrary local binaries on mobile.
- Claiming identical packaging or lifecycle behavior without per-platform verification.

## Acceptance criteria

- P0 spikes prove or reject the chosen shell on every target OS family.
- Rust and TypeScript pass the same versioned contract fixtures.
- One desktop build connects to an existing OpenAB instance and completes an ACP session.
- One mobile build reads and mutates a fixture fleet through the management contract.
- Authorization decisions and secret access occur in Rust or the authoritative fleet endpoint, never
  only in TypeScript presentation code.
- Device capability negotiation distinguishes platform execution support from user authorization.

## Follow-up decisions

Separate ADRs are required for Fleet Management API encoding and controller topology, cross-device
history, encrypted local storage, identity federation, and release/support tiers.
