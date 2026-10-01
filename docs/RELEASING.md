# Releases & updater

## Versioning

Single source of truth for the app version:

- Root [`package.json`](../package.json) `version`
- Workspace [`Cargo.toml`](../Cargo.toml) `[workspace.package] version`
- [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json) `version`
- [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml) package version

Keep these aligned on every release. Prefer SemVer (`0.1.0`).

## Tag → installer

1. Update [CHANGELOG.md](../CHANGELOG.md) (`## [x.y.z] - date`).
2. Bump versions in the four places above.
3. IP hygiene ([IP.md](IP.md)): `cargo deny check licenses bans sources`, `npm run check:licenses`, refresh [NOTICE](../NOTICE) via `npm run gen:notice`.
4. `git tag v0.1.0 && git push origin v0.1.0`
5. CI `release` job builds MSI/NSIS and uploads artifacts.

## Auto-updater

PitWall uses `@tauri-apps/plugin-updater` / `tauri-plugin-updater`.

1. Generate signing keys: `npm run tauri signer generate -w ~/.tauri/pitwall.key`
2. Set GitHub secrets / env for the private key at release time.
3. Publish update JSON next to release artifacts (see Tauri updater docs).
4. Frontend may call `check()` on startup (opt-in in settings when wired).

Until keys and a public update endpoint exist, updater checks should remain
disabled by default so contributors are not blocked.
