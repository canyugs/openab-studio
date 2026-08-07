# OpenAB Studio Agent Guide

Work only from a ready, bounded issue. Do not use a local worktree, branch, commit, or roadmap prose
as evidence of live task status.

## Canonical routing and ownership

- Read the [remote-first client ADR](docs/adr/remote-first-client.md) and
  [plugin platform ADR](docs/adr/plugin-platform.md) before changing a product or system boundary.
- Use the [roadmap](ROADMAP.md) for phase direction and exit criteria.
- Use [workstreams](docs/workstreams.md#workstreams) for workstream scope and maintainer ownership.
- Use the [tracker ownership rules](docs/project-management.md#canonical-records) and
  [Project fields](docs/project-management.md#project-fields-and-views) for live task status, owner,
  dependencies, and blockers.

ADRs own decisions; the roadmap owns sequencing; workstreams own contract/module boundaries; GitHub
Issues and the Project own live execution state. If a decision changes a boundary, update or add an
ADR before dependent implementation merges. Keep temporary cutover steps in an issue or temporary
runbook, not in a canonical ADR.

## Execution rules

- One ready task produces one logical PR with explicit proof and security/platform notes.
- Do not expand a Studio change into an OpenAB runtime change; use paired cross-repository issues and
  state the dependency.
- Freeze the Review Contract in round 1. Later review focuses on unresolved findings, incremental
  changes, regressions, and frozen acceptance criteria.
- Run `sh scripts/validate-templates.sh` after changing repository templates or their dry-run
  fixtures.

For contributor-facing detail, see [CONTRIBUTING.md](CONTRIBUTING.md).
