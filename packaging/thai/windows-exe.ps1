<#
.SYNOPSIS
  Build the Thai edition for Windows x64 as one pdfcraft.exe. No signing, no installer.

.DESCRIPTION
  The exe is self-contained (static CRT, no Visual C++ redistributable) and keeps its settings in
  %APPDATA%\PdfCraft like any installed app. Two things are left out on purpose:
    - the OCR models. Without a models\ folder beside the exe the built-in Latin OCR is unavailable;
      everything else works.
    - an MSI. The WiX files name ArtCraft, and this edition may not carry that name.
  The licence texts that must accompany a copy go in a separate text file next to the exe.
  Output in dist\release: pdfcraft-<version>-thai-windows-x64.exe and ...-licences.txt
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

# Static CRT; the icon and version info must embed or the build fails.
[Environment]::SetEnvironmentVariable('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS', '-C target-feature=+crt-static')
$env:PDFCRAFT_REQUIRE_WINRES = '1'
Push-Location $Root
try {
  # The CLI is built only to smoke-test the build: the app is a GUI program and prints nothing.
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p pdfcraft -p pdfcraft-cli --target $Target }
  $Bin = Join-Path $Root "target\$Target\release"
  Invoke-Native 'pdfcraft-cli --version' { & (Join-Path $Bin 'pdfcraft-cli.exe') --version }

  $Exe = Join-Path $Dist "pdfcraft-$Version-thai-windows-x64.exe"
  Copy-Item -Force (Join-Path $Bin 'pdfcraft.exe') $Exe

  # Licence texts that go with a copy of the program, in one file.
  $Notes = Join-Path $Dist "pdfcraft-$Version-thai-windows-x64-licences.txt"
  Remove-Item -Force $Notes -ErrorAction SilentlyContinue
  foreach ($f in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'assets\app-icon\LICENSE-document.txt') {
    $p = Join-Path $Root $f
    if (Test-Path $p) {
      Add-Content -Path $Notes -Encoding utf8 -Value ("=" * 78)
      Add-Content -Path $Notes -Encoding utf8 -Value $f
      Add-Content -Path $Notes -Encoding utf8 -Value ("=" * 78)
      Add-Content -Path $Notes -Encoding utf8 -Value (Get-Content -Raw $p)
      Add-Content -Path $Notes -Encoding utf8 -Value ""
    }
  }
  Get-Item $Exe, $Notes | Format-Table Name, Length
} finally {
  Pop-Location
}
