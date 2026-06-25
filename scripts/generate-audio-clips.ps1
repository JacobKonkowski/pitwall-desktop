# Dev-only: batch-export PitWall coach WAV clips using Windows WinRT neural speech.
# Does NOT run inside the PitWall app - only on your machine when regenerating assets.
#
# Examples:
#   .\scripts\generate-audio-clips.ps1
#   .\scripts\generate-audio-clips.ps1 -Voice "Jenny"
#   .\scripts\generate-audio-clips.ps1 -ListVoices
#   .\scripts\generate-audio-clips.ps1 -Engine Placeholder

param(
    [ValidateSet("WinRT", "Placeholder")]
    [string]$Engine = "WinRT",
    [string]$Voice = "",
    [switch]$ListVoices
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Manifest = Join-Path $Root "src-tauri\Cargo.toml"

Push-Location $Root
try {
    $cargoArgs = @(
        "run",
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

    $engineFlag = if ($Engine -eq "Placeholder") { "placeholder" } else { "winrt" }
    $cargoArgs += "--engine", $engineFlag

    if ($Voice) {
        $cargoArgs += "--voice", $Voice
    }

    Write-Host "Exporting clips (engine=$engineFlag). Neural runs here only, not in PitWall at runtime." -ForegroundColor Cyan
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Write-Host ""
    Write-Host "Done. Clips: src-tauri\resources\audio\coach\default\" -ForegroundColor Green
    Write-Host "Commit the WAVs and manifest.json to ship this voice in builds."
}
finally {
    Pop-Location
}
