# Contributing to OpenAB Studio

OpenAB Studio is in the planning and contract phase. Keep each change to one logical PR and begin
with the canonical records rather than inferring ownership or progress from a local branch.

## Read before changing a contract

- Product and system decisions: [remote-first client ADR](docs/adr/remote-first-client.md) and
  [plugin platform ADR](docs/adr/plugin-platform.md).
- Phase direction and exit criteria: [roadmap](ROADMAP.md).
- Workstream scope and maintainer ownership: [workstreams](docs/workstreams.md#workstreams).
- Live status, owner, dependencies, blockers, and Project fields: [tracker ownership rules](docs/project-management.md#canonical-records)
  and [Project fields](docs/project-management.md#project-fields-and-views).

ADRs own product and system decisions. The roadmap owns phase direction and exit criteria.
Workstreams own module and contract boundaries through their maintainer owners. GitHub Issues and the
Project own live task status, direct ownership, dependencies, and blockers. Do not duplicate live
task checklists in ADRs or infer tracker state from a worktree, commit, or roadmap prose.

## Issues and pull requests

Use the repository issue forms for tasks, decisions/spikes, and bugs. A task is ready only when its
scope, dependencies, acceptance criteria, proof, and security/platform considerations are explicit.
Keep cross-repository changes paired with an OpenAB issue; Studio must not silently change the
instance runtime contract.

Every pull request uses the frozen Review Contract. In review round 1, agree the goal, non-goals,
accepted residual risks, acceptance proof, and follow-ups. Later rounds address unresolved findings,
incremental changes, regressions, and frozen criteria unless new direct correctness, security, or
data-loss evidence clears the late-blocker gate.

Run the template proof before opening a documentation-only PR:

```sh
sh scripts/validate-templates.sh
```

Automation agents should also follow [AGENTS.md](AGENTS.md).
