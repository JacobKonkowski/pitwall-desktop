# Download the bundled Piper coach voice into src-tauri\resources\audio\coach\piper\.
# Needed once per checkout (and in release CI) before `tauri build`, baking clips,
# or hearing the neural voice under `tauri dev`. The folder is gitignored.
#
# Examples:
#   .\scripts\fetch-piper-voice.ps1
#   .\scripts\fetch-piper-voice.ps1 -Force

param(
    # Only ship voices whose training data permits redistribution: Norman is trained
    # from scratch on public-domain LibriVox audio. Lessac-derived voices are research-only.
    [string]$Voice = "en_US-norman-medium",
    # Re-download even when the voice is already present.
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Dest = Join-Path $Root "src-tauri\resources\audio\coach\piper"
$Model = Join-Path $Dest "$Voice.onnx"

if ((Test-Path $Model) -and -not $Force) {
    Write-Host "Piper voice already present: $Model" -ForegroundColor Green
    return
}

$Archive = "vits-piper-$Voice"
$Url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/$Archive.tar.bz2"
$Work = Join-Path ([System.IO.Path]::GetTempPath()) "race-refinery-piper-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $Work | Out-Null

try {
    $Tarball = Join-Path $Work "$Archive.tar.bz2"
    Write-Host "Downloading $Url" -ForegroundColor Cyan
    $ProgressPreference = "SilentlyContinue"
    Invoke-WebRequest -Uri $Url -OutFile $Tarball -UseBasicParsing

    & tar -xjf $Tarball -C $Work
    if ($LASTEXITCODE -ne 0) { throw "tar failed to extract $Tarball" }
    $Src = Join-Path $Work $Archive

    Get-ChildItem $Dest -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne "README.md" } |
        Remove-Item -Recurse -Force
    New-Item -ItemType Directory -Path $Dest -Force | Out-Null

    Copy-Item (Join-Path $Src "$Voice.onnx") $Dest
    Copy-Item (Join-Path $Src "tokens.txt") $Dest
    Copy-Item (Join-Path $Src "espeak-ng-data") $Dest -Recurse
    foreach ($Extra in @("MODEL_CARD", "$Voice.onnx.json")) {
        $Path = Join-Path $Src $Extra
        if (Test-Path $Path) { Copy-Item $Path $Dest }
    }

    $SizeMb = [math]::Round(((Get-ChildItem $Dest -Recurse -File | Measure-Object Length -Sum).Sum / 1MB), 1)
    Write-Host "Piper voice ready in $Dest ($SizeMb MB)" -ForegroundColor Green
}
finally {
    Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
}
