# OpenAB Studio Project and Progress Management

- **Status:** Proposed operating model
- **Last updated:** 2026-08-07
- **Roadmap:** [Milestones](../ROADMAP.md)
- **Task model:** [Workstreams](./workstreams.md)

## Canonical records

| Information | Canonical location |
|---|---|
| Product/system decision and consequences | ADR in `docs/adr/` |
| Phase direction and exit criteria | `ROADMAP.md` |
| Task status, owner, dependency, blocker | GitHub Issue and Project |
| Code change and acceptance evidence | Pull request |
| Temporary cutover/checklist | Issue or dedicated temporary runbook |
| Released support claim | Release notes plus platform evidence |

Do not copy live task checkboxes into stable ADRs. Do not infer status from a local worktree, branch,
commit, or roadmap prose when the GitHub Issue says otherwise.

## Issue hierarchy

- **Decision:** one bounded question that results in an ADR or explicit rejection.
- **Epic:** a roadmap capability spanning multiple independently accepted tasks.
- **Task:** an S/M/L deliverable producing one logical PR or evidence artifact.
- **Spike:** time-boxed uncertainty reduction with a decision-quality output, not production code by
  default.
- **Bug:** observed behavior that violates an accepted contract or release claim.

GitHub sub-issues or task lists connect epics to tasks. A tracking issue never substitutes for each
task's acceptance criteria and owner.

## Labels

Use namespaced labels so filtering stays predictable:

| Dimension | Labels |
|---|---|
| Type | `type:decision`, `type:epic`, `type:task`, `type:spike`, `type:bug` |
| Area | `area:architecture`, `area:core`, `area:fleet`, `area:protocol`, `area:plugin`, `area:memory`, `area:ui`, `area:platform`, `area:security`, `area:quality` |
| Platform | `platform:shared`, `platform:macos`, `platform:windows`, `platform:linux`, `platform:ios`, `platform:android` |
| Size | `size:S`, `size:M`, `size:L`, `size:XL` |
| Priority | `priority:P0`, `priority:P1`, `priority:P2`, `priority:P3` |
| State qualifier | `status:blocked`, `status:needs-decision`, `status:external` |
| Risk | `risk:security`, `risk:data-loss`, `risk:protocol`, `risk:platform`, `risk:migration` |

Project Status is a field, not a duplicate label. Phase names use milestones `P0 Contracts` through
`P7 Ecosystem`. Priority describes urgency within a phase; it does not replace the phase dependency.

## Project fields and views

Create one GitHub Project with these fields:

| Field | Values/purpose |
|---|---|
| Status | Backlog, Ready, In progress, In review, Blocked, Done |
| Phase | P0–P7 |
| Workstream | W0–W7 |
| Area | Primary contract/module owner |
| Platform | Shared or one target OS |
| Size | S/M/L/XL |
| Priority | P0/P1/P2/P3 |
| Owner | One directly responsible person/agent operator |
| Target iteration | Optional short planning window, never a release promise |

Recommended views:

- **Now:** Ready/In progress/In review grouped by workstream.
- **Dependencies:** Blocked and needs-decision items grouped by phase.
- **Roadmap:** Epics grouped by milestone.
- **Platforms:** release and spike tasks grouped by target platform.
- **Security/protocol:** filtered high-risk contract changes.
- **Dogfood:** tasks and bugs affecting the current internal workflow.

## Issue contract

Every implementation issue contains:

```markdown
## Goal
One observable result.

## Non-goals
Adjacent work deliberately excluded.

## Dependencies
Accepted ADRs, schemas, issues, external accounts, or OpenAB changes.

## Affected contracts
Schema/API/module ownership and compatibility expectations.

## Acceptance criteria
- [ ] Behavior or artifact that can be independently verified.

## Proof
Automated tests, fixtures, screenshots, packages, devices, or manual procedure required.

## Security and platform notes
Permissions, secrets, migration, data-loss, and OS-specific risks.
```

An issue is **Ready** only when the goal will not change based on an unanswered architecture question,
dependencies are merged/available, acceptance is testable, and an owner can work within one declared
scope. Blocked issues name the blocker and the next party/action; “waiting” alone is insufficient.

## Pull request Review Contract

Every PR freezes this contract during review:

```markdown
## Review Contract

### Goal

### Non-goals

### Accepted residual risks

### Acceptance criteria and proof

### Follow-ups
```

Review round 1 may challenge and then freezes the contract. Later rounds review unresolved findings,
incremental changes, regressions, and frozen criteria. Post-freeze findings are classified as
`ORIGINAL`, `REGRESSION`, `NEW EVIDENCE`, or `SCOPE EXPANSION`; scope expansion becomes a follow-up
unless direct correctness, security, or data-loss evidence passes the late-blocker gate.

Use the default stopping rule: full review, fix verification, final regression check. Only the
maintainer/owner revises the contract or authorizes another round.

## Definition of done

A task becomes Done only when:

- code/docs and generated artifacts are merged;
- acceptance proof is attached or linked from the PR;
- relevant unit, contract, E2E, migration, and platform checks pass;
- security-sensitive changes show deny/revoke/redaction behavior, not only the happy path;
- user-facing or author-facing contracts and migrations are documented;
- follow-ups are separate issues rather than hidden TODO prose; and
- the parent epic/project status is reconciled.

“Works on all platforms” requires recorded evidence for every claimed OS family and architecture.
Shared source code or a green Linux CI job is not cross-platform evidence.

## Measuring progress

Do not report percent complete from issue count, code volume, or elapsed roadmap phases. Report:

- capabilities whose exit criteria are proven;
- current vertical slice that a user can actually run;
- Ready/In progress/In review/Blocked counts by workstream and size;
- dependency chain and oldest unresolved blocker;
- platform evidence earned versus claimed support matrix;
- dogfood usage, failures, and recovery outcomes; and
- accepted scope/ADR changes since the previous report.

A concise weekly update should be:

```markdown
## Outcome
What became usable or provable this week.

## Evidence
Merged PRs, fixtures, artifacts, devices, and dogfood sessions.

## In flight
Owner, issue, size, expected acceptance event.

## Blockers and decisions
Blocker, owner/next action, affected dependency chain.

## Scope changes
Accepted ADR changes and newly created follow-ups.
```

## Work-in-progress policy

- Prefer one implementation task per owner plus at most one review task.
- Start from Ready, not directly from Backlog.
- S/M work is the normal unit; split XL work before assignment.
- A workstream may have several parallel worktrees only when they do not share a chokepoint.
- If a blocker lasts beyond the next coordination checkpoint, update the issue and pull forward an
  independent Ready task rather than hiding work on an untracked branch.
- Cross-repository blockers use paired links and state which repository owns the next action.

## Change control

- A product/system boundary change requires an ADR update or a new ADR before dependent implementation
  merges.
- A sequencing change updates the roadmap or Project fields, not a stable architecture document.
- A new platform claim requires release evidence and support-tier acceptance.
- A plugin permission or runtime expansion requires threat-model and compatibility review.
- Temporary migration/cutover steps live outside canonical ADRs and are removed or archived after
  completion.

## Initial tracker setup order

After maintainers review this planning set:

1. create the labels, P0–P7 milestones, and Project fields/views;
2. create P0 epic and issue-ready tasks from `docs/workstreams.md`;
3. link blocking relationships and select only dependency-free Ready tasks;
4. assign the root-workspace bootstrap chokepoint before parallel worktrees start; and
5. create P1 tasks only as needed for contract review, leaving implementation blocked until P0 exits.

This setup should be one auditable administrative change. It should not create every later-phase task
prematurely; later epics stay coarse until their prerequisites make accurate S/M decomposition possible.
