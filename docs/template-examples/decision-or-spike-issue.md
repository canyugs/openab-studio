# Dry-run decision or spike issue

## Issue type

Decision (`type:decision`)

## Goal

Decide whether repository templates can enforce complete planning inputs without changing runtime
behavior.

## Non-goals

Do not adopt a new issue tracker or automate Project-field updates.

## Dependencies

The accepted project-management issue contract and Review Contract.

## Affected contracts

Repository contribution and tracker-authoring guidance only.

## Acceptance criteria

- [ ] The decision records whether forms can collect every required field.
- [ ] The result links to a deterministic validation proof.

## Proof

Review the rendered form and run `sh scripts/validate-templates.sh`.

## Security and platform notes

No runtime permissions, secrets, migrations, data-loss behavior, or OS-specific behavior change.
