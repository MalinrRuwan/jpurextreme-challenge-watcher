# hkwatch setup — Windows (PowerShell)
# Installs dependencies and builds/downloads the hkwatch binary.
$ErrorActionPreference = "Stop"

$RepoDir = Split-Path -Parent $PSScriptRoot
Set-Location $RepoDir

Write-Host "==> hkwatch setup (Windows)" -ForegroundColor Cyan

# ---- 1. Node.js ----
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "==> Installing Node.js via winget (lts)" -ForegroundColor Green
    winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
    # refresh PATH from machine env
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}
Write-Host "==> Node.js: $(node --version)"

# ---- 2. Rust ----
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "==> Installing Rust via winget (rustup)" -ForegroundColor Green
    winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    & "$env:USERPROFILE\.cargo\bin\rustup-init.exe" -y --default-toolchain stable
}
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Host "==> Cargo: $(cargo --version)"

# ---- 3. Playwright stealth fetcher ----
Write-Host "==> Installing Node deps for the cloak fetcher" -ForegroundColor Green
Set-Location "$RepoDir\cloak"
npm install --no-fund --no-audit
npx playwright install chromium
Set-Location $RepoDir

# ---- 4. opencode2 (solver) ----
if (-not (Get-Command opencode2 -ErrorAction SilentlyContinue)) {
    Write-Host "==> Installing opencode2 (@opencode-ai/cli)" -ForegroundColor Green
    npm install -g @opencode-ai/cli
}
Write-Host "==> opencode2: $(opencode2 --version 2>$null)"

# ---- 5. hkwatch binary ----
$BinDir = "$RepoDir\bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
if (-not (Test-Path "$BinDir\hkwatch.exe")) {
    Write-Host "==> Building hkwatch from source (cargo)" -ForegroundColor Green
    Set-Location "$RepoDir\watcher"
    cargo build --release
    Copy-Item "target\release\hkwatch.exe" "$BinDir\hkwatch.exe"
    Set-Location $RepoDir
}

# ---- 6. credentials ----
if (-not (Test-Path "$RepoDir\.hkwatch.env")) {
    Copy-Item "$RepoDir\.hkwatch.env.example" "$RepoDir\.hkwatch.env"
    Write-Host "==> Created .hkwatch.env — EDIT IT with your HackerRank credentials" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Setup complete ===" -ForegroundColor Cyan
Write-Host "Next steps:"
Write-Host "  1. notepad $RepoDir\.hkwatch.env   # add your HackerRank username/password"
Write-Host "  2. $BinDir\hkwatch.exe watch --headless --skip-current"
Write-Host "     (SIGUSR1 is not available on Windows; restart the watcher to toggle solving)"
