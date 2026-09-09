# Privacy

PitWall Desktop is designed to run **entirely on your machine**.

## What we do not do

- No analytics, crash phone-home, or third-party telemetry SDKs
- No accounts, cloud sync, or remote coaching servers
- No uploading of IBT files or live session data by the app itself

## What stays local

| Data | Location |
|------|----------|
| Imported sessions / laps | SQLite under `%LOCALAPPDATA%\pitwall-desktop\` |
| Settings | JSON beside the database |
| Coach WAV clips | Bundled app resources (and any you generate locally) |
| OpenXR layer install | Per-user registry + staged DLL under AppData |
| Logs | Process stdout/stderr when you enable `RUST_LOG` |

## What may appear in logs

When logging is enabled (`RUST_LOG=…`), messages may include:

- File paths under your iRacing telemetry folder
- Track/car names and session identifiers from the sim
- Error strings from import, live connect, or VR layer install

Do not share raw logs publicly if they contain paths or driver names you want private.

## Imports

IBT import is limited to paths you pick via the file dialog or files under the
configured telemetry directory (see import path policy in FOUNDATION / SETUP).
