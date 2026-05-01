# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the Apache License, Version 2.0
# http://www.apache.org/licenses/LICENSE-2.0

$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Step {
    param([string]$Label, [scriptblock]$Command)
    Write-Host "$Label..." -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: $Label (exit code $LASTEXITCODE)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

$env:RUSTFLAGS = "-D warnings"

# ---------------------------------------------------------------------------
# Format + Lint
# ---------------------------------------------------------------------------

Invoke-Step "Formatting" { cargo fmt }

Invoke-Step "Clippy" { cargo clippy --workspace -- -D warnings }

# ---------------------------------------------------------------------------
# no_std checks
# ---------------------------------------------------------------------------

Invoke-Step "no_std checks" {
    cargo check `
        -p faction `
        -p faction-validation `
        -p faction-protocol `
        --no-default-features `
        --lib
}

# ---------------------------------------------------------------------------
# Integration Tests
# ---------------------------------------------------------------------------

Invoke-Step "faction tests" {
    cargo test `
        -p faction `
        -p faction-validation `
        -p faction-protocol `
        -p faction-system-tests `
}

Write-Host "`nFaction core, validation, protocol and system tests passed!" -ForegroundColor Green
Pop-Location
exit 0
