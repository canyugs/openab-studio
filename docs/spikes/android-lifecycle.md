# Android phone and tablet Tauri lifecycle spike

- **Issue:** [#13](https://github.com/canyugs/openab-studio/issues/13)
- **Scope:** P0 build-and-launch feasibility for the existing typed `workspace_bootstrap` Rust/TypeScript seam.
- **Evidence workflow:** [Android lifecycle spike](https://github.com/canyugs/openab-studio/actions/workflows/spike-android.yml)
- **Decision status:** pending the first hosted-runner result.

## Boundary

This spike deliberately changes neither the product shell nor Android source. The workflow creates
`apps/studio/src-tauri/gen/android` only on a disposable GitHub-hosted runner, builds it, captures
evidence, and lets the runner disappear. Generated Android source, build outputs, AVD state, debug
keys, SDK/NDK/JDK setup, and emulators are never committed and are never created on a developer host.

The exact seam under test is intentionally narrow:

```text
TypeScript invoke("workspace_bootstrap")
  -> Tauri command
  -> studio_core::workspace_bootstrap()
  -> typed { protocolVersion, status: "ready" } response rendered by the WebView
```

It proves that the current Tauri mobile entry point can package and launch the existing boundary. It
does not prove a Fleet, ACP, storage, credential, plugin, or full-management workflow.

## Reproducible hosted procedure

The `Android lifecycle spike` workflow uses a read-only checkout with credentials disabled and no
dependency, Gradle, or AVD cache. Every action is commit-SHA pinned. It uses an Ubuntu 24.04
GitHub-hosted runner, Rust 1.85, Node 22.12.0, pnpm 10.29.2, Temurin JDK 21, Android API 35,
build-tools 35.0.0, NDK 27.2.12479018, and Google APIs x86_64 emulators.

The runner:

1. installs all four Tauri Android Rust targets and generates the Android project with
   `tauri android init --ci --skip-targets-install`;
2. builds debug, split APKs for `aarch64`, `armv7`, `i686`, and `x86_64`;
3. launches the x86_64 APK on distinct Pixel 7 (phone) and Pixel Tablet (tablet) AVD profiles;
4. captures a foreground launch screenshot/activity dump, Home/background process/task state,
   foreground return, force-stop process absence, and cold relaunch; and
5. captures emitted manifest/Gradle metadata, per-APK native libraries, SDK/target metadata,
   debug-signing verification, package data, a deep-link attempt, logcat, and source-tree status.

The artifact is named `android-lifecycle-spike-<run-id>` and is retained for 14 days. It includes no
release or debug private key. A debug certificate fingerprint may appear in `apksigner` output and
is evidence only, not a signing identity.

## Evidence interpretation

| Question              | What the workflow can establish                                                                                                                                                                              | What it cannot establish                                                                                                                                                                     |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Typed seam            | The generated x86_64 app installs, starts, captures its WebView/activity state, and can render the bootstrap screen. The screenshot is retained when Android accessibility does not expose WebView DOM text. | Any product operation beyond the fixed no-op response.                                                                                                                                       |
| Phone vs tablet       | Pixel 7 and Pixel Tablet AVDs provide separately captured display/profile, install, foreground/background, termination, and relaunch evidence.                                                               | Real-device touch ergonomics, adaptive full-management UX, OEM display behavior, or vendor power policy.                                                                                     |
| ABI                   | The workflow records a separate debug APK inspection for aarch64, armv7, i686, and x86_64. Only x86_64 is launched on the hosted emulator.                                                                   | That x86_64 behavior proves ARM behavior; each ARM ABI still needs physical-device evidence.                                                                                                 |
| Foreground/background | Launch, `KEYCODE_HOME`, task/activity state, process ID, and foreground return are recorded for each AVD.                                                                                                    | Android callback timing, Doze, memory pressure, process reclaim, reboot recovery, or a vendor's background restrictions. The current seam has no lifecycle instrumentation.                  |
| Termination           | `am force-stop` is followed by recorded process absence and cold relaunch for each AVD.                                                                                                                      | Crash, upgrade, OS kill, device reboot, restore, or battery-optimization behavior.                                                                                                           |
| Reconnect             | The workflow demonstrates a fresh relaunch only.                                                                                                                                                             | Network/session reconnect: the current shell has no ACP/Fleet transport, connection state, or reconnect implementation to measure. A cold start must not be described as reconnect evidence. |

An Android emulator is an AOSP/Google APIs test environment, not a representative phone or tablet
vendor. The proof is therefore intentionally partitioned by AVD profile and ABI, and it does not
make a generic Android lifecycle claim.

## Current platform constraints

### Secure storage and signing

No current `studio-core` secret adapter invokes Android Keystore. An emulator build cannot prove a
hardware-backed key, StrongBox availability, lock-screen invalidation, backup/restore behavior, or
device-loss recovery. Any future credential adapter must use the [Android Keystore
system](https://developer.android.com/privacy-and-security/keystore), carry opaque handles rather
than raw secret values into the WebView, and gain physical-device security evidence before a release
claim.

The spike produces debug APKs only. Gradle's debug signing is inspected with `apksigner`, while no
release keystore, upload key, signing secret, store credential, or release signing configuration is
provided. A passing build is not distribution or store-signing evidence.

### Networking, deep links, and notifications

The current source has no Fleet/ACP network client or reconnect state machine, and its Tauri content
security policy permits only same-origin and local IPC connections. The emitted manifest and package
dump are archived so any generated `INTERNET` declaration is visible, but a declaration is not
networking behavior.

There is no committed Android intent-filter/deep-link configuration. The workflow records an
`openab-studio://spike` launch attempt as a negative configuration check; it does not claim
deep-link support. There is also no notification permission request, notification adapter, or
notification delivery path in the current seam, so the spike records this as unimplemented rather
than manufacturing a notification test.

### API level, device class, and vendor lifecycle

The emitted Gradle file, manifest, `aapt` badging, and package dump are artifacts of the pinned
Tauri generation on each run. They are the source of truth for the observed compile/min/target SDK
values; API 35 in the runner setup is not itself an Android release-tier decision.

The phone and tablet AVDs cover only the named Google profiles and x86_64 system images. ARM devices,
32-bit-device availability, OEM task killers, battery optimizers, split-screen/windowing, rotation,
accessibility, offline/low-bandwidth operation, push delivery, and Android vendor-specific lifecycle
policies remain separate release evidence. The minimal bootstrap screen is also not an adaptive
tablet or phone management UI.

## Decision rule and residual blockers

A green hosted run makes Tauri Android **feasible for the current P0 typed-boundary build and
x86_64-emulator launch only**. It does not accept an Android support tier or a P5/P6 release. A
failed run is retained with its logs/artifact and identifies the next reproducible host/toolchain or
Tauri boundary to investigate.

Before Android tablet or phone release claims, the project still needs separately owned proof for:

- an ARM physical phone and tablet per supported ABI/API tier;
- a Rust-owned secure credential adapter and device-loss/revocation behavior;
- Fleet/ACP networking, interrupted-operation recovery, and measured reconnect semantics;
- deliberate deep-link, notification, API-level, signing, upgrade, and store-distribution designs;
- background restrictions on selected OEM/vendor devices; and
- adaptive, accessible full-management phone and tablet workflows.
