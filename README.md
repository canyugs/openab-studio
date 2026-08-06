# OpenAB Studio

- **Status:** Planning
- **Last updated:** 2026-08-07
- **Architecture:** [System architecture](./docs/architecture.md)
- **Client decision:** [Cross-platform remote-first client ADR](./docs/adr/remote-first-client.md)
- **Plugin decision:** [Plugin platform ADR](./docs/adr/plugin-platform.md)
- **Delivery:** [Roadmap](./ROADMAP.md) · [Workstreams](./docs/workstreams.md) · [Project management](./docs/project-management.md)

OpenAB Studio is a cross-platform control surface and plugin playground for one or more fleets of
OpenAB instances. It manages connections, agents, resources, grants, memory, plugins, and audit
history while using ACP-over-WebSocket for interactive agent sessions.

Studio is not the OpenAB runtime. An OpenAB instance may be local, private, or managed remotely.
The first releases connect to existing instances; installing and provisioning instances is a later
plugin-driven capability.

## Product target

The target surfaces are macOS, Windows, Linux, iPadOS, iOS, Android tablets, and Android phones.
Delivery is staged, but mobile is a full management surface rather than a read-only companion.
Platform constraints affect where code can execute, not which fleets a user can manage.

The browser-hosted Web App is paused. Shared UI code may remain portable, but Web distribution is
not part of the active roadmap.

## Product principles

1. **Remote first, local compatible.** Local and remote OpenAB instances use the same contracts.
2. **Multi-fleet from the data model.** Fleet identity is not retrofitted onto a single-instance app.
3. **Two protocol planes.** Management uses a durable Fleet Management API; live sessions use ACP
   over WebSocket. MCP is the plugin tool/data plane, not the entire management model.
4. **Kernel core-first, features plugin-first.** Identity, policy, secrets, lifecycle, storage, audit,
   and protocol negotiation belong to the trusted core. Integrations and domain features should use
   the same public Plugin SDK available to third parties.
5. **Capabilities are grants.** Agent access to resources, memory, plugins, and device functions is
   explicit, scoped, revocable, and auditable.
6. **Management parity, adaptive execution.** Phones and tablets can perform complete fleet
   management. Arbitrary local executables remain desktop-only; mobile manages remote plugins and
   instance-hosted capabilities.
7. **Dogfood the public seams.** First-party integrations must not rely on private APIs unavailable
   to external plugin authors.

## Initial product slices

- Register existing OpenAB instances and organize them into fleets.
- Connect to an agent through ACP-over-WebSocket, with honest reconnect and resume semantics.
- Inspect agents, resources, grants, plugins, memory, and audit events across a fleet.
- Install a low-risk `echo` plugin through the public plugin lifecycle.
- Dogfood a `studio-dev` plugin that exposes read-only repository, pull request, and CI information.
- Ship verified desktop builds before expanding the same management model to tablets and phones.

## Repository boundary

This repository owns the Studio product, trusted client core, UI, device shells, Plugin SDK, and
Studio-side contracts. Changes to the OpenAB instance runtime or its protocol implementation belong
in the [OpenAB repository](https://github.com/openabdev/openab). A cross-repository feature should
have one tracking issue per repository and an explicit dependency between them.

GitHub Issues are the live source of task status once execution starts. These documents define the
accepted direction, boundaries, milestone exit criteria, and task decomposition; they are not a
substitute for the tracker.

## Contributing

Use the [contribution guide](./CONTRIBUTING.md) for the canonical ADR, roadmap, workstream, and
tracker-ownership routing rules. Automation agents should also follow [AGENTS.md](./AGENTS.md).
