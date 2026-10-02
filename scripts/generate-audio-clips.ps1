# Dev-only: batch-export PitWall coach WAV clips with the bundled Piper neural voice
# (the same voice the app uses live for numbers), or WinRT / placeholder.
# Does NOT run inside the PitWall app - only on your machine when regenerating assets.
#
# Examples:
#   .\scripts\generate-audio-clips.ps1
#   .\scripts\generate-audio-clips.ps1 -Only tyre_hot,lap_invalid,radio_beep
#   .\scripts\generate-audio-clips.ps1 -Engine WinRT -Voice "Guy"
#   .\scripts\generate-audio-clips.ps1 -ListVoices
#   .\scripts\generate-audio-clips.ps1 -Engine Placeholder

param(
    [ValidateSet("Piper", "WinRT", "Placeholder")]
    [string]$Engine = "Piper",
    # WinRT voice substring; ignored by the other engines.
    [string]$Voice = "",
    [switch]$ListVoices,
    # Regenerate only these keys; other clips and manifest entries are kept.
    [string[]]$Only = @()
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Manifest = Join-Path $Root "src-tauri\Cargo.toml"

Push-Location $Root
try {
    $cargoArgs = @(
        "run",
        "--release",
        "--manifest-path", $Manifest,
        "--bin", "gen-audio-clips",
        "--"
    )

    if ($ListVoices) {
        $cargoArgs += "--list-voices"
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        return
    }

    if ($Engine -eq "Piper") {
        & (Join-Path $PSScriptRoot "fetch-piper-voice.ps1")
    }

    $engineFlag = $Engine.ToLowerInvariant()
    $cargoArgs += "--engine", $engineFlag

    if ($Voice) {
        $cargoArgs += "--voice", $Voice
    }
    if ($Only.Count -gt 0) {
        $cargoArgs += "--only", ($Only -join ",")
    }

    Write-Host "Exporting clips (engine=$engineFlag)." -ForegroundColor Cyan
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Write-Host ""
    Write-Host "Done. Clips: src-tauri\resources\audio\coach\default\" -ForegroundColor Green
    Write-Host "Commit the WAVs and manifest.json to ship this voice in builds."
}
finally {
    Pop-Location
}
