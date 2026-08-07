# Dry-run pull request

Closes #8

## Review Contract

### Goal

Add valid issue and PR templates with repository guidance and deterministic validation.

### Non-goals

Do not create later-phase issues, automate merge approval, or implement Studio product behavior.

### Accepted residual risks

GitHub rendering is additionally checked in the pull request; the local validator proves the tracked
form structure but cannot emulate GitHub UI behavior.

### Acceptance criteria and proof

- [x] All three issue forms require the seven issue-contract fields, proven by the validator.
- [x] The PR template has all frozen Review Contract headings, proven by the validator.
- [x] Contribution and agent guides link the canonical records, proven by local link validation.

### Follow-ups

None. Later-phase issues remain deliberately out of scope.
