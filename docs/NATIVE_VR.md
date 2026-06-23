# Native In-Headset VR

PitWall renders its HUD **inside the headset** through its own OpenXR API layer —
the same mechanism RaceLab VR and OpenKneeboard use — so you do not need
OpenKneeboard, RaceLab, or SteamVR overlays to see PitWall in VR.

This reverses the June 2026 no-go in [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md).
That document remains accurate about *why* the work is hard; this one is the
implementation and setup guide.

## Architecture

```
PitWall desktop (Tauri/Rust)              iRacing (OpenXR app)
  LiveService -> LiveSnapshot               |
     |                                       v
  vr::shm::ShmWriter --> Local\PitWallVR --> pitwall-openxr-layer.dll
  (30 Hz, seqlock)        shared memory       hooks xrEndFrame,
                                              draws the HUD with Direct2D,
                                              appends XrCompositionLayerQuad
                                                |
                                                v
                                         Meta / SteamVR / VDXR runtime
```

- **Producer:** [`src-tauri/src/vr/shm.rs`](../src-tauri/src/vr/shm.rs) writes a
  compact mirror of `LiveSnapshot` plus per-overlay placement into the named
  shared-memory block `Local\PitWallVR` at ~30 Hz, guarded by a seqlock.
- **Contract:** [`openxr-layer/include/pitwall_vr_shm.h`](../openxr-layer/include/pitwall_vr_shm.h)
  is the canonical byte layout; the Rust structs mirror it field-for-field.
- **Consumer:** the [`openxr-layer/`](../openxr-layer/) C++ DLL hooks `xrEndFrame`,
  reads the block, draws each enabled overlay with Direct2D/DirectWrite, and
  appends an `XrCompositionLayerQuad`.

The HTTP HUD in [`hud_server.rs`](../src-tauri/src/vr/hud_server.rs) stays as the
browser preview and the visual reference the Direct2D renderer mirrors. Sending
pre-rendered pixels over shared memory (cross-process GPU texture sharing) was
deliberately avoided in v1 for robustness; the layer draws from the snapshot.

## Why a separate native DLL

A Tauri/Rust process cannot composite over another OpenXR app from the outside.
`XR_EXTX_overlay` is unsupported on consumer runtimes, so the only viable path
is an **implicit OpenXR API layer** loaded into the iRacing process by the
OpenXR loader. See [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md) for the full rationale.

## Build the layer

Requires CMake 3.22+, an MSVC C++17 toolchain, and the Windows SDK. The OpenXR
SDK headers are fetched automatically.

```powershell
cmake -S openxr-layer -B openxr-layer/build -A x64
cmake --build openxr-layer/build --config Release
```

For a packaged release, stage the artifacts so the installer bundles them:

```powershell
copy openxr-layer\build\Release\pitwall-openxr-layer.dll  src-tauri\resources\openxr-layer\
copy openxr-layer\manifest\pitwall_openxr_layer.json      src-tauri\resources\openxr-layer\
```

## Install and enable

In PitWall: start the live monitor, then **Start in-headset HUD** with VR mode
set to **Native** (Settings → VR mode). If the layer is not yet registered, the
panel shows an **Install VR layer** button, which registers the manifest under
`HKCU\Software\Khronos\OpenXR\1\ApiLayers\Implicit`
(see [`layer_install.rs`](../src-tauri/src/vr/layer_install.rs)). Restart iRacing
after installing so the loader picks up the layer.

Set `PITWALL_VR_DISABLE=1` to bypass the layer without unregistering it.

## Quest 3 + Meta Link setup

1. Connect the Quest 3 via Meta Link (or Air Link) and set iRacing to **OpenXR**.
2. In PitWall: Settings → **VR mode: Native**, install the VR layer, start the
   in-headset HUD, then launch iRacing and get on track.
3. Under Settings → **Overlay widgets**, enable the widgets you want and tune
   each one's **VR height / scale / opacity**. Widgets are head-locked (no
   recenter step, unlike the web fallback).
4. Choose **Field pace (coach)** (session best, optimal, or both) for the FLD/OPT
   readout on the coach widget.

## Migrating off RaceLab

PitWall VR is built to **replace** RaceLab VR, not sit beside it:

1. Disable RaceLab's VR overlay before enabling PitWall native — only one API
   layer compositor should be driving the same overlay surface while you test.
2. Run PitWall native and enable the widgets you want under
   Settings → Overlay widgets.
3. With coach, standings, relative, and radar all available, RaceLab can be
   uninstalled.

## Overlay widgets

PitWall ships one shared widget catalog. The same enable flags and field-pace
preference drive both the desktop pop-out and the in-headset HUD, so a widget
you turn on appears on both surfaces. Each widget keeps separate placement per
surface: pixel position/size on the desktop (drag and resize the panel), and a
VR height / scale / opacity you tune under Settings → Overlay widgets.

The protocol carries four head-locked overlay slots; the slot index equals the
widget kind, so each keeps a stable, correctly-sized swapchain:

| Slot | Widget | VR placement |
|------|--------|--------------|
| 0 | Coach HUD | Centered upper windshield (wide-short) |
| 1 | Standings strip | Lower-left (tall list) |
| 2 | Relative board | Lower-right (square) |
| 3 | Proximity radar | Low-center (square) |

Disabled widgets are published with `enabled = 0` and skipped by the compositor.
The web preview renders the same four layouts
(`/vr?layout=ironman|standings|relative|radar`).

## Desktop overlay

The desktop pop-out (Pop out overlay) is a transparent always-on-top window that
renders the same enabled widgets with the same React components used for the
in-app preview. Drag any widget by its top edge to move it and drag the
bottom-right corner to resize it; positions persist per widget. Drag an empty
area of the window to move the whole overlay. Enable or disable widgets and set
field pace from the main window under Settings → Overlay widgets.

## Troubleshooting

| Symptom | Check |
|---------|-------|
| HUD not visible in VR | iRacing in OpenXR mode? Layer installed and iRacing restarted? `PITWALL_VR_DISABLE` unset? |
| "VR layer not installed" persists | Run **Install VR layer** again; confirm the registry value under the Implicit ApiLayers key |
| Black screen / crash on launch | Disable other API layers (RaceLab VR, OpenXR Toolkit) and retry to isolate load-order conflicts |
| HUD shows but no data | Live monitor running and connected? Compositor status in the panel should read active |
| Spotter pack line never shows | Requires the `CarLeftRight` Int32 fix (shipped) and traffic alongside you |
