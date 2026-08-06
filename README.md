# OpenAB Studio

- **Status:** Planning
- **Last updated:** 2026-08-07
- **Architecture:** [Remote-first client ADR](./docs/adr/remote-first-client.md)
- **Delivery plan:** [OpenAB Studio roadmap](./ROADMAP.md)
- **Server contract:** [ACP-over-WebSocket base ADR](https://github.com/openabdev/openab/blob/main/docs/adr/acp-server-websocket-base.md)

## Product definition

OpenAB Studio is a cross-platform client for working with an OpenAB instance. The instance may run
on the same machine, on a private server, or as a managed remote deployment. The app is a surface
over OpenAB, not the owner of the agent runtime.

The first product target is one client experience shared by:

- a browser-hosted web app;
- a desktop shell for macOS, Windows, and Linux; and
- later clients that reuse the same ACP client library.

The app connects to OpenAB through ACP over WebSocket. It may also expose explicitly granted
client-side capabilities through MCP-over-ACP, but those capabilities are optional and are not part
of the first chat milestone.

## Why this can start now

The server already provides the minimum transport needed for a reference client:

- authenticated `GET /acp` WebSocket connections;
- ACP `initialize`, `session/new`, `session/resume`, `session/prompt`, and `session/cancel`;
- reconnect-to-resume without server-side transcript replay; and
- reverse MCP-over-ACP for a client that acts as an MCP server.

This is enough to prove connection management, chat, local transcript ownership, and session
resume. It is not yet enough to claim a production multi-user remote service or a fine-grained
native capability host. Those boundaries are explicit in the roadmap.

## Product principles

1. **Remote first, local compatible.** Local OpenAB is one connection profile, not a separate
   architecture. The app does not bundle or own the OpenAB core by default.
2. **Web UI first.** Browser and desktop render the same application and consume the same typed ACP
   client package. Platform integrations sit behind a narrow host interface.
3. **Protocol before product coupling.** The app negotiates ACP capabilities and does not depend on
   OpenAB implementation details that are absent from the wire contract.
4. **The client owns presentation state.** Until `session/load` exists, the app retains the
   transcript and treats `session/resume` as context continuation without history replay.
5. **Capabilities are grants, not conveniences.** Files, terminal, browser, clipboard, and similar
   host access stay unavailable until the user has granted an understandable scope.
6. **Honest controls.** A UI action must not imply stronger behavior than the server provides. For
   example, the current cancel path stops the ACP waiter but may not stop downstream agent work.
7. **One agent per ACP session.** A multi-agent room, if added, is client orchestration over several
   independent sessions rather than server-side answer aggregation.

## Initial user journeys

### Connect to an OpenAB instance

The user creates a connection profile with a display name, a `wss://` endpoint, and credentials.
Secrets are stored by the platform credential service in the desktop app and are never written into
the transcript or diagnostic export.

### Start and resume work

The user opens a conversation, the app creates an ACP session, and the app stores the returned
`sessionId` with the connection profile. After reconnecting, the app calls `session/resume` and
continues the session without expecting OpenAB to replay prior messages.

### Work from browser or desktop

The web and desktop surfaces share conversation, connection, and protocol behavior. Desktop-only
features such as native notifications, credential storage, deep links, and later client-side tools
are provided through a host adapter instead of leaking into the core UI.

### Expose a client capability

In a later milestone, the user enables a capability provider for a session. The client declares it
as a `type:acp` MCP server and serves its tools over the existing WebSocket. High-impact providers
must wait for a real permission broker; the current OpenAB auto-approval behavior is not sufficient
for a general file or terminal provider.

## Surface boundaries

| Concern | Web app | Desktop app | OpenAB server |
|---|---|---|---|
| ACP client state machine | Shared TypeScript package | Shared TypeScript package | ACP server |
| Conversation UI | Yes | Same UI | No |
| Transcript display store | Browser storage initially | Desktop app data store | No replay store today |
| Credential storage | Browser-session constraints | OS credential service | Validates transport credential |
| Local files / terminal | Browser sandbox only | Future host capability | Routes granted MCP calls |
| Agent process and workspace | No | No, by default | Yes |
| Identity and policy | Presents user/session identity | Presents user/session identity | Must enforce it before public remote use |

## Reference projects

These projects inform the product shape, not the OpenAB wire contract:

- [KiroCrew](https://github.com/kirodotdev/KiroCrew) demonstrates one Web UI used from a browser
  and desktop shell, with connections to local or remote gateways. Its desktop app bundles and
  supervises a backend; OpenAB Studio deliberately does not make that the default because remote
  OpenAB remains a first-class deployment.
- [qm](https://github.com/yc-software/qm) demonstrates a headless core with optional UI surfaces
  and strong separation between runtime, persistence, identity, and presentation. OpenAB Studio keeps
  the same separation while using ACP-over-WebSocket as its session protocol.

## Success definition for the first release

The first release is successful when a user on macOS, Windows, or Linux can install the app,
connect to a private local or remote OpenAB endpoint, start a conversation, reconnect and resume it,
cancel the client-side wait, and understand the actual connection/session state without using a
platform-specific chat adapter.

Public multi-user hosting, native host tools, automatic local-core installation, and multi-agent
rooms are later scopes. Their prerequisites and exit criteria live in the roadmap rather than this
stable product definition.
