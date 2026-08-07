# iOS and iPadOS Tauri lifecycle spike

- **Issue:** [#12](https://github.com/canyugs/openab-studio/issues/12)
- **Workstream:** W6 platform and release
- **Status:** The complete hosted run
  [31158300888](https://github.com/canyugs/openab-studio/actions/runs/31158300888) OCR-asserted the
  typed seam in distinct iPhone and iPad foreground screenshots plus an iPhone cold-relaunch
  screenshot. This establishes simulator-path feasibility for the existing shared shell, not a
  release decision. Historical run
  [31150578770](https://github.com/canyugs/openab-studio/actions/runs/31150578770) remains a partial
  result and does not meet that criterion.
- **Native-project boundary:** `apps/studio/src-tauri/gen/apple` is generated only on the disposable
  macOS Actions runner. It is intentionally neither committed nor generated on a contributor host by
  this spike.

## Question and decision boundary

Can the existing Tauri 2 shell generate, build, install, and launch on current iPhone and iPad
simulators while invoking its typed Rust/TypeScript bootstrap seam? This is an implementation
feasibility spike for the installed, remote-first client selected in the
[remote-first ADR](../adr/remote-first-client.md). It is not a mobile release decision.

The product goal remains full remote Fleet and instance management from phones and tablets. A mobile
device may manage remote or instance-hosted plugins; it must not be treated as a host for arbitrary
local `mcp-stdio` executables. That distinction follows the
[architecture placement matrix](../architecture.md#platform-and-placement-matrix) and the
[plugin-platform ADR](../adr/plugin-platform.md#runtime-placements).

## Reproducible hosted evidence

Run [`.github/workflows/spike-ios.yml`](../../.github/workflows/spike-ios.yml) with
`workflow_dispatch`, or let its scoped pull-request trigger run. It uses a disposable `macos-15`
 runner and selects its installed iPhone and iPad simulators from the newest available iOS runtime
 that has both form factors.
 The runner is disposable, so those simulator instances are still ephemeral. It records its exact
 macOS, Xcode, simulator SDK, Rust, Node, pnpm, Tauri CLI, CocoaPods, selected device model/runtime,
 and initial simulator state in the artifact rather than assuming a stable runner image.

The initial run on `macos-14` is retained as a failed evidence artifact: it generated the Apple
project and booted both ARM simulators, but its Xcode 15.4 failed to open the generated project-file
format 77. That is a hosted-toolchain compatibility finding, not a product or signing failure. The
workflow therefore selects `macos-15`, whose supported Xcode family is compatible with the generated
project format; every run still records the exact image/Xcode pair before it makes a support claim.

The workflow deliberately has `contents: read`, disables checkout credentials and package-manager
caches, uses full-commit action pins, and supplies no signing or Apple-account secret. If CocoaPods
is absent, the pinned gem is installed only on that disposable runner. The workflow then:

1. adds only `aarch64-apple-ios-sim` and initializes the native project with
   `tauri ios init --ci --skip-targets-install`;
2. gives selected runner-local iPhone and iPad simulators unique disposable names, then boots them
   serially with an explicit readiness bound;
3. uses the pinned local Tauri CLI to run the existing app on the iPhone simulator;
4. explicitly launches the app on each simulator, verifies the reported app PID is still alive, and
   retries each screenshot every five seconds for up to one minute until runner-local Swift/Vision
   OCR finds `Trusted core ready`; and
5. asks `simctl` to terminate the iPhone app, cold-launches it again, and captures a third
   OCR-asserted screenshot.

The uploaded `ios-ipados-lifecycle-evidence-<run-id>` artifact retains the init and launch logs,
generated-project build settings, selected simulator metadata, app binary architecture, lifecycle
command output, device diagnostics, screenshots, app-PID probes, and OCR transcripts. Vision OCR
runs locally on the disposable runner; screenshots are retained in the 14-day GitHub Actions
artifact and are not sent to any external recognition service. The OCR assertion verifies that each
final screenshot contains the current TypeScript UI's `workspace_bootstrap` result:
`Trusted core ready (protocol 1).` It is not a substitute for a broader mobile UI test.

## Hosted results

### Complete simulator result

Run [31158300888](https://github.com/canyugs/openab-studio/actions/runs/31158300888) on commit
`79a68baad068d40d0fe706a35f6b9baf6b95ade5` is the complete feasibility result. Its artifact
`ios-ipados-lifecycle-evidence-31158300888` (artifact `8986585346`, retained until
2026-08-21T07:59:01Z) records macOS 15.7.7, Xcode 16.4, iOS simulator SDK 18.5, and the runner's
iOS 26.2 iPhone 17 Pro and iPad Pro 13-inch (M5) simulators. The generated simulator executable is
arm64, has `MinimumOSVersion` 14.0, and declares both iPhone and iPad device families.

Each reported launch PID passed its simulator liveness probe, and the runner-local Vision OCR
assertion passed on its first attempt for all three screenshots: iPhone foreground (PID 53182), iPad
foreground (PID 58972), and iPhone cold relaunch (PID 60268). The cold-process evidence records
`simctl terminate` at 07:57:54Z, the cold launch request at 07:57:57Z, and the asserted relaunch
screen at 07:58:01Z. The iPad foreground screenshot visibly renders `Trusted core ready (protocol
1).`; it is not a Home Screen capture.

### Earlier partial result

Run [31150578770](https://github.com/canyugs/openab-studio/actions/runs/31150578770) on artifact
`ios-ipados-lifecycle-evidence-31150578770` (artifact `8983542691`, retained until
2026-08-21T05:47:28Z) is a **partial pass**, not a feasibility conclusion. It ran macOS 15.7.7,
Xcode 16.4, iOS simulator SDK 18.5, and the runner's iOS 26.2 iPhone 17 Pro and iPad Pro 13-inch
(M5) simulators. Its iPhone foreground and cold-relaunch screenshots show the typed seam, and the
generated simulator binary is arm64 with `MinimumOSVersion` 14.0.

The iPad `simctl launch` command returned PID 53737, but the fixed ten-second screenshot captured
the iPad Home Screen. Its device log records the first WebKit commit about 27 seconds after launch,
so the screenshot was too early and cannot establish the tablet seam. The workflow therefore now
checks process liveness and waits for OCR-confirmed seam text instead of accepting a timed sleep.
The run also records that Tauri installed a missing `ios-deploy` tool only on the disposable runner;
no contributor-host package or mobile initialization was performed.

### Cancelled diagnostic

Run [31156525248](https://github.com/canyugs/openab-studio/actions/runs/31156525248) was
intentionally cancelled during an elapsed-time investigation; it is neither a product failure nor
feasibility evidence. Its diagnostic artifact `8985972075` contains no screenshots or app-container
path. Its `iphone-dev.log` reaches `BUILD SUCCEEDED`, then `Deploying app to device...` and the
Tauri-reported PID 70471, so it cannot establish where the remaining transition was waiting or
whether cleanup was involved. Commit `79a68ba` nevertheless bounds background `tauri ios dev`
cleanup with TERM, a short poll, then KILL before waiting, preventing an unbounded cleanup wait in
future hosted runs.

## What each result can establish

| Area | Automated evidence | What it establishes | What it does not establish |
|---|---|---|---|
| Rust/TypeScript seam | OCR-asserted foreground screenshots from both simulators contain the current UI's `workspace_bootstrap` result. | The generated native shell can render the existing typed command boundary on the selected simulator runtime. | A management API mutation, ACP session, auth, storage, or reconnection flow. |
| iPhone | A selected runner-local iPhone simulator receives the generated app, reports a live PID, and has an OCR-asserted foreground screenshot. | Phone simulator build/install/launch for the recorded device model/runtime. | Small-screen usability, accessibility, one-handed flow, interruption recovery, cellular behavior, or physical-phone behavior. |
| iPad | The same generated simulator app is installed, reports a live launch PID, and has an OCR-asserted foreground screenshot. | Tablet simulator install/launch and the common shell's basic runtime path. | Adaptive/dense management UI, rotation, Split View, Stage Manager, external display, keyboard/trackpad, or physical-iPad behavior. |
| Termination and cold relaunch | `simctl terminate` succeeds, followed by `simctl launch`, a live PID, and an OCR-asserted relaunch screenshot. | The simulator accepts a process termination and the app can be launched again from a cold process. | State persistence, graceful lifecycle callbacks, pending-mutation recovery, or transport reconnection. |
| Background/foreground transition | No artificial background transition is attempted. | Nothing beyond the foreground launch above. | Background suspension, resume, WebSocket survival, background task scheduling, push wake-up, or reconnect behavior. These remain unmeasured. |
| Reconnect | The bootstrap shell currently has no Fleet Management API or ACP-over-WebSocket transport. | Nothing; the absence is explicit. | Network-loss recovery or session resume. That needs the future transport/state-machine work and a physical-device experiment. |

The separation is intentional: `simctl terminate` is a cold-process test, not a background test, and
a simulator cannot stand in for device notification or background-execution policy. The present
bootstrap command also returns immediately and has no lifecycle or reconnect instrumentation, so no
stronger lifecycle claim would be evidence-based.

## Platform and security constraints recorded by this spike

| Concern | Current observation / evidence source | Release implication |
|---|---|---|
| Secure storage | No secret broker or iOS Keychain adapter exists in the current shell; the simulator run supplies no credentials. | C-03 needs an opaque-handle adapter, denial/redaction tests, and a physical-device Keychain migration/revocation matrix before a credential claim. |
| Networking | The current `tauri.conf.json` CSP permits same-origin content and Tauri IPC only. The bootstrap has no remote Fleet/ACP request. A dev simulator uses Tauri/Vite development networking, not a production management connection. | Add authenticated HTTPS/WSS transport, ATS policy, certificate/error behavior, offline handling, and real network-loss/reconnect evidence before claiming remote management. |
| Deep links | No deep-link configuration or handler is present in the current app. | Define link ownership, authorization/replay rules, and iOS URL/universal-link registration before implementing or claiming links. |
| Notifications | No notification plugin, permission request, APNs entitlement, or event handler is present. | A signed physical-device build and APNs/push-background proof are required; simulator launch is not notification evidence. |
| Signing | The simulator path uses no Apple signing secret, team, provisioning profile, IPA export, or App Store operation. | A device/release pipeline needs an approved team, certificate/profile handling, entitlement review, and separate signing/revocation evidence. |
| Minimum OS / architecture | The artifact includes the generated Xcode `IPHONEOS_DEPLOYMENT_TARGET`/`ARCHS` settings plus the built simulator executable's `lipo` result. The hosted runner and simulator runtime are recorded exactly. | Those values describe one generated build, not an accepted support tier. Physical iPhone/iPad is expected to be arm64; x86_64 simulator and a physical device remain untested. R-04 owns a release-tier decision. |

No credential, certificate, Apple account, physical-device identifier, production endpoint, or
notification token is used by the workflow. The artifact records only the disposable runner's
simulator IDs. Generated native files and app artifacts are retained only in the short-lived Actions
evidence artifact.

## Feasibility criteria and residual blockers

The complete run above is evidence that Tauri remains feasible for the existing shared shell's ARM
simulator path on both form factors. It does **not** clear P5/P6, grant iOS/iPadOS release support,
or prove full remote management. The remaining release blockers are:

- device signing, entitlements, installation, and physical iPhone/iPad validation;
- trusted storage and device enrollment;
- Fleet Management API and ACP transport, including authenticated reconnect and idempotent
  interrupted mutations;
- background suspension/resume, push-assisted awareness, and notification permission behavior;
- adaptive phone/tablet management UX and accessibility; and
- an explicit minimum OS/architecture support-tier decision in R-04.

If generation, build, installation, launch, or the displayed typed seam fails, attach the artifact
and treat that failure as the next platform blocker rather than silently retrying on a developer
machine.
