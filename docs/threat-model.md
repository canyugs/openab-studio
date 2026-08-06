# OpenAB Studio Threat Model

- **Status:** Proposed planning contract
- **Planning ID:** A-03 / [GitHub issue #6](https://github.com/canyugs/openab-studio/issues/6)
- **Last updated:** 2026-08-07
- **Related:** [System architecture](./architecture.md), [remote-first client ADR](./adr/remote-first-client.md), and [plugin platform ADR](./adr/plugin-platform.md)

## Purpose and status

This document identifies the high-risk trust transitions in the proposed OpenAB Studio architecture
and gives later work stable threat identifiers to cite. It is a design input for P1 and later
acceptance criteria, not an implementation, penetration-test result, or security certification.

The planned controls and deterministic evidence below are obligations for the work items that
implement them. Their presence here does **not** mean the control exists today or that the listed
evidence has passed. A pull request may claim a threat is addressed only when it links the relevant
control, evidence, and any remaining residual risk in its Review Contract.

### Identifier rules

- **TM-*** identifies one stable high-risk threat. Identifiers are never repurposed; a future split
  keeps the original identifier and adds new child identifiers.
- **PC-*** identifies a planned control contract. It deliberately describes the required security
  property, not a prematurely chosen implementation.
- **EV-*** identifies the minimum deterministic test or review evidence expected from the
  implementing work item.
- **AR-*** identifies an accepted planning residual risk. It is accepted only for the stated
  planning scope and does not authorize a production security claim or a broader rollout.

## Scope

This model covers the Studio client, device shells, trusted Rust core, Fleet Management API and its
embedded or dedicated controller topology, OpenAB instances, ACP-over-WebSocket sessions, MCP/plugin
placements, grants, secret handling, local state, diagnostics, and update channels.

It does not change the OpenAB runtime, choose a concrete authentication protocol, define package
signing or registry governance, or implement any mitigation. Those undecided contracts remain
explicit risks and release gates below.

## Assets

| ID | Asset | Protection objective |
|---|---|---|
| AS-01 | Fleet identity, ownership, membership, policy, and instance registry | Preserve the single-owner Fleet boundary and prevent unauthorized control-plane changes. |
| AS-02 | Principal, device, enrollment, and service identity bindings | Prevent impersonation, unauthorized enrollment, and continued authority after loss or revocation. |
| AS-03 | Resources, grants, roles, memory scope, provider references, and deletion jobs | Preserve scoped authorization, integrity, confidentiality, and accountable lifecycle changes. |
| AS-04 | Fleet Management endpoint/controller identity and durable state | Prevent wrong-Fleet connections, forged inventory/policy state, replayed mutations, and control-plane confusion. |
| AS-05 | ACP session bindings, prompts, events, capability negotiation, and cancellation state | Prevent session hijack, cross-principal reuse, unauthorized reverse capabilities, and dishonest state claims. |
| AS-06 | Plugin packages, manifests, publisher identity, lifecycle state, MCP transports, and placement | Prevent untrusted code or services from receiving more authority than approved. |
| AS-07 | Secret values, opaque secret handles, broker policy, and credential-store references | Keep secret material confidential and correctly scoped. |
| AS-08 | Local caches, databases, encryption metadata, pending mutations, and device preferences | Prevent disclosure, tampering, rollback, replay, and cross-Fleet cache confusion. |
| AS-09 | Audit events and diagnostic/support exports | Preserve attributable, redacted evidence without leaking secrets or another Fleet's data. |
| AS-10 | Studio updater artifacts, plugin catalogs, package metadata, and compatibility state | Prevent supply-chain compromise, unsafe downgrade, and unrecoverable failed updates. |

## Actors, assumed attackers, and trust assumptions

### Legitimate actors

| Actor | Intended authority |
|---|---|
| Fleet user, member, administrator, or owner | Receives only role defaults and explicit grants allowed by Fleet policy. |
| Studio TypeScript UI and device shell | Requests commands and renders state; neither is an authorization authority. |
| Studio Rust core | Trusted kernel for policy mediation, storage, sync, transport, secrets, audit, and plugin lifecycle. |
| Fleet Management endpoint/controller | Authoritative durable inventory, policy, revision, audit, and plugin-placement contract for one Fleet context. |
| OpenAB instance and agent | Own active ACP session context and expose advertised capabilities; they are not the Fleet authorization database. |
| Plugin and plugin publisher | Contribute a declared package and execute only in an approved placement with granted capabilities. |
| Secret broker and platform credential store | Resolve authorized opaque handles without exposing secret values to unrelated callers. |
| Update/catalog publisher and release pipeline | Publish versioned artifacts and metadata that must be verified before activation. |
| Support or diagnostic recipient | Receives only an intentionally exported, redacted bundle. |

### Assumed attackers

| ID | Attacker capability |
|---|---|
| ATT-NET | Can observe, delay, replay, redirect, or modify traffic on an untrusted network. |
| ATT-LOW | Is an authenticated but low-privilege user, agent, service, or plugin attempting to expand authority. |
| ATT-DEV | Possesses a lost, stolen, or previously enrolled device, including an offline device with stale local state. |
| ATT-HOST | Has compromised a desktop or mobile host, its UI process, or its local files, but not necessarily every Fleet endpoint. |
| ATT-PLG | Publishes a malicious package or compromises an otherwise trusted plugin, publisher account, MCP service, or provider. |
| ATT-END | Compromises an OpenAB instance, agent endpoint, or ACP server and can emit arbitrary protocol data for sessions it hosts. |
| ATT-CTRL | Compromises, impersonates, or misconfigures a Fleet Management endpoint/controller. |
| ATT-DIAG | Obtains a diagnostic bundle, audit export, screenshot, or other support artifact outside its intended audience. |
| ATT-SUPPLY | Compromises a catalog, update channel, artifact host, or signing/release path, or attempts a downgrade. |

### Trust assumptions and limits

- The Rust core is the Studio trusted computing base. A compromised Studio binary or updater can
  defeat in-process controls; update verification and recovery therefore protect a critical boundary.
- The UI, local shell, ACP event stream, plugin output, plugin configuration, catalog metadata, and
  diagnostic input are untrusted until the trusted core validates and authorizes their requested
  action.
- An ACP bearer alone is not a sufficient multi-user Fleet identity. As the remote-first ADR states,
  public shared-Fleet use remains gated on later identity, revocation, and authorization work.
- A physically compromised or unlocked device can disclose data already available to it. Revocation
  can bound future authority; it cannot recall copied data or provide an availability guarantee while
  the device is offline.
- An authoritative controller or instance can maliciously alter data within the scope it already
  hosts. Studio must contain that scope, preserve Fleet separation, and support recovery evidence;
  it cannot make a compromised authority truthful by client-side validation alone.
- A plugin or external service can misuse data that it was legitimately granted. Least privilege,
  placement, and auditing reduce that exposure but do not transform an authorized recipient into a
  trusted data processor.
- Desktop and mobile share management authority semantics. Mobile does not execute arbitrary local
  child processes, and device execution capability must never be mistaken for user authorization.

## Trust boundaries and architectural invariants

| Boundary | Transition | Required interpretation |
|---|---|---|
| TB-01 | TypeScript UI or device shell to Rust core | Presentation can request an operation but cannot mint a grant, unwrap a secret, or mark a mutation durable. |
| TB-02 | User/device enrollment to local OS credentials and physical device state | Device identity, local credential storage, device loss, and re-enrollment require an explicit lifecycle. |
| TB-03 | Rust core to Fleet Management endpoint/controller | This is the durable management-plane boundary for Fleet identity, policy, revisions, and audit. Embedded and dedicated controller topologies use the same contract. |
| TB-04 | Fleet controller to OpenAB instance | Inventory, registration, health, and placement observations cross from durable Fleet policy to an instance runtime. |
| TB-05 | Rust core to OpenAB instance over ACP-over-WebSocket | Live prompts, session/resume state, events, and explicitly granted reverse MCP capabilities cross this interactive boundary. |
| TB-06 | Rust core or plugin host to a plugin through MCP or another declared runtime | Package validation, placement, lifecycle, capability enforcement, and plugin outputs cross here. |
| TB-07 | Plugin to its remote or instance-hosted service and external network | The plugin's declared destination, service identity, and brokered capabilities limit this untrusted dependency. |
| TB-08 | Rust core to secret broker and platform credential store | Only an authorized, context-bound secret operation may cross; raw secret values should not. |
| TB-09 | Rust core to local database, cache, migration state, and backups | Locally retained state must remain Fleet-scoped, revisioned, recoverable, and protected at rest. |
| TB-10 | Audit or diagnostic state to a support/export recipient | Export is a disclosure boundary with redaction, audience, and provenance requirements. |
| TB-11 | Catalog, plugin package, or Studio update channel to the installed client | Metadata is not trusted in place of verified identity, digest, compatibility, and recovery checks. |
| TB-12 | Fleet switcher or aggregate view across Fleet contexts | A multi-Fleet display is an aggregation of separate authorization contexts, never a merged authorization domain. |

The following architecture invariants are security-critical:

1. **INV-FLT-01 — one owning Fleet.** An OpenAB instance belongs to at most one owning Fleet. It may
   be temporarily unassigned only while enrolled or moved. A move preflights conflicts, revokes
   source authority, invalidates source bindings and caches, issues destination authority before
   activation, and audits the outcome. It does not silently copy Fleet-owned data.
2. **INV-FLT-02 — context-complete state.** Fleet and resource identity appear in cache keys, session
   bindings, grants, audit attribution, and reconciliation. An aggregate view evaluates each Fleet
   separately.
3. **INV-AUTH-01 — grants remain authoritative.** Roles provide Fleet-level defaults but do not
   bypass resource grants, secret policy, placement restrictions, expiry, or revocation.
4. **INV-CORE-01 — the core enforces high-impact operations.** TypeScript, plugins, and ACP events
   cannot independently authorize a mutation, secret operation, or durable success state.
5. **INV-PLG-01 — no ambient plugin authority.** Plugins receive explicit capability handles, not
   ambient user, filesystem, network, environment, or secret access.
6. **INV-SEC-01 — secrets stay opaque.** Secret values do not enter transcripts, diagnostics, plugin
   configuration, or audit bodies.
7. **INV-PLAT-01 — management parity is not execution parity.** Desktop-only local executable
   placement must fail closed on mobile; both platform families can manage an approved remote or
   instance placement.

## Planned control contracts

| Control | Planned security property |
|---|---|
| PC-DEV-01 | Enrollment binds a device identity, principal, and Fleet context through an explicit approval/lifecycle operation, records the binding, and rejects replay or substitution. |
| PC-DEV-02 | Device loss, logout, removal, and re-enrollment invalidate device-bound authority with a documented enforcement bound and do not rely on the UI to enforce it. |
| PC-FLT-01 | The authoritative registry enforces the one-owner invariant and the architecture's audited move sequence as a state transition, not as a best-effort UI workflow. |
| PC-FLT-02 | Every Fleet-scoped identifier, cache record, grant, ACP binding, event cursor, and audit record is context-complete; aggregate views never union authority. |
| PC-MGT-01 | A management profile verifies the expected endpoint/controller identity and Fleet binding before accepting durable state or mutating it. |
| PC-MGT-02 | Management mutations carry actor/Fleet context, expected revision, and idempotency identity; stale, replayed, or conflicting writes fail without being presented as durable. |
| PC-MGT-03 | Controller compromise containment enforces every action in its explicit Fleet context, records attributable events, and provides a defined rebind/recovery path rather than silently trusting a replacement endpoint. |
| PC-ACP-01 | ACP session creation, resume, and reverse capability use bind the authenticated principal, device, Fleet, instance, session, and negotiated capabilities. |
| PC-ACP-02 | ACP session and reverse-capability use revalidate effective grant, expiry, device, plugin, and revocation state at documented enforcement points. |
| PC-ACP-03 | ACP events, prompts, and plugin results are untrusted input; they cannot cause a management mutation or secret operation without a separate core authorization decision. |
| PC-PLG-01 | Plugin validation verifies manifest schema, package identity, digest/signature when the contract is available, compatibility, placement, requested permissions, and configuration before enablement. |
| PC-PLG-02 | Enablement grants only the policy-approved intersection of requested capabilities; invocation-time enforcement, resource limits, quarantine, rollback, and safe mode contain a plugin failure or abuse attempt. |
| PC-PLG-03 | Placement is explicit and enforced: remote/instance plugins are managed from all devices, local mcp-stdio is desktop-only, and native device providers expose only narrow reviewed APIs. |
| PC-GRT-01 | The trusted core or authoritative endpoint evaluates issuer, principal, action, resource selector, conditions, expiry, and revocation for each protected operation. |
| PC-GRT-02 | Revocation, plugin disable, Fleet removal/move, and device loss invalidate affected grants, caches, sessions, and secret handles within a documented bound. |
| PC-SEC-01 | The secret broker supplies brokered operations or opaque handles; declared schema redaction prevents raw values from reaching plugins, configuration, logs, audits, transcripts, or diagnostics. |
| PC-SEC-02 | Every secret handle is bound to its Fleet, principal, plugin/process or endpoint, target operation, and validity period; it cannot be replayed in another context. |
| PC-STO-01 | Local persistence minimizes retained sensitive data and uses the platform/storage protection selected by the later storage and credential decisions; no raw secret is cached by default. |
| PC-STO-02 | Local cache and database state are revisioned and Fleet-scoped; tampered, stale, failed-migration, or replayed state is detected and recovered without turning a local optimistic write into authority. |
| PC-DAT-01 | Memory/provider access carries Fleet, scope, provenance, retention, grant, and deletion-job state; local cache removal is not represented as provider deletion. |
| PC-DIA-01 | Diagnostic and support export follows declared redaction and audience policy, preserves provenance, and makes the export scope visible before release. |
| PC-AUD-01 | The core produces attributable, redacted audit records for high-impact operations; audit identifiers preserve Fleet and actor context even when an operation fails. |
| PC-UPD-01 | Studio updates and plugin updates verify trusted artifact identity, digest, compatibility, and rollout channel before activation and retain a safe rollback/recovery path. |
| PC-AVL-01 | Plugin, endpoint, and untrusted-input failures receive bounded resource/lifecycle behavior, with quarantine or safe mode rather than an unbounded restart loop. |

## High-risk threat catalogue

Every threat in this catalogue is high risk. The evidence is intentionally phrased as a deterministic
fixture, state-machine test, or recorded review artifact so a P1 issue can cite the threat without
repeating its entire rationale.

| Threat | Abuse case and affected boundary | Planned control | Required deterministic evidence | Residual risk or release gate |
|---|---|---|---|---|
| TM-DEV-01 | An attacker enrolls a substitute device, replays an enrollment artifact, or binds a valid device to the wrong Fleet. Affects AS-01, AS-02, and TB-02/TB-03. | PC-DEV-01, PC-MGT-01, PC-FLT-02 | EV-DEV-01: an enrollment fixture rejects a reused artifact, wrong principal, wrong Fleet, and substituted device, and proves no profile, grant, or audit success was created. | No residual acceptance; required before shared-Fleet device enrollment. |
| TM-DEV-02 | A lost, stolen, or offline device continues to use a cached grant, ACP session, or secret handle after loss/removal. Affects AS-02, AS-05, AS-07, and TB-02/TB-05/TB-08. | PC-DEV-02, PC-GRT-02, PC-ACP-02, PC-SEC-02 | EV-DEV-02: remove a device during a fixture session; after the documented enforcement bound, new management calls, resume, reverse capabilities, and handle use are denied. | AR-DEV-01 |
| TM-DEV-03 | A compromised desktop/mobile UI or local host bypasses presentation checks, reads stored state, or invokes a core command with another principal's context. Affects AS-02, AS-07, AS-08, and TB-01/TB-09. | PC-DEV-02, PC-GRT-01, PC-SEC-01, PC-STO-01 | EV-DEV-03: direct command-boundary fixtures bypass the UI and demonstrate deny-by-default for a mismatched principal, device, Fleet, and secret context. | AR-DEV-01 |
| TM-FLT-01 | A race or malformed move creates simultaneous ownership in two Fleets or activates the destination before source authority is revoked. Affects AS-01 and TB-03/TB-04/TB-12. | PC-FLT-01, PC-FLT-02, PC-GRT-02 | EV-FLT-01: property/state-machine tests exercise enroll, move, failure, retry, and concurrent move paths and assert zero or one active owner plus the required audit sequence. | No residual acceptance; this is the accepted Fleet ownership invariant. |
| TM-FLT-02 | Cache keys, session bindings, audit attribution, or aggregate UI confuse equal-looking identifiers from two Fleets and disclose or mutate the wrong context. Affects AS-01, AS-03, AS-05, AS-08, AS-09, and TB-09/TB-12. | PC-FLT-02, PC-MGT-01, PC-GRT-01 | EV-FLT-02: a two-Fleet fixture with colliding resource and agent display identifiers proves reads, mutations, cache lookup, audit, and aggregate rendering remain Fleet-scoped. | No residual acceptance. |
| TM-MGT-01 | A client accepts an impersonated, misconfigured, or wrong-Fleet management endpoint/controller and trusts its inventory or policy. Affects AS-01, AS-04, and TB-03. | PC-MGT-01, PC-FLT-02 | EV-MGT-01: profile fixtures reject an endpoint identity/Fleet mismatch before accepting events or applying a durable mutation. | AR-CTRL-01 for a controller that is genuinely compromised within its own Fleet. |
| TM-MGT-02 | A network or low-privilege actor replays a management mutation, races a stale revision, or causes a conflict to appear successful locally. Affects AS-01, AS-03, AS-04, and TB-03. | PC-MGT-02, PC-GRT-01, PC-STO-02 | EV-MGT-02: replay, stale-revision, and duplicate-idempotency fixtures prove one authoritative result, deterministic conflict state, and no false durable-success UI state. | No residual acceptance. |
| TM-MGT-03 | A compromised controller issues arbitrary in-Fleet policy changes, fabricates observations, or submits an action bound to one Fleet against another Fleet's records. Affects AS-01, AS-04, AS-09, and TB-03/TB-04/TB-12. | PC-MGT-03, PC-FLT-02, PC-AUD-01 | EV-MGT-03: a two-Fleet controller fixture proves an action bound to Fleet A cannot address Fleet B's records; recovery review records how a replaced controller is re-bound and audited. | AR-CTRL-01 |
| TM-ACP-01 | A stolen bearer, confused resume reference, or wrong endpoint binds an ACP session to a different user, device, Fleet, instance, or capability set. Affects AS-02, AS-05, and TB-05. | PC-ACP-01, PC-MGT-01, PC-FLT-02 | EV-ACP-01: connection and resume fixtures reject every mismatched principal, device, Fleet, instance, and negotiated-capability combination. | AR-ACP-01 |
| TM-ACP-02 | A revoked grant, disabled plugin, lost device, or Fleet move leaves an existing ACP session or reverse MCP capability usable. Affects AS-03, AS-05, AS-06, and TB-05/TB-06. | PC-ACP-02, PC-GRT-02, PC-DEV-02 | EV-ACP-02: revoke each source of authority during an active fixture; subsequent prompt, resume, cancel, and reverse-capability paths show the documented denied or terminated state. | AR-DEV-01 for previously copied session data. |
| TM-ACP-03 | A compromised instance/agent emits a malicious ACP event or result that tricks Studio into granting access, running a plugin, changing management state, or disclosing a secret. Affects AS-03, AS-05, AS-07, and TB-05/TB-08. | PC-ACP-03, PC-GRT-01, PC-SEC-01 | EV-ACP-03: adversarial event fixtures attempt a core command and reverse capability without the required grant and prove denial plus redacted attribution. | AR-END-01 |
| TM-PLG-01 | A tampered, incompatible, or falsely described package/catalog entry is installed or enabled, including a package update that changes its requested authority. Affects AS-06, AS-10, and TB-06/TB-11. | PC-PLG-01, PC-UPD-01 | EV-PLG-01: a manifest/package corpus rejects altered digest, invalid required field, incompatible schema, placement mismatch, and unapproved permission expansion. | AR-PLG-02 |
| TM-PLG-02 | A malicious plugin obtains ambient filesystem, environment, network, user, resource, or secret authority beyond its approved grant. Affects AS-03, AS-06, AS-07, and TB-06/TB-08. | PC-PLG-02, PC-GRT-01, PC-SEC-01 | EV-PLG-02: a plugin test host attempts each undeclared capability directly and proves core denial, no secret value exposure, and a redacted audit record. | AR-PLG-01 |
| TM-PLG-03 | A compromised remote or instance-hosted MCP plugin/service exfiltrates data, returns adversarial output, or reuses authority from another Fleet/session. Affects AS-03, AS-05, AS-06, AS-07, and TB-06/TB-07. | PC-PLG-02, PC-PLG-03, PC-SEC-02, PC-ACP-03 | EV-PLG-03: malicious remote and instance MCP fixtures attempt cross-Fleet handle use, oversized/adversarial output, and ungranted reverse invocation; each is denied or bounded. | AR-PLG-01 |
| TM-PLG-04 | A desktop local mcp-stdio plugin escapes its reviewed host boundary, inherits Studio credentials, or persists after disable/quarantine. Affects AS-06, AS-07, AS-08, and TB-06/TB-08/TB-09. | PC-PLG-02, PC-PLG-03, PC-SEC-01, PC-GRT-02 | EV-PLG-04: launcher fixtures prove no Studio credential is available without a brokered grant and prove disable/quarantine prevents new process capability use within its bound. | AR-DEV-01 if the desktop host itself is compromised. |
| TM-PLG-05 | A phone/tablet treats local executable placement as manageable execution, silently routes it through an unintended host, or mislabels a placement failure as an authorization grant. Affects AS-06 and TB-01/TB-06. | PC-PLG-03 | EV-PLG-05: platform capability fixtures reject device mcp-stdio on mobile while allowing management of a remote/instance plugin and distinguish unsupported execution from denied authority. | No residual acceptance. |
| TM-GRT-01 | A principal or role grants itself access, uses an overbroad selector, bypasses conditions, or receives implicit full user authority through an agent/plugin. Affects AS-01, AS-03, AS-07, and TB-01/TB-03/TB-06. | PC-GRT-01, PC-MGT-02, PC-FLT-02 | EV-GRT-01: table-driven evaluator tests cover issuer, action, selector, condition, expiry, role default, agent, plugin, and cross-Fleet denial cases. | No residual acceptance. |
| TM-GRT-02 | Revocation, expiry, disablement, device loss, or Fleet move races with cached grants, handles, sessions, or plugin work and leaves authority usable. Affects AS-02, AS-03, AS-05, AS-06, AS-07, and TB-03/TB-05/TB-06/TB-08. | PC-GRT-02, PC-ACP-02, PC-DEV-02, PC-SEC-02 | EV-GRT-02: deterministic revocation-race tests verify each affected cache, session, process, and handle is denied or terminated by the documented bound. | AR-DEV-01 for information already disclosed before enforcement. |
| TM-SEC-01 | A secret value leaks through a plugin configuration, MCP argument/result, ACP transcript, audit event, log, diagnostic export, local cache, or error. Affects AS-07, AS-08, AS-09, and TB-06/TB-08/TB-09/TB-10. | PC-SEC-01, PC-DIA-01, PC-AUD-01, PC-STO-01 | EV-SEC-01: seed a unique canary secret and assert it is absent from generated transcript, audit, plugin config, log, cache, error, and diagnostic fixtures. | No residual acceptance. |
| TM-SEC-02 | A valid opaque secret handle is replayed from another Fleet, principal, plugin/process, endpoint, session, target, or expiry window. Affects AS-01, AS-02, AS-07, and TB-03/TB-06/TB-08. | PC-SEC-02, PC-FLT-02, PC-GRT-01 | EV-SEC-02: handle-misuse fixtures vary one binding dimension at a time and prove every mismatch fails without revealing the value. | No residual acceptance. |
| TM-STO-01 | Local cache/database files, encryption metadata, or device backups disclose raw secrets or another Fleet's retained sensitive state. Affects AS-01, AS-07, AS-08, and TB-02/TB-09. | PC-STO-01, PC-FLT-02, PC-SEC-01 | EV-STO-01: inspect a deterministic local-state fixture and backup path for the canary secret, then prove separate Fleets cannot resolve each other's cache records. | AR-STO-01 |
| TM-STO-02 | An attacker tampers with, rolls back, or replays local state; a failed migration leaves authoritative-looking or cross-Fleet data. Affects AS-01, AS-03, AS-08, and TB-09/TB-12. | PC-STO-02, PC-MGT-02, PC-FLT-02 | EV-STO-02: modified, stale, replayed, and failed-migration fixtures produce detected conflict/recovery states and never create an unreviewed durable mutation. | AR-STO-01 |
| TM-DAT-01 | A memory provider, cache, or deletion job exposes data across personal/Fleet/workspace scope, loses provenance, or reports cache removal as provider deletion. Affects AS-03, AS-08, and TB-06/TB-07/TB-09. | PC-DAT-01, PC-GRT-01, PC-FLT-02 | EV-DAT-01: scoped provider fixtures deny cross-Fleet/scope reads and require a terminal provider deletion result before marking data deleted. | AR-PLG-01 when the provider is a legitimately authorized third party. |
| TM-DIA-01 | A diagnostic or support export contains secrets, unredacted prompts, raw credentials, cross-Fleet metadata, or hidden data that its recipient was not approved to receive. Affects AS-05, AS-07, AS-08, AS-09, and TB-10. | PC-DIA-01, PC-SEC-01, PC-AUD-01, PC-FLT-02 | EV-DIA-01: generate an adversarial two-Fleet support bundle with canary secrets and assert redaction, visible scope, correct provenance, and no other-Fleet record. | AR-DIA-01 |
| TM-AUD-01 | A high-impact mutation has no attributable audit record, is attributed to the wrong Fleet/principal, or an export hides the fact that data was redacted/omitted. Affects AS-01, AS-03, AS-09, and TB-03/TB-10/TB-12. | PC-AUD-01, PC-FLT-02, PC-MGT-03 | EV-AUD-01: success, denial, conflict, revoke, move, disable, and export fixtures emit one correctly scoped, redacted audit record with a correlation identifier. | AR-CTRL-01 for audit state controlled by a compromised authoritative controller. |
| TM-UPD-01 | A compromised Studio updater, catalog, package host, or downgrade path installs untrusted code or an incompatible schema without recovery. Affects AS-06, AS-08, AS-10, and TB-11. | PC-UPD-01, PC-PLG-01, PC-STO-02 | EV-UPD-01: tampered artifact, wrong channel, incompatible schema, failed migration, and prohibited downgrade fixtures are rejected or safely rolled back. | AR-UPD-01 |
| TM-END-01 | A compromised OpenAB endpoint reads session data it hosts, lies about state, or uses its session position to cause unauthorized follow-on actions. Affects AS-03, AS-05, AS-07, and TB-04/TB-05. | PC-ACP-03, PC-GRT-01, PC-SEC-01, PC-MGT-01 | EV-END-01: a malicious endpoint fixture returns arbitrary events/results and attempts reverse MCP and management actions; no ungranted action or secret disclosure succeeds. | AR-END-01 |
| TM-AVL-01 | A plugin, endpoint, update, or adversarial input creates resource exhaustion, crash loops, stuck mutations, or loss of safe recovery. Affects AS-04, AS-05, AS-06, AS-08, and TB-03/TB-05/TB-06/TB-11. | PC-AVL-01, PC-PLG-02, PC-UPD-01 | EV-AVL-01: deterministic crash, hang, oversized-result, retry, and failed-update fixtures prove bounded behavior, quarantine/safe mode, and a recoverable status. | AR-AVL-01 |

## Accepted planning residual risks

These are deliberately visible rather than implicit. They are accepted only as limitations of the
current planning scope; a release owner must re-evaluate them when the named gate becomes relevant.

| Residual | Accepted planning limitation | Required gate before a broader claim |
|---|---|---|
| AR-ACP-01 | Current ACP transport authentication is insufficient by itself for public shared-Fleet identity. | P3 identity, authorization, and revocation contract plus shared-Fleet conformance evidence. |
| AR-DEV-01 | A compromised, unlocked, or offline device may expose data it already possesses; revocation cannot recall copied information. | Device lifecycle, local-storage, session, and enforcement-bound evidence for the intended platform tier. |
| AR-CTRL-01 | A compromised authoritative controller can lie or act within a Fleet context it is authorized to control. Client checks cannot prove that controller truthful. | Endpoint identity, controller recovery/rebind, audit-retention, and operational incident-response decisions. |
| AR-END-01 | A compromised OpenAB instance can observe or manipulate the ACP content it hosts. Studio can prevent extra authority, not make that endpoint trustworthy. | Clear endpoint trust UX and the relevant OpenAB-side identity/session contract. |
| AR-PLG-01 | A plugin/service can misuse information it was legitimately granted. Capability minimization and audit limit exposure but cannot revoke copied data. | Per-plugin policy review, least-privilege grant evidence, and user/admin approval flow. |
| AR-PLG-02 | Package signing, publisher verification, and registry governance are intentionally undecided. | Dedicated signing/registry ADR, validation corpus, and release-channel implementation before a public catalog claim. |
| AR-STO-01 | Exact encrypted local-storage, backup, recovery, and platform credential behavior is still an open decision. | Storage/credential ADR or implementation decision plus per-platform at-rest and failed-migration evidence. |
| AR-DIA-01 | A user can intentionally export redacted data and an external recipient can retain it; Studio cannot recall a deliberately shared artifact. | Export scope/review UX and documented support handling before support-bundle release claims. |
| AR-UPD-01 | Per-platform updater signing, anti-downgrade, and rollback policy are not yet chosen. | Release/update ADR or contract, signed artifact tests, and per-platform rollback evidence. |
| AR-AVL-01 | No client can guarantee availability against network partition, a fully compromised endpoint, or physical device loss. | Documented bounded behavior, recovery posture, and platform-specific release evidence. |

## P1 and later traceability

Future issues and pull requests should cite these IDs instead of restating the threat model. The
listed work item owns the first expected proof; it does not imply that another work item may bypass
the same control.

| Work item from workstreams | Threats that must be considered | Minimum evidence to cite |
|---|---|---|
| C-01 studio-core command/event boundary | TM-DEV-03, TM-MGT-02, TM-ACP-03, TM-END-01 | EV-DEV-03, EV-MGT-02, EV-ACP-03, EV-END-01 |
| C-02 local database and migration harness | TM-STO-01, TM-STO-02, TM-FLT-02 | EV-STO-01, EV-STO-02, EV-FLT-02 |
| C-03 secret broker | TM-SEC-01, TM-SEC-02, TM-PLG-02, TM-PLG-04 | EV-SEC-01, EV-SEC-02, EV-PLG-02, EV-PLG-04 |
| C-04 Fleet/instance profile registry and capability cache | TM-FLT-01, TM-FLT-02, TM-MGT-01, TM-DEV-02 | EV-FLT-01, EV-FLT-02, EV-MGT-01, EV-DEV-02 |
| C-05 ACP-over-WebSocket state machine and C-06 session behavior | TM-ACP-01, TM-ACP-02, TM-ACP-03, TM-END-01 | EV-ACP-01, EV-ACP-02, EV-ACP-03, EV-END-01 |
| G-01 grant evaluator | TM-GRT-01, TM-GRT-02, TM-DEV-03, TM-DAT-01 | EV-GRT-01, EV-GRT-02, EV-DEV-03, EV-DAT-01 |
| G-02 audit schema and redaction | TM-AUD-01, TM-DIA-01, TM-SEC-01, TM-MGT-03 | EV-AUD-01, EV-DIA-01, EV-SEC-01, EV-MGT-03 |
| P-01 plugin manifest validator | TM-PLG-01, TM-PLG-05, TM-UPD-01 | EV-PLG-01, EV-PLG-05, EV-UPD-01 |
| P-02 plugin lifecycle and safe mode | TM-PLG-02, TM-PLG-03, TM-GRT-02, TM-AVL-01 | EV-PLG-02, EV-PLG-03, EV-GRT-02, EV-AVL-01 |
| P-03 public SDK/test host and P-04 echo plugin | TM-PLG-01, TM-PLG-02, TM-SEC-01 | EV-PLG-01, EV-PLG-02, EV-SEC-01 |
| U-02 ACP UI and U-03 management UI | TM-DEV-03, TM-ACP-02, TM-FLT-02, TM-PLG-05 | Direct-command/negative fixtures above plus platform-visible denied, conflicted, and unsupported states. |
| P3 Fleet identity/controller work | TM-DEV-01, TM-DEV-02, TM-MGT-01, TM-MGT-03, TM-FLT-01, TM-FLT-02 | EV-DEV-01, EV-DEV-02, EV-MGT-01, EV-MGT-03, EV-FLT-01, EV-FLT-02 |
| P2 D-03 desktop mcp-stdio host | TM-PLG-04, TM-GRT-02, TM-SEC-01 | EV-PLG-04, EV-GRT-02, EV-SEC-01 |
| P2 D-04/D-06 release and updater work | TM-UPD-01, TM-AVL-01, TM-STO-02 | EV-UPD-01, EV-AVL-01, EV-STO-02 |
| P2 D-05 diagnostics | TM-DIA-01, TM-AUD-01, TM-SEC-01 | EV-DIA-01, EV-AUD-01, EV-SEC-01 |
| P4 memory/provider work | TM-DAT-01, TM-GRT-01, TM-FLT-02 | EV-DAT-01, EV-GRT-01, EV-FLT-02 |

## Review checklist

Use this checklist when a P1 or later issue implements a control from this model:

- [ ] The issue/PR names every applicable TM identifier and the PC identifiers it claims to
  implement; new abuse cases receive a new TM identifier rather than being hidden in prose.
- [ ] The Review Contract's acceptance section links the required EV fixture, test, or recorded
  platform evidence, including negative deny/revoke/redaction behavior.
- [ ] The change preserves INV-FLT-01 and INV-FLT-02, including two-Fleet collision and move
  fixtures wherever it stores, displays, binds, or audits Fleet-scoped state.
- [ ] The implementation authorizes high-impact actions in the Rust core or authoritative endpoint,
  not only in TypeScript, a plugin, an ACP event handler, or a local UI state transition.
- [ ] Plugin work proves declared placement and permission intersection; mobile capability checks
  distinguish unsupported local execution from a denied user grant.
- [ ] Secret, audit, diagnostic, cache, transcript, and error paths include the relevant canary
  redaction evidence when they can carry sensitive data.
- [ ] Revocation-related work states its enforcement bound and tests device loss, plugin disable,
  grant expiry/revoke, or Fleet movement against cached/session/handle state.
- [ ] Any unchanged AR identifier is repeated under the PR Review Contract's accepted residual
  risks. A broader residual risk requires maintainer review and a follow-up issue, not a silent
  expansion of scope.
- [ ] Release, updater, controller, and platform claims are limited to evidence actually recorded;
  this threat model alone is never cited as a certification.

## Coverage check

| Required issue area | Primary threats and boundaries |
|---|---|
| Device enrollment and loss | TM-DEV-01 through TM-DEV-03; TB-01, TB-02, TB-03, TB-05, TB-08, TB-09 |
| Fleet Management endpoint/controller | TM-MGT-01 through TM-MGT-03; TB-03, TB-04, TB-12 |
| Single-owner Fleet and cross-Fleet isolation | TM-FLT-01, TM-FLT-02; INV-FLT-01, INV-FLT-02; TB-03, TB-09, TB-12 |
| ACP sessions and compromised endpoints | TM-ACP-01 through TM-ACP-03, TM-END-01; TB-05 |
| MCP and every plugin placement | TM-PLG-01 through TM-PLG-05; TB-06, TB-07, TB-11 |
| Grants, revocation, and secret broker | TM-GRT-01, TM-GRT-02, TM-SEC-01, TM-SEC-02; TB-03, TB-05, TB-06, TB-08 |
| Local caches/databases and memory/provider state | TM-STO-01, TM-STO-02, TM-DAT-01; TB-06, TB-07, TB-09 |
| Diagnostics, audit, and update channels | TM-DIA-01, TM-AUD-01, TM-UPD-01; TB-10, TB-11 |
| Desktop/mobile differences and compromised hosts | TM-DEV-03, TM-PLG-04, TM-PLG-05; INV-PLAT-01; TB-01, TB-02, TB-06, TB-09 |
| Availability and recovery | TM-AVL-01; TB-03, TB-05, TB-06, TB-11 |
