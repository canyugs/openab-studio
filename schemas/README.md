# Shared schema contracts

`studio.shared.v1alpha1.schema.json` is the single canonical source for shared OpenAB Studio
contracts. It owns the schema family/version, strict base-object rules, optional extension container,
and forward migration metadata.

The committed generated output is intentionally reviewable:

- Rust bindings: `../crates/studio-protocol/src/generated.rs`
- TypeScript bindings and generated schema value: `generated/typescript/`

Use `pnpm schemas:generate` after changing the source or generator. Never hand-edit generated files.
`pnpm schemas:check` is a clean-diff reproducibility proof: each generated header includes the exact
source and generator digests, and the command fails if regeneration differs.

`pnpm contracts:verify` is the full shared-contract gate. It verifies generated output, then runs the
same fixtures through Rust and compiled TypeScript validators, compatibility selection, migrations,
and serialization round trips. The negative fixtures must remain: a strict unknown base field is
`schema-unknown-field`, and an unavailable required extension is
`required-extension-unavailable` before any side effect.
