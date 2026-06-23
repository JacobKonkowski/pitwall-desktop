# Bundled OpenXR layer staging

`tauri build` bundles everything in this folder under
`<app resources>/resources/openxr-layer/`, where PitWall's `install_vr_layer`
command registers the manifest with the OpenXR loader.

Before packaging a release, build the layer and stage its artifacts here:

```powershell
cmake -S openxr-layer -B openxr-layer/build -A x64
cmake --build openxr-layer/build --config Release
copy openxr-layer\build\Release\pitwall-openxr-layer.dll  src-tauri\resources\openxr-layer\
copy openxr-layer\manifest\pitwall_openxr_layer.json      src-tauri\resources\openxr-layer\
```

This README is a placeholder so the bundle resource glob always matches at least
one file; the DLL and manifest are produced by the separate C++ build and are
not checked into the repository.
