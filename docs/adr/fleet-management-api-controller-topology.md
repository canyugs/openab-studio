# ADR: Fleet Management API and Controller Topology

- **Status:** Proposed
- **Date:** 2026-08-07
- **Tracking:** [A-02 / issue #4](https://github.com/canyugs/openab-studio/issues/4)
- **Related:** [System architecture](../architecture.md),
  [Cross-platform remote-first client](./remote-first-client.md),
  [Public plugin platform](./plugin-platform.md)

## Context

Studio manages fleets of OpenAB instances, but an ACP connection is an interactive, live-session
transport rather than a durable source of fleet state. The accepted fleet invariant is that an
OpenAB instance has at most one owning Fleet at a time. It may be unassigned while being enrolled or
moved, but Studio's multi-Fleet view does not create simultaneous membership.

The first OpenAB ACP endpoint is useful for private, authenticated sessions, but its transport
bearer and synthetic ACP sender are not a multi-user Fleet identity. MCP is likewise a plugin
tool/data plane, not an inventory, grants, audit, or lifecycle API. Treating either protocol as the
Fleet Management API would make identity propagation, revocation, offline reconciliation, and audit
attribution ambiguous.

Simple installations need a low-operations path while multi-instance fleets need a controller that
can survive an individual OpenAB instance being replaced. The client must not have different
semantics, authorization, or recovery rules merely because a controller is embedded in an OpenAB
instance instead of deployed separately.

## Decision

Adopt a versioned, durable Fleet Management API as the authoritative management plane for a Fleet.
Its initial wire profile is HTTPS JSON at the versioned management root, with JSON pages and an
SSE-compatible event stream for synchronization:

- management root: /fleet-management/v1alpha1;
- representation version: studio.openab.dev/fleet-management/v1alpha1;
- mutations: authenticated HTTPS requests with durable operation records;
- read synchronization: revisioned snapshots plus replayable, opaque event cursors; and
- live events: Server-Sent Events (SSE) using the same cursor semantics as JSON catch-up pages.

The exact shared compatibility matrix and deprecation policy remain owned by
[A-04 / issue #3](https://github.com/canyugs/openab-studio/issues/3). This ADR fixes the initial
management-plane boundary, mandatory safety behavior, and fixture shapes; A-04 must not weaken
required-field rejection, revision preconditions, or authorization checks.

An **OpenAB instance** is a deployed OpenAB runtime. An **ACP server** is only a protocol role that
an OpenAB instance may expose; it is not a synonym for an OpenAB instance or a Fleet controller.

### Durable management-plane boundary

The Fleet Management API is the authority for:

- Fleet identity, instance ownership and enrollment, principals, devices, roles, resource grants,
  audit records, and policy decisions;
- desired plugin lifecycle state, placement, configuration references, rollout state, and health
  summaries;
- memory metadata, provider ownership, retention, deletion jobs, and synchronization state;
- revisions, operations, idempotency, event retention, event cursors, and reconciliation; and
- issuing and revoking the narrowly scoped authorization context that an OpenAB instance needs for
  a Fleet-authorized ACP session.

It is not a proxy for ACP prompts, session updates, or transcript streaming. Studio opens ACP-over-WSS
to the selected OpenAB instance for interactive work after the management/trust boundary has
authorized that session. A management request never rides inside ACP JSON-RPC, and an ACP transport
bearer is never accepted as the identity of a Fleet principal.

MCP remains the plugin tool/data plane. The management API can declare that a plugin is installed,
enabled, placed, or revoked; invoking an enabled plugin tool and moving its data still use the
plugin contract and MCP where appropriate. An MCP tool cannot bypass management authorization or
write the Fleet registry directly.

### Controller topology

A Fleet controller is the authority that implements this API and owns its durable state. The
controller may be deployed in either topology:

| Topology | Deployment | Required behavior |
|---|---|---|
| Embedded controller | Management endpoint and durable controller state run with one OpenAB instance. | It remains a Fleet controller with a stable controller identity and persisted state; its host OpenAB instance does not get implicit extra Fleet authority. |
| Dedicated controller | Management endpoint and durable controller state run separately from all OpenAB instances. | It coordinates one or more OpenAB instances through an authenticated participant contract. It is not itself an OpenAB instance merely because it manages them. |

Both topologies implement the same management root, resource representations, authorization
decisions, revision rules, operation/idempotency behavior, error codes, event ordering, cursor
replay, session-authority propagation, and cross-Fleet-transfer boundary. A Studio client must not
branch product behavior on the topology. The topology is discoverable for diagnostics and operator
planning only.

An embedded controller must not silently elect another OpenAB instance when its host is unavailable.
A dedicated controller may have an implementation-specific high-availability deployment, but
failover must preserve one controller authority, its Fleet identity, revision history, and cursor
continuity. Peer election, split-brain reconciliation, and hosting/billing choices are outside this
ADR.

## Discovery and capabilities

Studio begins with a configured instance or controller profile and resolves the Fleet management
authority through a discovery document. An OpenAB instance may advertise the document, but
advertising does not make that instance authoritative for a different Fleet. The document is fetched
over the profile's authenticated TLS trust path before Studio sends a management credential.

GET /.well-known/openab-fleet-management returns a discovery document such as:

```json
{
  "apiVersion": "studio.openab.dev/fleet-management/v1alpha1",
  "kind": "FleetManagementDiscovery",
  "instanceId": "inst_01J9Q2H6R4D7N8K3M5V0",
  "versions": {
    "preferred": "studio.openab.dev/fleet-management/v1alpha1",
    "supported": [
      "studio.openab.dev/fleet-management/v1alpha1"
    ]
  },
  "management": {
    "endpoint": "https://fleet.example.test/fleet-management/v1alpha1",
    "controllerId": "ctrl_01J9Q31DMRT6XWWQTV6Y",
    "authorityEpoch": "ae_00000042",
    "topology": "embedded",
    "fleetId": "flt_01J9Q2FS4DNHJYCGHR7V",
    "expiresAt": "2026-08-07T12:10:00Z"
  },
  "capabilities": [
    {
      "id": "studio.openab.dev/fleet-management/revisions",
      "version": "v1alpha1",
      "required": true
    },
    {
      "id": "studio.openab.dev/fleet-management/event-cursors",
      "version": "v1alpha1",
      "required": true
    },
    {
      "id": "studio.openab.dev/fleet-management/session-revocation",
      "version": "v1alpha1",
      "required": true,
      "effectiveWithinSeconds": 60
    }
  ],
  "limits": {
    "eventRetentionSeconds": 604800,
    "idempotencyRetentionSeconds": 86400,
    "maximumEventPageSize": 500
  }
}
```

The discovery document supplies a canonical endpoint, controller identity, authority epoch, version
range, required capabilities, and operational limits. Clients negotiate behavior through those
capabilities, not through a controller build string or the embedded/dedicated label. Required,
unknown capabilities fail closed; optional extensions are namespaced and may be ignored only when
the client can still preserve authorization, revision, and audit semantics.

An unassigned OpenAB instance advertises its stable instance identity and supported management
versions but has no owning Fleet ID or active Fleet controller in this document. Enrollment bootstrap
uses the authenticated enrollment operation; it must not invent a provisional second Fleet membership.

Studio caches discovery only until its expiry. A controller endpoint change is not followed merely
because a different OpenAB instance advertises the same Fleet ID. Before an automatic endpoint
change, Studio must verify the configured trust anchor and a strictly newer authority epoch; otherwise
it requires explicit operator reapproval. The credential and trust-anchor formats are a separate
identity-federation decision.

## Resources, revisions, and mutations

Every managed resource has an immutable ID, Fleet ID where applicable, resource revision, and
server-generated timestamps. Every response also carries the current Fleet revision in the
Fleet-Revision header. Revisions are opaque strings: clients compare them for equality and never
derive ordering, time, or topology from their spelling.

- A resource revision changes whenever that resource representation changes.
- A Fleet revision advances for every accepted Fleet-visible change, including a completed operation
  that changes no user-facing resource but produces an audit-visible result.
- A Fleet revision establishes a total order only within one Fleet. There is no global order across
  Fleets.
- ETag equals the resource revision for a single-resource response. Mutations of an existing resource
  require If-Match with that ETag. A create requires If-None-Match: * when the caller expects
  absence. Compound operations carry an expected revision for every affected aggregate.
- Missing required preconditions fail with precondition_required. A stale precondition fails with
  revision_conflict. The controller never silently resolves a concurrent write with last-write-wins.

Enrollment is a controller operation because it may validate instance identity, establish a
participant channel, and write audit state. A client requests it with a durable idempotency key:

```http
POST /fleet-management/v1alpha1/fleets/flt_01J9Q2FS4DNHJYCGHR7V/instances:enroll HTTP/1.1
Authorization: Bearer <redacted-management-credential>
Content-Type: application/json
Accept: application/json
Idempotency-Key: ik_01J9Q5JZ5NVJ0Y8KYT9W
If-None-Match: *
X-Request-Id: req_01J9Q5KFNPSBAM0TVWCC

{
  "apiVersion": "studio.openab.dev/fleet-management/v1alpha1",
  "clientMutationId": "cm_01J9Q5JVKA9JKE8ET5B8",
  "instance": {
    "id": "inst_01J9Q2H6R4D7N8K3M5V0",
    "displayName": "research-runner",
    "advertisedDiscoveryUrl": "https://runner.example.test/.well-known/openab-fleet-management"
  },
  "expectedOwnerFleetId": null
}
```

On first acceptance, the controller returns a stable operation record:

```http
HTTP/1.1 202 Accepted
Content-Type: application/json
Location: /fleet-management/v1alpha1/operations/op_01J9Q5M4WKK2J0ZR4FSP
Fleet-Revision: fr_0000000000000042
X-Request-Id: req_01J9Q5KFNPSBAM0TVWCC

{
  "apiVersion": "studio.openab.dev/fleet-management/v1alpha1",
  "kind": "Operation",
  "metadata": {
    "id": "op_01J9Q5M4WKK2J0ZR4FSP",
    "fleetId": "flt_01J9Q2FS4DNHJYCGHR7V",
    "revision": "oprev_0000000000000001"
  },
  "status": {
    "state": "pending",
    "acceptedFleetRevision": "fr_0000000000000042",
    "requestId": "req_01J9Q5KFNPSBAM0TVWCC"
  }
}
```

The controller persists the tuple of authenticated principal, method, canonical target, and
Idempotency-Key with a fingerprint of the request. Retrying the same tuple and fingerprint returns
the original operation/status result, even after a connection failure. Reusing a key with a different
fingerprint returns idempotency_key_reused. Discovery advertises the minimum retention interval; a
client retains its clientMutationId and operation ID longer so it can query the operation after that
interval.

### Cross-Fleet transfer is deferred

V1alpha1 does not define a cross-controller Fleet transfer protocol. The instances:enroll operation
accepts only a truly unassigned OpenAB instance: at acceptance, the controller must verify
atomically against the durable ownership authority that the instance has no owning Fleet. The
expectedOwnerFleetId value of null is a precondition for initial enrollment, not a transfer
authorization.

If the instance is already owned by any Fleet, instances:enroll returns the stable
transfer_protocol_required error. Studio MUST NOT simulate a move by chaining independent source
detach/unenroll and destination enroll mutations, whether directly, through a retry queue, or in
background recovery. A client crash, replay, source-controller failure, or destination-controller
failure during such a chain could otherwise lose ownership or leave authority ambiguous.

The later, explicit cross-Fleet transfer contract must preserve the accepted move invariants:

- it never creates simultaneous ownership;
- it preflights identity, resource, and provider conflicts;
- it revokes source-Fleet authority and invalidates source-Fleet caches and session bindings;
- it issues destination-Fleet authority before activating the instance there;
- it records an audited, recoverable intermediate state rather than hiding a partial transfer; and
- it reports provider/resource migration outcomes separately from the ownership transition and never
  silently copies Fleet-owned resources, grants, memory, or audit history.

## Events, cursors, and resynchronization

The controller exposes:

- JSON catch-up pages at GET /fleets/{fleetId}/events?after={opaque-cursor}&limit={n}; and
- a live SSE stream at the same path when Accept is text/event-stream. Last-Event-ID is an
  equivalent way to provide the prior cursor.

The cursor denotes the position after the last event the client durably applied. It is opaque,
Fleet-scoped, and not an authorization credential. Events are ordered exactly once in the controller
log but delivered at least once to clients; Studio deduplicates by eventId and only advances its
stored cursor after atomically applying the event to the cache for that Fleet.

An SSE event has this fixture shape:

```text
id: cur_01J9Q6D6XGW49NEKJ5WW
event: instance.enrolled
data: {"apiVersion":"studio.openab.dev/fleet-management/v1alpha1","kind":"FleetEvent","eventId":"evt_01J9Q6D6WTDZ3Y02SA62","cursor":"cur_01J9Q6D6XGW49NEKJ5WW","fleetId":"flt_01J9Q2FS4DNHJYCGHR7V","fleetRevision":"fr_0000000000000043","aggregate":{"kind":"OpenABInstance","id":"inst_01J9Q2H6R4D7N8K3M5V0","revision":"irev_0000000000000001"},"occurredAt":"2026-08-07T12:02:10Z","actor":{"principalId":"usr_01J9PZWYH06E9K03DYH8","deviceId":"dev_01J9Q4QQBARYQMEYB7W4"},"requestId":"req_01J9Q5KFNPSBAM0TVWCC","data":{"ownerFleetId":"flt_01J9Q2FS4DNHJYCGHR7V","health":"enrolling"}}
```

The event body contains only fields visible to the subscribing principal. Secrets, raw credentials,
authorization tokens, and redacted audit payloads never appear in events. An event may say that a
resource became unavailable to the principal without exposing the resource's former contents.

A cursor outside advertised retention returns cursor_expired with a snapshot URL and current Fleet
revision. The client must fetch an authorized snapshot, replace only that Fleet's cached state in one
transaction, store the returned cursor, then resume catch-up. A cursor gap or an unexpected Fleet
revision similarly triggers snapshot resynchronization rather than client-side inference.

```json
{
  "apiVersion": "studio.openab.dev/fleet-management/v1alpha1",
  "kind": "FleetSnapshot",
  "fleetId": "flt_01J9Q2FS4DNHJYCGHR7V",
  "fleetRevision": "fr_0000000000000060",
  "cursor": "cur_01J9Q7FHY4E7C2VBFT48",
  "items": [
    {
      "kind": "OpenABInstance",
      "metadata": {
        "id": "inst_01J9Q2H6R4D7N8K3M5V0",
        "fleetId": "flt_01J9Q2FS4DNHJYCGHR7V",
        "revision": "irev_0000000000000003"
      },
      "status": {
        "health": "ready"
      }
    }
  ]
}
```

## Error model

Errors use one machine-readable envelope. The HTTP status says whether the request reached and was
understood by the controller; the code says whether retry, user conflict resolution, reauthentication,
or resynchronization is required.

```json
{
  "apiVersion": "studio.openab.dev/fleet-management/v1alpha1",
  "error": {
    "code": "revision_conflict",
    "message": "The OpenAB instance changed after the supplied revision.",
    "httpStatus": 412,
    "retryable": false,
    "requestId": "req_01J9Q8EMVWR2DQ73W1D8",
    "details": {
      "currentRevision": "irev_0000000000000003",
      "currentFleetRevision": "fr_0000000000000060",
      "conflictingFields": [
        "spec.displayName"
      ]
    }
  }
}
```

| HTTP status | Code | Required client behavior |
|---:|---|---|
| 401 | unauthenticated or credential_revoked | Do not retry with the same credential. Reauthenticate; a revoked device also clears the Fleet from active UI state. |
| 403 | authorization_denied or grant_revoked | Do not reveal or reconstruct hidden resource data. Refresh authorized state after the next cursor/snapshot. |
| 409 | transfer_protocol_required | Do not call an independent source detach/unenroll or destination enroll. Preserve the visible current state and require the later transfer protocol. |
| 409 | idempotency_key_reused | Retain the original operation state; generate a new key only for a new user intent after explicit review. |
| 412 | revision_conflict | Fetch the current representation, retain the pending user intent, and require explicit resolution. |
| 428 | precondition_required | Retry only after attaching the required current revision or create precondition. |
| 410 | cursor_expired | Fetch the supplied authorized snapshot and resume from its cursor. |
| 426 | required_capability_missing | Stop the operation; update/downgrade through an explicitly supported compatibility path. |
| 503 | controller_unavailable | Preserve the operation ID/idempotency key, back off, rediscover when appropriate, and query operation status after reconnect. |

Errors are redacted according to the caller's grants. In particular, an unauthorized caller cannot
use a conflict or operation error to enumerate instance IDs, controller endpoints, secret references,
or another principal's identity.

## Offline, reconnect, revocation, and controller loss

Studio can cache authorized, revisioned Fleet state for offline display. The cache is partitioned by
Fleet ID and carries its snapshot revision, cursor, discovery expiry, and staleness time. It is not
authority to grant access, create an audit record, or make a mutation appear accepted.

While offline, Studio may queue only explicit user intents that have a clientMutationId,
Idempotency-Key, required revision precondition, and a safe dependency order. It stores no
unredacted credential in the queue. On reconnect, Studio:

1. refreshes discovery and authentication before sending any queued intent;
2. catches up events or replaces the local Fleet snapshot if the cursor expired;
3. reevaluates capabilities and current authorization at the controller;
4. submits each queued intent with its original idempotency key; and
5. surfaces revision conflicts, policy denial, or unknown operation status for explicit user action.

Studio never resolves a rejected offline mutation with last-write-wins. If the network drops after
submission, it treats the result as unknown and queries the durable operation before submitting
again. Pending and conflicting state remains visibly distinct from controller-confirmed state.

Revocation is an active state transition, not merely a UI refresh:

- The controller revalidates a management credential and grant on every request and before retaining
  an event subscription.
- A credential or device revocation terminates the event stream after the controller can safely
  report the revocation reason, rejects further management calls, and causes Studio to hide and
  purge the affected Fleet cache from normal access.
- A grant/resource revocation emits an authorized audit/event record, invalidates relevant
  controller caches, and prevents new operations immediately at the controller.
- A controller-issued ACP session authorization carries Fleet ID, principal, device, audience
  OpenAB instance, expiry, and revocation epoch. The OpenAB instance validates it on session create
  and resume and enforces revocation within the advertised
  session-revocation effectiveWithinSeconds bound.

When a controller is unreachable, Studio labels management state stale and returns or displays
controller_unavailable; it does not substitute direct instance calls for Fleet mutations. An embedded
controller outage is therefore a management-plane outage even if its host OpenAB instance can still
serve an already-authorized ACP session. A dedicated-controller outage has the same client contract.

OpenAB instances may continue only work covered by a locally verified, unexpired session authority.
They must not mint or extend Fleet authority while disconnected from the controller, and they fail
closed once that authority expires. A controller outage therefore cannot turn a cached ACP transport
bearer into indefinite Fleet access. Existing ACP prompt/cancel semantics remain ACP-specific and are
not represented as a successful management mutation.

## Identity propagation and audit attribution

The management credential authenticates the Studio user/device to the controller. The controller
derives a request identity from its verified credential and device binding, evaluates Fleet role and
resource grants, and records the decision. Studio must not put an asserted principal, role, Fleet
grant, or controller privilege in a JSON body and expect it to be trusted.

For every accepted high-impact mutation, the controller records an audit attribution with at least:

- authenticated principal ID and credential/session ID;
- device ID or service-account identity;
- Fleet ID, target resource IDs, request ID, clientMutationId, and idempotency key fingerprint;
- controller ID, authority epoch, policy/grant revision, resulting Fleet revision, and timestamp;
- redacted request/result digest; and
- the delegated actor when a service or agent acted on behalf of a principal.

When an authorized ACP session is created, the controller gives the selected OpenAB instance a
minimized, audience-bound session authority rather than the Studio management credential. The
logical claims are issuer/controller identity, subject principal, device/service identity, Fleet ID,
target instance ID, evaluated grant/policy revision, expiry, and revocation epoch. Token encoding,
proof-of-possession format, and identity federation are deferred, but the claims and audience
validation are mandatory.

The ACP transport bearer authenticates only the ACP connection. The fixed ACP sender identity used by
the current OpenAB transport cannot be elevated into this Fleet identity. An OpenAB instance must
receive the evaluated session authority through the paired OpenAB work below before shared-Fleet
Studio sessions are claimed as supported.

## Paired OpenAB repository dependencies

This ADR does not create external tracker state. Before dependent implementation is marked Ready,
root coordination must create and link the following issue-ready OpenAB follow-ups to the Studio
consumer work. They are paired because Studio cannot safely emulate these OpenAB instance behaviors
in its own repository.

| Pair | Issue-ready OpenAB follow-up title and scope | Studio work gated by the result | Required proof |
|---|---|---|---|
| OAB-FM-01 | **Fleet Management: OpenAB instance discovery and controller participant contract** — advertise stable instance identity, management discovery, protocol/capability versions, authenticated enrollment, controller acknowledgement, desired-state acknowledgement, and health/heartbeat facts. | Instance enrollment, capability cache, controller topology discovery. | An embedded and a dedicated fixture advertise indistinguishable client contract data except diagnostic topology. |
| OAB-FM-02 | **Fleet Management: audience-bound session authority propagation into ACP** — accept a controller-issued session authority at ACP session create/resume, bind it to the selected OpenAB instance and principal/device, and keep ACP transport authentication separate. | Shared-Fleet ACP session launch and attributable agent access. | An ACP transport bearer alone is denied as a Fleet identity; a valid controller authority succeeds only for its audience and expiry. |
| OAB-FM-03 | **Fleet Management: revocation and authorization-lease enforcement** — consume controller revocation/epoch changes, bound local authorization caching, reject new/renewed session authority when controller connectivity is lost, and expose enforcement capability/latency. | Grant/device revocation UX and controller-loss recovery. | Revoked management/session authority is rejected within the advertised bound; controller loss cannot extend an expired authority. |
| OAB-FM-04 | **Fleet Management: revisioned instance status and operation acknowledgement** — publish lifecycle/health/configuration status with instance-side revisions and acknowledge controller operations without exposing secrets. | Fleet event ingestion, operation completion UI, plugin placement/health summaries. | Duplicate delivery, out-of-order acknowledgement, and a restarted instance converge through revision/cursor fixtures. |

These follow-ups intentionally do not ask OpenAB to implement the Studio controller, hosting,
billing, or Studio UI. They define the OpenAB-instance-facing half of the shared management
contract. Each resulting OpenAB issue and PR must link back to this ADR and its paired Studio
consumer issue before the relevant P3 work is considered unblocked.

### Deferred Studio decision: Cross-Fleet Transfer Protocol

| Owner | Issue-ready decision/follow-up | Scope | Required failure and retry proof |
|---|---|---|---|
| W0/W2 maintainer; root coordination creates and links the issue | **Fleet Management: define cross-Fleet transfer protocol** | Choose the coordinator, transfer authorization/receipt, operation endpoint, durable state model, and recovery authority for moving an OpenAB instance between independently authoritative Fleet controllers. It must implement the deferred invariants above and identify any paired OpenAB participant work. | Inject a client crash/replay, source or destination controller loss, duplicate idempotency retry, and controller restart at every transfer boundary. Prove exactly one owner, no unbounded source or destination authority, a durable queryable recovery state, and explicit provider/resource migration outcomes. |

## Consequences

- Studio has one durable contract for fleet state and can provide the same management functionality
  from desktop, tablet, and phone.
- A personal embedded deployment can start simply without creating a second client protocol, while a
  dedicated controller can scale to multi-instance/shared fleets without a client migration.
- Revisions, idempotency keys, operation records, and cursors make retries and multi-device
  reconciliation explicit rather than relying on transport success.
- The single-owner invariant remains enforceable at the only authority that can audit ownership
  changes.
- Shared-Fleet ACP use stays intentionally gated on the paired OpenAB identity and revocation work;
  this ADR does not overstate current ACP transport authentication.

## Non-goals

- Implementing a controller, controller store, OpenAB participant, or Studio client.
- Selecting controller hosting, high-availability vendor, tenancy, billing, or marketplace policy.
- Defining identity-provider federation, token encoding, proof-of-possession, or key rotation.
- Defining or implementing the deferred cross-controller Fleet transfer protocol.
- Replacing ACP session traffic or making ACP a Fleet inventory/authorization protocol.
- Replacing MCP or making MCP a Fleet-management mutation bypass.
- Defining transcript synchronization, plugin package signing, memory-provider data migration, or
  controller split-brain recovery.

## Acceptance criteria

- Contract fixtures can exercise discovery, an idempotent revisioned mutation, an operation response,
  replayable event cursor, snapshot resynchronization, and a redacted error envelope.
- Embedded and dedicated controllers expose the same management contract and differ only in
  discoverable diagnostic topology.
- Offline retry, reconnect, cursor expiry, credential/grant revocation, and controller loss have
  explicit client and OpenAB-instance behavior without silent last-write-wins.
- V1alpha1 enrollment accepts only unassigned instances; cross-Fleet moves require the deferred,
  coordinated transfer protocol and cannot be composed from independent mutations.
- ACP session authority is distinct from the ACP transport bearer, and MCP remains a plugin data
  plane.
- The paired OpenAB dependency map names the instance work that must land before shared-Fleet
  management/session claims are enabled.

## Follow-up decisions

The next accepted contracts must define shared schema/compatibility policy, identity federation and
credential formats, controller persistence/failover guarantees, exact audit retention/redaction
requirements, the concrete OpenAB participant wire protocol, and the deferred cross-Fleet transfer
protocol. Those decisions may refine this profile only if they preserve the management-plane boundary
and the safety behavior above.
