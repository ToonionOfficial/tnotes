# PowerShell packaging script for TNotes Desktop on Windows
param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path "$ScriptDir\..\.."
$DistDir = "$RepoRoot\dist"

if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}

if (-not $Version) {
    $CargoPath = "$RepoRoot\Cargo.toml"
    if (-not (Test-Path $CargoPath)) {
        $CargoPath = "$RepoRoot\apps\desktop\Cargo.toml"
    }
    $CargoContent = Get-Content $CargoPath -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        $Version = "0.1.0"
    }
}

Write-Host "==> Packaging Windows distribution for TNotes v$Version..."

$Binary = "$RepoRoot\target\release\tnotes-desktop.exe"
if (-not (Test-Path $Binary)) {
    Write-Host "--> Building tnotes-desktop release binary..."
    cargo build --release -p tnotes-desktop
}

# 1. Package Portable ZIP
Write-Host "--> Creating Portable ZIP archive..."
$PortableDir = "$DistDir\tnotes-$Version-windows-x64"
if (Test-Path $PortableDir) {
    Remove-Item -Recurse -Force $PortableDir
}
New-Item -ItemType Directory -Path $PortableDir | Out-Null

Copy-Item $Binary -Destination "$PortableDir\tnotes-desktop.exe"
Copy-Item "$RepoRoot\apps\desktop\assets\app-icon.ico" -Destination "$PortableDir\app-icon.ico"

$ZipOutput = "$DistDir\TNotes-Windows-x64-Portable.zip"
if (Test-Path $ZipOutput) {
    Remove-Item -Force $ZipOutput
}
Compress-Archive -Path "$PortableDir\*" -DestinationPath $ZipOutput
Remove-Item -Recurse -Force $PortableDir

# 2. Build Inno Setup Installer
Write-Host "--> Compiling Inno Setup installer..."
$Iscc = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
if (-not $Iscc) {
    $PotentialPaths = @(
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe"
    )
    foreach ($Path in $PotentialPaths) {
        if (Test-Path $Path) {
            $Iscc = $Path
            break
        }
    }
}

if ($Iscc) {
    & $Iscc "/DAppVersion=$Version" "$ScriptDir\installer.iss"
} else {
    Write-Warning "ISCC.exe not found. Install Inno Setup 6 or run in CI with choco install innosetup."
}

Write-Host "==> Windows build complete! Artifacts in $DistDir"
Get-ChildItem "$DistDir\TNotes-Windows-*" | Format-Table Name, Length, LastWriteTime
