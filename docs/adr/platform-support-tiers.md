# ADR: Platform Support and Release Tiers

- **Status:** Proposed
- **Date:** 2026-08-07
- **Tracking:** [R-04 / issue #14](https://github.com/canyugs/openab-studio/issues/14)
- **Related:** [Remote-first client](./remote-first-client.md),
  [system architecture](../architecture.md), and the
  [desktop](../spikes/desktop-packaging.md),
  [iOS/iPadOS](../spikes/ios-ipados-lifecycle.md), and
  [Android](../spikes/android-lifecycle.md) spikes

## Context

OpenAB Studio targets macOS, Windows, Linux, iPadOS, iOS, Android tablets, and Android phones. A
shared Tauri/Rust/TypeScript source tree does not prove that a package is installable, secure,
usable, or supportable on any one operating-system and CPU combination.

The P0 spikes established build and narrow lifecycle feasibility on named hosted environments. They
did not exercise product networking, secure storage, updates, rollback, accessibility, real mobile
devices, or signed public distribution. Downloaded macOS dogfooding also found a gap hidden by the
hosted launch probe: the ad-hoc-signed application fails strict bundle-signature validation and is
blocked by Gatekeeper after a normal quarantined download.

The release policy must keep broad product reach without turning unmeasured combinations into a
support promise. Minimum deployment values emitted by a generator are build facts, not release
floors. CI availability and end-user evidence are tracked separately.

## Decision

Use four externally meaningful platform tiers:

| Tier | Meaning | Required release behavior |
|---|---|---|
| **Supported** | A normal public release commitment for a named OS range and architecture. | Every release is built and signed, passes install/launch/update/rollback/removal checks, and receives product and security fixes while its vendor OS remains supported. |
| **Tested preview** | Users may install it, but a documented platform limitation remains. | CI and release artifacts are maintained; known limitations are release-noted; promotion requires closing the named evidence gaps. |
| **Best effort** | The package may work, but it is not a release gate. | Breakage is accepted unless a maintainer promotes the combination through a separate decision and test matrix. |
| **Not supported** | No distributable package or compatibility promise is made. | Builds may exist as toolchain or emulator evidence and must be labelled development-only. |

`Feasibility evidence` is deliberately not a fifth user-facing tier. It records what P0 measured
while every current artifact remains non-release evidence. No combination is **Supported** or
**Tested preview** merely because this ADR names its proposed destination tier.

### Proposed initial release matrix

| Surface | Proposed minimum OS | Proposed architecture and tier | Evidence today | Status before promotion |
|---|---|---|---|---|
| macOS desktop | macOS 15 | arm64 **Supported** | Native macOS 15 arm64 bundle/copy/launch plus local downloaded-artifact dogfood | Feasibility only; downloaded app is Gatekeeper-blocked |
| macOS desktop | macOS 15 | x86_64 **Tested preview** | No native bundle/install/launch run | Not supported |
| Windows desktop | Windows 11 24H2 | x86_64 **Supported** | Windows Server 2022 x64 NSIS install and launch on a hosted runner | Feasibility only; no Windows 11 user install evidence |
| Windows desktop | Windows 11 24H2 | arm64 **Tested preview** | No native runner or device evidence | Not supported |
| Linux desktop | Ubuntu 22.04 LTS while under standard maintenance; Ubuntu 24.04 LTS thereafter | x86_64 **Supported** | Ubuntu 22.04 x64 `.deb` install and Xvfb launch | Feasibility only; no signed repository/update or visible desktop test |
| Linux desktop | Ubuntu 24.04 LTS | arm64 **Tested preview** | No native bundle/install/launch run | Not supported |
| Other desktop Linux distributions | Vendor-supported release | x86_64/arm64 **Best effort** through a portable package | No distribution-specific install evidence | Not supported |
| iPhone | iOS 18 | physical-device arm64 **Supported** in P6 | arm64 iOS 26.2 simulator foreground and cold relaunch | Feasibility only; no signed physical-device run |
| iPad | iPadOS 18 | physical-device arm64 **Supported** in P5 | arm64 iPadOS 26.2 simulator foreground launch | Feasibility only; no signed physical-device run |
| Android phone/tablet | Android 10 / API 29 | arm64-v8a **Supported** | Four-ABI debug build; API 35 x86_64 phone/tablet emulator lifecycle | Feasibility only; no ARM physical-device run |
| Android phone/tablet | Android 10 / API 29 | armeabi-v7a **Best effort** | Build inspection only | Not supported |
| Android emulator/ChromeOS-like environments | Android 10 / API 29 | x86_64 **Best effort** | API 35 Google phone/tablet emulator lifecycle | Not supported for end users |
| Android phone/tablet | Any | i686 | Build inspection only; no intended modern device release surface | Not supported |

The iOS/iPadOS generated `MinimumOSVersion=14.0` and Android generated `minSdk=24` remain recorded
toolchain facts. The proposed iOS/iPadOS 18 and Android API 29 release floors are intentionally
higher until secure storage, transport interruption, and background policy have real-device
matrices. Lowering either floor is a later measured compatibility decision, not a package-generator
default.

Windows 10 is excluded from the initial promise because Microsoft ended its ordinary support on
2025-10-14 and recommends moving to Windows 11; paid ESU and LTSC variants do not create a general
consumer support tier for Studio. See the
[Microsoft lifecycle FAQ](https://learn.microsoft.com/en-us/lifecycle/faq/windows).

Ubuntu 22.04 remains an explicit, time-bounded candidate because Canonical provides standard
maintenance only through April 2027. The release matrix must move its minimum to Ubuntu 24.04 before
that date unless the project separately accepts an Ubuntu Pro/ESM support obligation. See the
[Ubuntu 22.04 support lifespan](https://documentation.ubuntu.com/release-notes/22.04/).

Apple and Google distribution toolchains continue to move independently of deployment floors. As of
this decision, Apple requires App Store uploads to use the iOS/iPadOS 26 SDK or later, and Google
Play requires new apps and updates to target Android API 36 from 2026-08-31. Release automation must
follow those current submission requirements without silently raising the minimum install OS. See
[Apple's submission requirements](https://developer.apple.com/app-store/submitting/) and
[Google Play's target API policy](https://developer.android.com/google/play/requirements/target-sdk).

### Evidence behind the current status

| Family | Final P0 run and artifact | Highest evidence earned | Important limit |
|---|---|---|---|
| Desktop | [run 31150124485](https://github.com/canyugs/openab-studio/actions/runs/31150124485); artifacts `8983019776`, `8983151791`, and `8983142682` | Native bundle, isolated install, ten-second process liveness, and WebView/runtime inspection for one architecture per OS family | No signed/notarized package, normal downloaded install, update, rollback, or visible interaction proof |
| iOS/iPadOS | [run 31160635948](https://github.com/canyugs/openab-studio/actions/runs/31160635948); artifact `8987905126` | Current iPhone/iPad simulator foreground UI plus iPhone termination/cold relaunch | Simulator only; no background/reconnect, signing, store, Keychain, or physical-device evidence |
| Android | [run 31156391442](https://github.com/canyugs/openab-studio/actions/runs/31156391442); artifacts `8985478367`, `8985544108`, and `8985540551` | Four-ABI debug build plus API 35 x86_64 phone/tablet emulator foreground/background/force-stop/cold launch | Only x86_64 was run; no release key, store, Keystore, OEM, or physical ARM evidence |

The macOS artifact's DMG SHA-256 is
`f4b812bceb56a7da61dc63a0f9f4b91484bad0d77388f19897dabf64e5b34b60`, matching the CI evidence,
and `hdiutil verify` reports a valid disk image. The failure is therefore not a corrupt download.
The contained app reports an ad-hoc signature with no Team identifier, and
`codesign --verify --deep --strict` fails because the bundle has no sealed resources although its
signature says resources must be present. A normal Chrome download carries quarantine and Gatekeeper
reports the app as damaged. Removing quarantine is permitted only as an explicit developer
workaround after hash verification; it is not install evidence and never appears in user-facing
release instructions.

## Promotion gates

No platform moves to **Tested preview** or **Supported** until its bounded release task records:

1. a package built from the release commit with provenance and durable retention;
2. platform-native release signing, notarization or store validation, and revocation ownership;
3. a normal quarantined/downloaded or store install without security bypasses;
4. visible UI launch plus authenticated Fleet/ACP behavior on the minimum OS and architecture;
5. secure credential storage, redaction, device loss/revocation, and migration evidence;
6. network loss, reconnect/resume, suspend/resume, and interrupted-operation behavior;
7. update, rollback, downgrade rejection, uninstall, and user-data retention/removal behavior; and
8. accessibility and adaptive-layout evidence for the claimed device class.

Additional promotion proof is platform-specific:

- macOS: Developer ID signing, hardened runtime where applicable, notarization, stapling, strict
  `codesign` verification, and Gatekeeper assessment from a quarantined download;
- Windows: Authenticode signing/timestamping, SmartScreen-aware install, WebView2 policy, and a real
  supported Windows 11 device or VM;
- Linux: package/repository signature policy, supported WebKitGTK versions, visible Wayland/X11
  launch, desktop integration, and repository/update removal behavior;
- iOS/iPadOS: approved signing/provisioning, TestFlight or App Store install, physical phone/tablet,
  Keychain, background suspension, notification, and device rotation/windowing evidence; and
- Android: release/App Bundle signing, Play policy, physical ARM phone/tablet, Keystore, OEM
  background restrictions, rotation/multi-window, notification, and upgrade evidence.

Every release workflow records OS build, CPU, package digest, signing identity, install source, and
evidence level. CI emulators remain useful regression gates but cannot replace a required real-device
or downloaded-package gate.

## Consequences

- Every requested OS family remains on the product roadmap, while launch order and architecture
  breadth stay evidence-driven.
- The first public desktop release can be narrow without making mobile a read-only companion.
- Older OS versions and secondary CPU architectures require explicit test ownership instead of
  inheriting support from a successful cross-compile.
- Minimum OS changes are release-contract changes and require release notes, migration impact, and a
  Project decision; routine SDK/target API updates are not automatically minimum-OS changes.
- The matrix is reviewed at each phase boundary and whenever a vendor OS leaves standard security
  maintenance.

## Non-goals

- Claiming that any current P0 artifact is a public release.
- Promising dates or simultaneous platform launches.
- Resuming the paused browser-hosted Web App.
- Supporting every Linux distribution, obsolete OS, or compiler target merely because it builds.
- Changing mobile's full-management product requirement.

## Acceptance criteria

- Every target OS family and observed CPU architecture has an explicit proposed or rejected tier.
- Unsupported combinations name the missing measured evidence rather than a generic platform label.
- Hosted CI, emulator/simulator, physical-device, and normal user install evidence remain distinct.
- Promotion gates identify the concrete work required before a public support claim.
