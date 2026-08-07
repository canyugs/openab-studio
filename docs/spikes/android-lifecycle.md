# Android phone and tablet Tauri lifecycle spike

- **Issue:** [#13](https://github.com/canyugs/openab-studio/issues/13)
- **Scope:** P0 build-and-launch feasibility for the existing typed `workspace_bootstrap` Rust/TypeScript seam.
- **Evidence workflow:** [Android lifecycle spike](https://github.com/canyugs/openab-studio/actions/workflows/spike-android.yml)
- **Decision status:** feasible for the current typed P0 seam and x86_64 Google-AVD launch only; this is not a phone/tablet release decision. [Run 31154613064](https://github.com/canyugs/openab-studio/actions/runs/31154613064) passed the build, phone, and tablet jobs.

## Boundary

This spike deliberately changes neither the product shell nor Android source. The build job creates
`apps/studio/src-tauri/gen/android` only on a disposable GitHub-hosted runner, builds it, captures
build evidence, and lets that runner disappear. Separate, fresh GitHub-hosted runners download only
the inspected APK artifact to exercise the phone and tablet emulators. Generated Android source,
build outputs, AVD state, debug keys, SDK/NDK/JDK setup, and emulators are never committed and are
never created on a developer host.

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
dependency, Gradle, or AVD cache. Every action is commit-SHA pinned. Its `build-apks` job uses an
Ubuntu 24.04 GitHub-hosted runner, Rust 1.85, Node 22.12.0, pnpm 10.29.2, Temurin JDK 21, Android
SDK platform 35, build-tools 35.0.0, and NDK 27.2.12479018.

The build job:

1. installs all four Tauri Android Rust targets and generates the Android project with
   `tauri android init --ci --skip-targets-install`;
2. builds debug, split APKs for `aarch64`, `armv7`, `i686`, and `x86_64`;
3. archives generated Gradle and manifest metadata, each APK's native libraries and debug-signing
   verification, paths, checksums, and source-tree boundary; and
4. uploads the x86_64 APK and four-ABI build evidence as `android-build-spike-<run-id>` for 14 days.

The independent `phone` and `tablet` lifecycle jobs each receive that artifact at `android-build`,
set up only JDK 21 plus the emulator's Android platform tools/API 35, and launch the x86_64 APK on
one fresh Google APIs AVD: Pixel 7 or Pixel Tablet. They capture a foreground launch
screenshot/activity dump, Home/background process/task state, foreground return, force-stop process
absence, cold relaunch, package data, a deep-link attempt, and logcat. Each job emits its own
`android-lifecycle-<phone|tablet>-<run-id>` artifact for 14 days.

For [run 31154613064](https://github.com/canyugs/openab-studio/actions/runs/31154613064), the tablet
action requested `ram-size: 2048M` and `heap-size: 256` after an earlier default-profile attempt was
killed with exit 137 during its first UI capture. The action wrote `hw.ramSize=2048M` and
`hw.heapSize=256`, but the Android Emulator then logged `Increasing RAM size to 4096MB` and booted
with `androidboot.dalvik.vm.heapsize=576m`. The successful tablet run is evidence for that observed
runtime environment only: it neither establishes the prior kill's cause nor shows that a 2GiB/256M
cap caused recovery. The requested values are a hosted-runner configuration, not a claim about
physical Pixel Tablet capacity or Android tablet requirements.

The runner boundary is intentional: the first post-build attempt showed that cold four-ABI
compilation plus emulator installation can exhaust an Ubuntu runner's disk. Lifecycle jobs therefore
do not check out the repository, generate Android source, install an NDK, or compile. The handoff
records the downloaded x86_64 APK checksum and free disk before the AVD starts.

Artifacts include no release or debug private key. A debug certificate fingerprint may appear in
`apksigner` output and is evidence only, not a signing identity.

## Hosted evidence

### Final result

[Run 31154613064](https://github.com/canyugs/openab-studio/actions/runs/31154613064) completed green
for the four-ABI build, fresh phone lifecycle, and fresh tablet lifecycle jobs. Its build artifact is
[8984811069](https://github.com/canyugs/openab-studio/actions/runs/31154613064), with independently
retained phone [8984872344](https://github.com/canyugs/openab-studio/actions/runs/31154613064) and
tablet [8984866437](https://github.com/canyugs/openab-studio/actions/runs/31154613064) evidence.

Both lifecycle runners installed the same inspected x86_64 APK checksum
`70d16426709cd78cd7b8a81c9b56bedbd6edaf9deffda5f7413cc8e5d3d342be`, rendered
`Trusted core ready (protocol 1).`, sent `dev.openab.studio/.MainActivity` to the launcher with the
process still present after Home, returned it to the foreground, observed no process after
`am force-stop`, and cold-launched it again. The phone profile recorded 1080x2400 at 420 dpi; the
tablet profile recorded 2560x1600 at 320 dpi. The deliberately unsupported
`openab-studio://spike` deep-link attempt remained unresolved on both profiles.

The tablet workflow's requested `2048M` RAM and `256` heap values were not the effective values
reported by the emulator: it logged an increase to 4096MB RAM and a 576m Dalvik heap. Therefore, the
green tablet job proves the observed final emulator environment, not that those requested values
fixed the earlier exit 137. No root cause has been established for that earlier termination.

### Earlier diagnostic attempts

| Run                                                                                                               | Result                                                                                                                                                                                             | Evidence retained                                                                                                                                                                                                                                                             | What it establishes                                                                                                                                                                                                                                                              |
| ----------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [31149533175](https://github.com/canyugs/openab-studio/actions/runs/31149533175)                                  | Built all four split debug APKs, then stopped before emulator setup because `rg` was absent on the Ubuntu 24.04 image.                                                                             | `android-lifecycle-spike-31149533175`, artifact [8983019262](https://github.com/canyugs/openab-studio/actions/runs/31149533175)                                                                                                                                               | Tauri generation and the ABI build reached the inspection step; no phone/tablet lifecycle evidence. The unsupported `rg` invocation was replaced with portable `grep`/`find` in `638b809`.                                                                                       |
| [31150001027](https://github.com/canyugs/openab-studio/actions/runs/31150001027)                                  | Built and inspected all four ABIs, then Android Emulator package preparation failed with `No space left on device`.                                                                                | `android-lifecycle-spike-31150001027`, artifact [8983166697](https://github.com/canyugs/openab-studio/actions/runs/31150001027)                                                                                                                                               | This is a hosted disk/tooling failure, not app lifecycle evidence: no AVD booted and neither foreground/background/termination/relaunch behavior was measured. It motivated the fresh-runner artifact handoff.                                                                   |
| [31151264887](https://github.com/canyugs/openab-studio/actions/runs/31151264887)                                  | The split build completed and fresh phone/tablet AVDs booted, but the lifecycle script exited before installation because the action invokes `/usr/bin/sh` and does not support `set -o pipefail`. | Build artifact [8983527105](https://github.com/canyugs/openab-studio/actions/runs/31151264887); phone [8983566903](https://github.com/canyugs/openab-studio/actions/runs/31151264887); tablet [8983567860](https://github.com/canyugs/openab-studio/actions/runs/31151264887) | The fresh-runner disk boundary and both AVD profiles are viable. This is still not lifecycle proof because no APK was installed or launched. Replacing `pipefail` exposed the action's line-by-line execution in the next run.                                                   |
| [31152211001](https://github.com/canyugs/openab-studio/actions/runs/31152211001)                                  | The minimized x86_64 artifact was downloaded and both AVDs booted, but the action executed the `apk` assignment and its next `test` in separate shells, so no variable survived into installation. | Build artifact [8983881402](https://github.com/canyugs/openab-studio/actions/runs/31152211001); phone [8983923360](https://github.com/canyugs/openab-studio/actions/runs/31152211001); tablet [8983925172](https://github.com/canyugs/openab-studio/actions/runs/31152211001) | The final handoff shape is viable, but this remains pre-install evidence. The probe is now one POSIX `sh -eu -c` command so the artifact path and lifecycle state share one shell.                                                                                               |
| [31153815313](https://github.com/canyugs/openab-studio/actions/runs/31153815313)                                  | The build and phone lifecycle passed. The tablet installed and launched the x86_64 APK, reached its UI dump, then the probe process was killed with exit 137 before the first screenshot.          | Build artifact [8984494717](https://github.com/canyugs/openab-studio/actions/runs/31153815313); phone [8984555267](https://github.com/canyugs/openab-studio/actions/runs/31153815313); tablet [8984550022](https://github.com/canyugs/openab-studio/actions/runs/31153815313) | The phone has complete lifecycle proof. The tablet has pre-capture launch proof only. The log does not establish the kill cause. The 4GiB default AVD RAM is only a memory-pressure hypothesis; the next run requested lower AVD configuration values but was not a causal test. |
| All build-producing attempts used the generated package `dev.openab.studio`; the emitted Gradle metadata observed |
| `minSdk=24`, `compileSdk=36`, and `targetSdk=36`. The AVD test image is API 35. These are distinct                |
| facts: SDK platform 35 in setup and the API 35 emulator do not by themselves select a release SDK                 |
| tier, while the generated Gradle metadata is the source of truth for its observed build SDK values.               |

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
`openab-studio://spike` launch attempt as a negative configuration check; it does not claim deep-link
support. There is also no notification permission request, notification adapter, or notification
delivery path in the current seam, so the spike records this as unimplemented rather than
manufacturing a notification test.

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

A green hosted build job plus both green lifecycle jobs makes Tauri Android **feasible for the
current P0 typed-boundary build and x86_64-emulator launch only**. It does not accept an Android
support tier or a P5/P6 release. A failed run is retained with its logs/artifact and identifies the
next reproducible host/toolchain or Tauri boundary to investigate.

Before Android tablet or phone release claims, the project still needs separately owned proof for:

- an ARM physical phone and tablet per supported ABI/API tier;
- a Rust-owned secure credential adapter and device-loss/revocation behavior;
- Fleet/ACP networking, interrupted-operation recovery, and measured reconnect semantics;
- deliberate deep-link, notification, API-level, signing, upgrade, and store-distribution designs;
- background restrictions on selected OEM/vendor devices; and
- adaptive, accessible full-management phone and tablet workflows.
