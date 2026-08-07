# iOS and iPadOS Tauri lifecycle spike

- **Issue:** [#12](https://github.com/canyugs/openab-studio/issues/12)
- **Workstream:** W6 platform and release
- **Status:** Automated simulator evidence is required before this becomes a feasibility result.
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
`workflow_dispatch`, or let its scoped pull-request trigger run. It uses a disposable `macos-14`
runner and creates new iPhone and iPad simulators from the newest available iOS runtime and installed
device types. The runner records its exact macOS, Xcode, simulator SDK, Rust, Node, pnpm, Tauri CLI,
CocoaPods, device type, and runtime versions in the artifact rather than assuming a stable runner
image.

The workflow deliberately has `contents: read`, disables checkout credentials and package-manager
caches, uses full-commit action pins, and supplies no signing or Apple-account secret. If CocoaPods
is absent, the pinned gem is installed only on that disposable runner. The workflow then:

1. adds only `aarch64-apple-ios-sim` and initializes the native project with
   `tauri ios init --ci --skip-targets-install`;
2. boots fresh iPhone and iPad simulators;
3. uses the pinned local Tauri CLI to run the existing app on the iPhone simulator;
4. captures a foreground iPhone screenshot after the app is installed, then installs and launches
   the same simulator app on iPad and captures an iPad screenshot; and
5. asks `simctl` to terminate the iPhone app, cold-launches it again, and captures a third
   screenshot.

The uploaded `ios-ipados-lifecycle-evidence-<run-id>` artifact retains the init and launch logs,
generated-project build settings, selected simulator metadata, app binary architecture, lifecycle
command output, device diagnostics, and screenshots. The screenshots are the visual evidence that
the current TypeScript UI rendered the result of its `workspace_bootstrap` Tauri command:
`Trusted core ready (protocol 1).` They are not a substitute for a future automated mobile UI test.

## What each result can establish

| Area | Automated evidence | What it establishes | What it does not establish |
|---|---|---|---|
| Rust/TypeScript seam | The foreground screenshots from both simulators show the current UI's `workspace_bootstrap` result. | The generated native shell can render the existing typed command boundary on the selected simulator runtime. | A management API mutation, ACP session, auth, storage, or reconnection flow. |
| iPhone | A newly created iPhone simulator receives the generated app and has a foreground screenshot. | Phone simulator build/install/launch for the selected device type. | Small-screen usability, accessibility, one-handed flow, interruption recovery, cellular behavior, or physical-phone behavior. |
| iPad | The same generated simulator app is installed, launched, and captured on a newly created iPad simulator. | Tablet simulator install/launch and the common shell's basic runtime path. | Adaptive/dense management UI, rotation, Split View, Stage Manager, external display, keyboard/trackpad, or physical-iPad behavior. |
| Termination and cold relaunch | `simctl terminate` succeeds, followed by `simctl launch` and a relaunch screenshot. | The simulator accepts a process termination and the app can be launched again from a cold process. | State persistence, graceful lifecycle callbacks, pending-mutation recovery, or transport reconnection. |
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
notification token is used by the workflow. The artifact records only the ephemeral simulator IDs it
creates. Generated native files and app artifacts are retained only in the short-lived Actions
evidence artifact.

## Feasibility criteria and residual blockers

A green run with all three screenshots is evidence that Tauri remains feasible for the existing
shared shell's ARM simulator path on both form factors. It does **not** clear P5/P6, grant iOS/iPadOS
release support, or prove full remote management. The remaining release blockers are:

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
