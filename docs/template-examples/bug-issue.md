# Dry-run bug issue

## Observed behavior

An issue form can be submitted without a required contract field.

## Expected behavior

Each task, decision/spike, and bug form requires every issue-contract field before submission.

## Reproduction and evidence

1. Open a form in GitHub.
2. Leave one required field blank.
3. Confirm GitHub prevents submission and the local validator confirms the required configuration.

## Goal

Restore required issue-contract fields to every repository issue form.

## Non-goals

Do not add product validation, change Project fields, or automate issue closure.

## Dependencies

The accepted project-management issue contract and GitHub issue-form support.

## Affected contracts

Repository issue-authoring and review evidence contracts only.

## Acceptance criteria

- [ ] Every required issue-contract field has `validations.required: true`.
- [ ] The rendered form and local validator agree on the required fields.

## Proof

Run `sh scripts/validate-templates.sh` and reproduce the required-field behavior in GitHub.

## Security and platform notes

No secret, permission, migration, data-loss, or OS-specific behavior changes; the bug form still
requires authors to identify applicable risks.
