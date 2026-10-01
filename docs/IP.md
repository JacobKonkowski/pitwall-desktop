# Intellectual property hygiene

This document describes how PitWall Desktop handles **copyright**, **third-party
licenses**, **trademarks**, and **patents**. It is an engineering checklist, not
legal advice. Patent freedom-to-operate and trademark clearance require counsel.

## Copyright (our code)

- Original PitWall Desktop source and docs are licensed under the MIT License
  ([LICENSE](../LICENSE)).
- Do **not** paste proprietary SDK samples, game assets, branded audio, track
  maps from commercial products, or substantial third-party source into this
  tree without a compatible license and attribution in [NOTICE](../NOTICE).
- Prefer clean-room implementations against published telemetry **interfaces**
  (channel names, shared-memory layout docs, OpenXR headers) over copying
  another vendor’s client.

## Third-party software licenses

| Surface | Enforcement |
|---------|-------------|
| Rust crates | [`deny.toml`](../deny.toml) + `cargo deny check licenses bans sources` in CI |
| npm packages | `npm run check:licenses` (allowlist in [`scripts/check-npm-licenses.mjs`](../scripts/check-npm-licenses.mjs)) |
| Human-readable inventory | [NOTICE](../NOTICE); refresh with `npm run gen:notice` before tagged releases |

Allowed SPDX sets are intentionally permissive (MIT/Apache/BSD/ISC/MPL-2.0 and
similar). Copyleft that would force this app under GPL/AGPL is denied for Rust
and blocked by the npm allowlist.

### Key redistributed / build-time dependencies

| Component | Role | License (typical) |
|-----------|------|-------------------|
| [`pitwall`](https://crates.io/crates/pitwall) (We Race) | Live SHM + IBT reader | MIT |
| Tauri / wry / webview stack | Desktop shell | Apache-2.0 / MIT |
| OpenXR-SDK headers (Khronos) | Fetched at OpenXR layer build | Apache-2.0 |
| React, Vite, Recharts, etc. | Frontend / build | MIT / ISC / Apache-2.0 / CC-BY-* (data) |

Exact texts live in each package’s LICENSE file on crates.io / npm.

## Trademarks and product names

PitWall Desktop is an independent project. Unless a written license says
otherwise:

- **iRacing** and related marks are trademarks of iRacing.com Motorsport
  Simulations, LLC. Use them only to describe compatibility (nominative use).
  Do not imply endorsement, sponsorship, or official partnership.
- **OpenXR** and **Khronos** are trademarks of the Khronos Group.
- **OpenKneeboard**, **RaceLab**, headset vendors, and other third-party tools
  are mentioned for interoperability only.
- The crates.io name **`pitwall`** and the “We Race” / Pitwall telemetry stack
  are third-party branding. This app **depends on** that MIT-licensed crate; it
  is not the We Race product. Avoid implying ownership of their marks. If you
  commercialize or widen distribution, get trademark advice on the “PitWall”
  product name relative to We Race and iRacing.

UI and marketing copy should prefer “works with iRacing” / “reads iRacing
telemetry” over “official iRacing …” wording.

## iRacing telemetry interface

PitWall reads data iRacing exposes for third-party tools:

- Live: Windows shared memory (`Local\IRSDKMemMapFileName`) when enabled in
  `app.ini`
- Disk: user-recorded `.ibt` files under the user’s telemetry folder

Engineering constraints we follow:

1. **Read-only** telemetry consumption — do not write into the IRSDK map or
   claim to mutate sim state through the SDK.
2. **No cheats / unauthorized experience mods** — coaching and HUDs that display
   information already available to the driver are the intent; do not add
   features that circumvent anti-cheat or alter the sim client.
3. **Local-only by default** — see [PRIVACY.md](PRIVACY.md). Do not upload IBT or
   live session data without an explicit, documented user action and policy.
4. Revisit iRacing’s current Terms of Use / EULA and any Commercial Software
   Terms before **commercial** distribution or cloud features.

iRacing’s published terms can change; treat this section as a maintainer
checklist, not a compliance certificate.

## OpenXR layer

The `openxr-layer/` DLL is original PitWall code. Structure follows the common
implicit-layer pattern documented by Khronos and illustrated by community
templates (e.g. Ybalrid/OpenXR-API-Layer-Template, MIT). We do not vendor that
template’s source tree; if you copy substantial template code later, keep its
copyright notice and MIT terms in NOTICE.

OpenXR headers are pulled via CMake `FetchContent` from Khronos OpenXR-SDK
(Apache-2.0). Attribute Khronos in NOTICE; do not claim Khronos endorsement.

## Audio assets

Coach WAV clips under `src-tauri/resources/audio/coach/default/` are generated
locally via Windows speech APIs (`scripts/generate-audio-clips.ps1`). Do not
commit voice packs, spotter packs, or music from commercial products. If you
replace the default voice with a licensed third-party voice, record the license
in NOTICE.

## Patents

**No repository process can prove patent non-infringement.**

- Software patents (telemetry HUDs, coaching UIs, VR overlays, seqlocks, etc.)
  may exist independently of copyrighted source.
- Apache-2.0 dependencies grant a limited patent license **for those
  contributions**; that does not cover unrelated third-party patents or the iRacing
  platform itself.
- Before fundraising, App Store distribution, or selling the product, obtain a
  **freedom-to-operate** review from qualified counsel in relevant jurisdictions.
- If you knowingly implement a patented method, document the patent number and
  license status in NOTICE — or do not ship it.

## Maintainer checklist (each release)

1. `cargo deny check licenses bans sources`
2. `npm run check:licenses`
3. `npm run gen:notice` and merge meaningful changes into [NOTICE](../NOTICE)
4. Skim new files for pasted proprietary samples or unlicensed media
5. Confirm README trademark disclaimer still accurate
6. For commercial plans: counsel review of iRacing terms + trademarks + patents

## Related

- [LICENSE](../LICENSE) — MIT for this repository’s original code
- [NOTICE](../NOTICE) — third-party attributions
- [PRIVACY.md](PRIVACY.md) — local data handling
- [SECURITY.md](../SECURITY.md) — vulnerability reporting
- [RELEASING.md](RELEASING.md) — tag / installer process
