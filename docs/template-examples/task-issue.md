# Dry-run task issue

## Goal

Add repository templates that require a complete issue contract and a frozen PR Review Contract.

## Non-goals

Do not create later-phase issues, automate merge approval, or implement Studio product behavior.

## Dependencies

The planning contracts in `docs/project-management.md` and `docs/workstreams.md` are accepted.

## Affected contracts

The repository issue and PR authoring contract; no OpenAB runtime or Studio product contract changes.

## Acceptance criteria

- [ ] Task, decision/spike, and bug forms require every issue-contract field.
- [ ] The PR template contains the frozen Review Contract.

## Proof

Run `sh scripts/validate-templates.sh` and inspect the rendered forms and PR body on GitHub.

## Security and platform notes

No secrets or platform-specific behavior are introduced; the forms require authors to record either
applicable risks or `None`.
