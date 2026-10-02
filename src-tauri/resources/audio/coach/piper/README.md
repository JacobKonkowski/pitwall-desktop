# Piper coach voice

The audio coach speaks live numbers (lap times, gaps, deltas) with a bundled
[Piper](https://github.com/rhasspy/piper) neural voice, and the fixed callouts in
`../default/` are baked from the same voice.

The model files are not committed. Populate this folder with:

```powershell
.\scripts\fetch-piper-voice.ps1
```

which downloads `en_US-norman-medium` (sherpa-onnx conversion) and leaves:

- `en_US-norman-medium.onnx`, `tokens.txt`: the voice
- `espeak-ng-data/`: phonemizer tables
- `MODEL_CARD`: dataset and license (public domain, LibriVox recordings)

Without these files the coach falls back to Windows (WinRT) speech for numbers.
