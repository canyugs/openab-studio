# Desktop Packaging Feasibility Spike

- **Issue:** [#11](https://github.com/canyugs/openab-studio/issues/11)
- **Workstream:** W6 — Platform and release
- **Status:** Final hosted run
  [31150124485](https://github.com/canyugs/openab-studio/actions/runs/31150124485) passed native
  bundle, isolated install, and launch-liveness jobs for macOS arm64, Windows x64, and Linux x64.
  A normal downloaded macOS dogfood attempt was Gatekeeper-blocked; this remains feasibility
  evidence, not a release decision.
- **Decision boundary:** P0 feasibility only; this does not choose supported release tiers or ship a
  desktop release.

## Question and scope

Can the current Tauri shell produce a native package, install it in an isolated GitHub-hosted runner,
and keep the installed application process alive on one named architecture for each desktop operating
system family?

The controlled attempt is intentionally narrow:

| Family | Hosted runner | Native target | Requested bundles | Install and launch attempt |
|---|---|---|---|---|
| macOS | `macos-15` / arm64 | `aarch64-apple-darwin` | `.app`, `.dmg` | Copy the `.app` to an ephemeral application directory, then run its installed executable. |
| Windows | `windows-2022` / x64 | `x86_64-pc-windows-msvc` | `.msi`, NSIS setup executable | Silently install the NSIS bundle into a runner-temp directory, then launch its installed executable. |
| Linux | `ubuntu-22.04` / x64 | `x86_64-unknown-linux-gnu` | `.deb`, AppImage | Install the Debian package with `apt`, then launch its installed executable under Xvfb. |

Each job writes `uname`/Windows architecture data and fails if it does not match the declared
architecture. It builds with an explicit Tauri `--target` and `--bundles` list, verifies the expected
package files, records hashes, performs the named installation attempt, and considers launch evidence
successful only when the process remains alive for ten seconds. The GitHub-hosted runner is ephemeral;
no local host package installation, signing key, certificate, cache, or persistent credential is used.

## Evidence vocabulary

The workflow records evidence at the following levels. It must not promote a result from one level to
another.

| Level | Meaning | Does not prove |
|---|---|---|
| **B — bundle** | The runner produced the requested package files and artifact hashes. | Installation, launch, signing, or a user-visible window. |
| **I — installation** | The named installer/app-copy command returned successfully on the ephemeral runner. | A normal end-user install, upgrade, rollback, uninstall, or OS reputation acceptance. |
| **L — launch liveness** | The installed executable remained alive for ten seconds; Linux uses an ephemeral Xvfb display. | Rendered UI correctness, human interaction, accessibility, suspend/resume, network behavior, or real-device lifecycle. |
| **W — WebView environment** | The runner records the platform runtime/dependencies and reaches launch liveness. | The minimum WebView version or all supported user environments. |
| **R — release evidence** | A signed, notarized where required, update/rollback-tested package on an accepted support tier. | Nothing in this spike reaches this level. |

Artifact contents are the authoritative evidence: `runner-and-toolchain.txt`, `build.log`, bundle
file/hash listings, install logs, launch stdout/stderr, process information, and the platform-specific
WebView/signature inspection. A green compilation with no bundle, installation, and launch artifact is
only source-level evidence and is not a result for this spike.

## Reproduction

The dedicated [workflow](../../.github/workflows/spike-desktop.yml) runs on qualifying pull requests
and can be started manually with `workflow_dispatch`. It uses only full-SHA-pinned actions, top-level
`contents: read`, `persist-credentials: false`, pinned Node/Rust/pnpm versions, and no dependency cache.
It rejects a run if common macOS, Linux, updater, or Windows signing-material environment variables are
present, so a manual rerun cannot accidentally consume release credentials.

The Linux target intentionally uses Ubuntu 22.04, rather than the existing Ubuntu 24.04 source-check
runner, because the [Tauri Debian guidance](https://v2.tauri.app/distribute/debian/) recommends building
against the oldest supported baseline that still supplies WebKitGTK 4.1. The command installs only the
Linux build/runtime dependencies inside the disposable runner. Apt repository updates mean this is a
repeatable runner procedure, not a byte-for-byte reproducible build claim.

## Current implementation constraints

The current shell has `bundle.active: false` and the repository script invokes `tauri build --no-bundle`.
The spike deliberately overrides that command with explicit bundle formats without changing the shared
Tauri configuration, manifests, lockfiles, source code, or existing CI. A generated bundle proves that
the CLI override worked for the recorded job; it does not make bundles the default product behavior.

| Concern | Current observation | Spike treatment | Classification / next boundary |
|---|---|---|---|
| Platform WebView | The shell is Tauri 2. Windows uses WebView2; Linux depends on WebKitGTK; macOS uses the system WebKit. | Record Windows WebView2 registry information, Linux package versions/linkage, and macOS executable dependencies alongside launch logs. | **W only.** Minimum runtime/version policy remains a W6 release-tier decision. |
| Credential store | `Cargo.toml` and `apps/studio/src-tauri/Cargo.toml` contain only the Tauri shell/core dependencies; no credential-store/secret-broker adapter exists. | No credential is created, stored, or inspected. | Unimplemented product prerequisite, not a package failure. It is gated by C-03 and per-OS D-04 work. |
| Deep links | No deep-link plugin, registered scheme, or single-instance forwarding exists in the current shell/configuration. | An installed-package launch is tested, but no deep-link invocation is attempted. | Unimplemented product prerequisite. Desktop deep-link testing must occur after a reviewed scheme, install registration, and single-instance contract exist. |
| Updater | No updater plugin, endpoint, public key, signed metadata, or rollback contract exists. | No update check, download, install, or rollback is attempted. | Unimplemented product/release prerequisite. A Tauri updater requires a public key and release signing key; neither belongs in this spike. |
| macOS signing/notarization | The workflow provides no Apple certificate, Apple ID/API credential, or signing identity. | `codesign` inspection is captured only to make the actual unsigned/ad-hoc state visible. | **Account requirement** for Developer ID/App Store signing and notarization. No production macOS claim is possible. |
| Windows signing/reputation | The workflow supplies no Authenticode certificate or timestamping credential. | `Get-AuthenticodeSignature` output is captured for packages. | **Account requirement** for a signed release; SmartScreen/reputation is outside a hosted-runner process test. |
| Linux signing | The workflow supplies no GPG/AppImage signing material. | The `.deb` and AppImage are uploaded as untrusted feasibility artifacts. | **Account/key-management requirement** for any signed-distribution policy. AppImage signature handling also needs a separately reviewed trust model. |
| Artifact retention | GitHub Actions artifacts are uploaded for seven days. | Artifacts are proof inputs, not release downloads. | **CI gap** if upload/bundle collection fails; a later release pipeline needs a durable artifact/provenance destination. |
| Architecture coverage | The matrix is one native architecture per family only. | macOS arm64, Windows x64, and Linux x64 are checked explicitly. | **Unsupported target** for every architecture not listed here until it receives native evidence (for example macOS x64, Windows arm64, Linux arm64). |

The [Tauri Windows installer documentation](https://v2.tauri.app/distribute/windows-installer/) confirms
that Windows installer generation uses MSI/NSIS and that the default WebView2 bootstrapper policy depends
on Internet access. The [macOS signing documentation](https://v2.tauri.app/distribute/sign/macos/)
requires Apple credentials for release signing/notarization, and the
[updater documentation](https://v2.tauri.app/plugin/updater/) treats update signatures as a separate
private-key concern. These are release-engineering constraints, not evidence of a product defect in the
current minimal shell.

## Final hosted and downloaded result

Run [31150124485](https://github.com/canyugs/openab-studio/actions/runs/31150124485) completed green
on commit `be0191b5b63cccda46f665cf4316ae106b37f9fe`. It retained macOS arm64 artifact `8983019776`,
Windows x64 artifact `8983151791`, and Linux x64 artifact `8983142682` until 2026-08-14. Each job
earned B/I/L/W evidence on its named hosted environment; none earned release evidence.

The downloaded macOS DMG hash is
`f4b812bceb56a7da61dc63a0f9f4b91484bad0d77388f19897dabf64e5b34b60`, exactly matching the CI
artifact record, and `hdiutil verify` reports a valid image. The downloaded file carried Chrome's
quarantine attribute. The contained `.app` reports an ad-hoc signature with no Team identifier, and
strict deep signature verification fails with `code has no resources but signature indicates they
must be present`. Gatekeeper consequently reports that the app is damaged. Removing quarantine made
this verified developer artifact launch locally, but that bypass is not a supported installation
path. Developer ID signing, bundle resource sealing, notarization, stapling, and a fresh quarantined
download assessment are mandatory before a macOS release claim.

Windows and Linux reached hosted isolated install and process-liveness checks only. They still lack
normal user-machine installation, trusted signing, visible interaction, update, rollback, and
removal evidence.

## Failure classification

Use the generated logs before assigning a cause. The same symptom must not be reported as a generic
"desktop unsupported" result.

| Observed condition | Classification | Required response |
|---|---|---|
| A clean native runner reaches the Tauri command, but a source/configuration error prevents the requested bundle or installed app from launching. | **Code defect** | File a bounded bug with runner, target, command, failing log excerpt, and a regression test/procedure. |
| The source is viable, but the workflow lacks a native dependency, correct runner image, package collection, or launch harness. | **CI gap** | Amend this spike workflow or create a W6 CI task; do not call the target unsupported. |
| Build/install is feasible but signing, notarization, store, certificate, timestamp, or updater key provisioning is absent. | **Account requirement** | Create a release-authority task with named owner/account policy. Do not add secrets to this workflow. |
| The requested CPU/OS cannot be run natively by the declared runner, or the product/toolchain does not support the target. | **Unsupported target** | Record the exact target and available alternative; create a separate feasibility spike instead of extrapolating from another CPU/OS. |

## Bounded follow-up recommendations

These are recommendations, not hidden implementation work in this spike. They should become separate,
reviewable issue contracts before P2 release claims.

1. **R-04: Propose desktop support tiers from native evidence** — W6/W0, S. Consume this spike plus
   R-02/R-03 evidence; name minimum OS/CPU tiers and explicitly reject untested architectures. Proof:
   accepted matrix with cited runner/device evidence and residual-risk decision.
2. **D-04a/b/c: Per-OS secure credential, deep-link, updater, and lifecycle seams** — W6/W1, M per
   operating system after C-03. Define one narrow credential-store adapter, registered deep-link scheme
   and single-instance behavior, updater capability/configuration, and install/rollback acceptance
   procedure. Proof: stored-secret redaction/denial checks plus native install/deep-link/update/rollback
   evidence on the chosen architecture.
3. **D-06a/b/c: Signed release pipeline by operating system** — W6/W7, M per operating system after
   D-04. Keep certificate/key authority outside pull-request workflows; document signing, notarization
   where applicable, provenance, durable artifact retention, and rollback ownership. Proof: release
   artifact verification performed with approved release credentials, never this spike's artifacts.
4. **Windows WebView2 distribution policy** — W6, S. Decide the supported Windows baseline and whether
   the installer uses the default downloaded bootstrapper, embedded bootstrapper, offline installer, or
   fixed runtime. Proof: installer size/network/failure matrix on the accepted Windows tier.

## Decision status

Do not select a release tier until all three workflow artifact sets have been inspected and this section
is updated with their run URLs, exact bundle names/hashes, and the highest evidence level earned per
target. A passing B/I/L job remains P0 feasibility evidence, not a production support claim.
