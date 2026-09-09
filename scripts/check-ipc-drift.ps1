# IPC contract sync
#
# Prefer regenerating shared TypeScript types from Rust with specta/tauri-specta
# as that lands. Until then, CI runs this drift smoke check: every #[tauri::command]
# name in commands/mod.rs should appear in src/shared/api.ts.

$ErrorActionPreference = "Stop"
$cmdFile = Join-Path $PSScriptRoot "..\src-tauri\src\commands\mod.rs"
$apiFile = Join-Path $PSScriptRoot "..\src\shared\api.ts"

$commands = Select-String -Path $cmdFile -Pattern 'pub (async )?fn ([a-z0-9_]+)\(' -AllMatches |
  ForEach-Object { $_.Matches } | ForEach-Object { $_.Groups[2].Value } |
  Where-Object { $_ -notmatch '^(new)$' } | Sort-Object -Unique

$api = Get-Content $apiFile -Raw
$missing = @()
foreach ($c in $commands) {
  # snake_case command often invoked as camelCase helper — check both
  $camel = [regex]::Replace($c, '_(.)', { param($m) $m.Groups[1].Value.ToUpper() })
  if ($api -notmatch [regex]::Escape($c) -and $api -notmatch [regex]::Escape($camel)) {
    $missing += $c
  }
}

if ($missing.Count -gt 0) {
  Write-Error ("IPC drift: commands missing from api.ts: " + ($missing -join ", "))
  exit 1
}
Write-Host ("IPC drift check OK (" + $commands.Count + " commands scanned).")
