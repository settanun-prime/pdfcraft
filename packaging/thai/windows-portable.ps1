<#
.SYNOPSIS
  Build the Thai edition for Windows x64 and pack it as a portable zip. No signing, no MSI.

.DESCRIPTION
  Same layout as the portable zip of packaging/windows/package.ps1: pdfcraft.exe, pdfcraft-cli.exe,
  the licence files, the OCR models and portable.txt. It skips the MSI on purpose: the WiX files name
  ArtCraft, and this edition may not carry that name. Output: dist\release\pdfcraft-<version>-thai-windows-x64-portable.zip
#>
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Target = 'x86_64-pc-windows-msvc'

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in [workspace.package] of the root Cargo.toml.
$Version = $env:PDFCRAFT_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }

$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null
if (-not $env:PDFCRAFT_BUILD_SHA) { $env:PDFCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null) }
if (-not $env:PDFCRAFT_BUILD_DATE) { $env:PDFCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }
Write-Output "PdfCraft TH $Version for Windows x64"

# Static CRT, so no VC++ redistributable is needed; the icon and version info must embed or the build fails.
[Environment]::SetEnvironmentVariable('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS', '-C target-feature=+crt-static')
$env:PDFCRAFT_REQUIRE_WINRES = '1'
Push-Location $Root
try {
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p pdfcraft -p pdfcraft-cli --target $Target }

  $Bin = Join-Path $Root "target\$Target\release"
  $Pack = Join-Path $Root "target\thai-package\pdfcraft-$Version-thai-windows-x64-portable"
  Remove-Item -Recurse -Force $Pack -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $Pack | Out-Null
  Copy-Item (Join-Path $Bin 'pdfcraft.exe'), (Join-Path $Bin 'pdfcraft-cli.exe') $Pack
  foreach ($f in 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'README.md') {
    $p = Join-Path $Root $f
    if (Test-Path $p) { Copy-Item $p $Pack }
  }
  Copy-Item (Join-Path $Root 'assets\app-icon\LICENSE-document.txt') $Pack
  Copy-Item (Join-Path $Root 'packaging\windows\portable.txt') $Pack

  # OCR models: the app looks in models\ beside pdfcraft.exe; xtask verifies each file's SHA-256.
  $Models = Join-Path $Pack 'models'
  New-Item -ItemType Directory -Force -Path $Models | Out-Null
  Invoke-Native 'cargo xtask models' { cargo xtask models $Models }
  Remove-Item -Force (Join-Path $Models '*.part') -ErrorAction SilentlyContinue

  $Zip = Join-Path $Dist "pdfcraft-$Version-thai-windows-x64-portable.zip"
  Remove-Item -Force $Zip -ErrorAction SilentlyContinue
  Compress-Archive -Path $Pack -DestinationPath $Zip

  # The binary must run and say which edition it is.
  Invoke-Native 'pdfcraft-cli --version' { & (Join-Path $Pack 'pdfcraft-cli.exe') --version }
  Get-Item $Zip | Format-Table Name, Length
} finally {
  Pop-Location
}
