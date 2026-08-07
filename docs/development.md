# Development

This is the intentionally small B-01 workspace: `crates/studio-core` owns the first trusted-core
seam and `apps/studio` owns the shared TypeScript UI plus one Tauri 2 shell. The same shell uses
`#[cfg_attr(mobile, tauri::mobile_entry_point)]`, so it is the desktop and mobile entry point; there
are no placeholder crates or future product packages.

## Pinned toolchain dependencies

The dependency versions below were checked against the official Tauri documentation and registries
on 2026-08-07. They are exact pins; update them deliberately with both lockfiles reviewed.

| Dependency | Version |
|---|---:|
| Rust edition / minimum compiler | 2024 / 1.85 |
| `tauri` | 2.11.5 |
| `tauri-build` | 2.6.3 |
| `@tauri-apps/api` | 2.11.1 |
| `@tauri-apps/cli` | 2.11.4 |
| Prettier | 3.9.6 |
| Vite | 8.2.1 |
| TypeScript | 7.0.2 |
| pnpm | 10.29.2 |

Node.js must satisfy Vite's `^20.19.0 || >=22.12.0` engine range. Ensure pnpm 10.29.2 is available
before the first install. The Node 22.12.0-bundled Corepack cannot currently verify that release's
signing key, so use an approved local pnpm installation rather than relying on that Corepack version:

```sh
pnpm --version # expected: 10.29.2
pnpm install --frozen-lockfile
```

## Local quality commands

```sh
pnpm format
pnpm format:check
pnpm check
pnpm test
pnpm build
```

`pnpm format` formats Rust through `cargo fmt`, the human-authored schema TypeScript and generator
sources through root-pinned Prettier, and the Tauri UI through its local Prettier configuration.
`pnpm check` performs a host Rust check and TypeScript check. `pnpm build` additionally builds the
Vite assets and the Rust workspace.

## Shared schema workflow

`schemas/studio.shared.v1alpha1.schema.json` is the canonical source for the versioned shared
Fleet, plugin, memory, grant, capability, and audit contracts. Do not edit bindings under
`crates/studio-protocol/src/generated.rs` or `schemas/generated/typescript/` by hand.

```sh
pnpm schemas:generate
pnpm schemas:check
pnpm schemas:test:typescript
pnpm contracts:verify
```

`schemas:generate` updates the committed Rust and TypeScript output. `schemas:check` regenerates in
memory and fails when the working tree's generated output does not exactly match both the canonical
source and generator. `schemas:test:typescript` compiles and runs the TypeScript fixture harness;
`pnpm test` includes it with the Rust workspace tests. `contracts:verify` is the dedicated
cross-language compatibility gate: reproducibility, Rust fixtures, then TypeScript fixtures.

The root `format` and `format:check` commands include every human-authored schema TypeScript source
under `schemas/typescript/` and the schema generator/harness sources
(`scripts/generate-schemas.mjs` and `scripts/test-typescript-contract-fixtures.mjs`) through the
root-pinned Prettier. They intentionally exclude `schemas/generated/typescript/`: those bindings are
byte-for-byte generator output, and `pnpm schemas:check` is the formatting/reproducibility authority
for them. Do not run Prettier over generated bindings; regenerate them after changing the canonical
source or generator.

The corpus under `schemas/fixtures/` is retained for supported, degraded, rejected, migration, and
unknown-field/required-extension behavior. Expected rejection fixtures are asserted as successful
test outcomes; the gate exits nonzero only when either language disagrees with the declared result.

## CI quality gates

[`CI`](../.github/workflows/ci.yml) runs for pull requests and pushes to `main`. It installs the
pinned pnpm release through `pnpm/action-setup` v6.0.10, pinned by commit SHA, with caching disabled.
Node 22.12.0's bundled Corepack cannot verify pnpm 10.29.2's current signing key, so CI does not
depend on that runner metadata. When reproducing a Node-backed gate locally, first ensure pnpm
10.29.2 is available and then install locked dependencies:

```sh
pnpm --version # expected: 10.29.2
pnpm install --frozen-lockfile
```

| CI job | Local reproduction | Scope |
|---|---|---|
| Formatting | `pnpm format:check` | Rust `cargo fmt` plus Prettier for schema TS/MJS and the Studio UI. |
| Static analysis | `cargo clippy --workspace --all-targets --locked -- -D warnings` followed by `pnpm check` | Rust Clippy and check, plus TypeScript `tsc --noEmit`. |
| Unit tests | `pnpm test` | Workspace Rust tests and `pnpm schemas:test:typescript`. |
| Template validation | `sh scripts/validate-templates.sh` | Issue forms, pull-request template, dry-run fixtures, and guidance links. |
| Contract compatibility fixtures | `pnpm contracts:verify` | Regenerated-source clean-diff check and the Rust/TypeScript compatibility corpus. |

Every command above must exit zero. For a controlled failure check, use a disposable worktree, make
one temporary relevant source, template, generated-output, or fixture change, confirm that its row's
command exits nonzero with the named failure, then remove the worktree without committing the change.
The compatibility corpus deliberately contains invalid inputs whose expected rejection is asserted;
those fixtures make the normal contract command pass rather than serving as failure probes.

The workflow uses an Ubuntu 24.04 runner only to exercise these source-level checks. It performs no
packaging, signing, deployment, device, or non-Linux platform validation, so it is not evidence of
desktop/mobile release coverage. It has read-only repository permission, persists no checkout
credentials, uses no GitHub Actions dependency cache, and reads no repository or deployment secret.

## Desktop development

```sh
pnpm tauri:desktop:dev
pnpm tauri:desktop:build
```

The build command deliberately uses `tauri build --no-bundle`: it compiles the desktop Tauri shell
without introducing packaging, signing, updater, or release support. The only permission assigned to
the `main` webview is the generated `allow-workspace-bootstrap` app-command permission. The app
enables no core defaults, shell, process, filesystem, HTTP, plugin, or remote-URL permissions. Its
content security policy allows only same-origin resources and the local Tauri IPC endpoint.

## Mobile development

Generate each native project once from the pinned Tauri CLI, then use the corresponding development
entry point:

> [!WARNING]
> Even with `--ci --skip-targets-install`, the Tauri CLI may mutate host package state. On the
> observed host, iOS initialization automatically installed Homebrew `libimobiledevice` before
> CocoaPods blocked the command. Use only an approved, prepared host and review current Tauri
> prerequisite behavior before running either init command.

```sh
pnpm tauri:ios:init
pnpm tauri:ios:dev

pnpm tauri:android:init
pnpm tauri:android:dev
```

The init commands use `--ci --skip-targets-install`, so Rust target installation remains explicit.
They do not make platform-host setup side-effect free. Install the target families explicitly when
working on them:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

For a physical iOS device, Tauri supplies `TAURI_DEV_HOST`; the Vite configuration consumes it so
the device can reach the development server. Select a device explicitly when needed, for example:

```sh
pnpm --filter @openab/studio exec tauri ios dev 'iPhone 15'
```

## Current platform evidence and blockers

The B-01 checks ran on macOS 26.5.2 (`aarch64-apple-darwin`), Rust 1.94.0, Node 26.5.0, pnpm 10.29.2,
and Xcode 26.6 with the iPhoneOS 26.5 SDK.

Desktop host builds are expected to run on that machine. Mobile release support is not claimed:

| Target family | Current result | Reproducible blocker / next prerequisite |
|---|---|---|
| iOS / iPadOS | `cargo check --workspace --locked --target aarch64-apple-ios` fails with E0463 because the target is absent. `pnpm tauri:ios:init` found Xcode 26.6, installed its missing `libimobiledevice` host dependency, then failed at `pod install`: CocoaPods is absent and the CLI's gem fallback requires `sudo`. | Install CocoaPods through the approved host setup, add `aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios`, configure a signing team/certificate when targeting a device, then rerun `pnpm tauri:ios:init`. |
| Android | `pnpm tauri:android:init` failed before scaffolding: `Android SDK not found. Make sure the SDK and NDK are installed and the ANDROID_HOME and NDK_HOME environment variables are set.` | Install Android Studio's SDK, NDK, and a JDK; set `ANDROID_HOME`, `NDK_HOME`, and `JAVA_HOME`; add the four Android Rust targets above; then rerun `pnpm tauri:android:init`. |
| Windows | `cargo check --workspace --locked --target x86_64-pc-windows-gnu` reached Tauri's Windows resource build and then failed because `x86_64-w64-mingw32-windres` is unavailable on this host. | Use a Windows build host or install the matching MinGW resource tool before retrying. This is a compile feasibility check, not Windows package proof. |
| Linux | No Linux target, linker, WebKitGTK development libraries, or native host proof is present on this macOS ARM host. | Run the documented Tauri Linux prerequisite setup and native build in Linux CI or a Linux development host. |

These are environment/platform prerequisites, not product bugs. Re-run the relevant command on a
prepared host and record its build/device evidence before claiming support for that target family.
