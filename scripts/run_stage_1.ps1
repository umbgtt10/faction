# Copyright (c) 2025-2026 Umberto Gotti
# SPDX-License-Identifier: MIT

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

# Runs every system integration test in its own process, bounded to $MaxParallel,
# via slotgate (https://crates.io/crates/slotgate). Each slot gets a disjoint port
# range; process isolation replaces the in-process --test-threads=1 serialization.
function Invoke-SystemTestsParallel {
    param([int]$MaxParallel = 6)

    Write-Host "faction system tests (parallel via slotgate)..." -ForegroundColor Cyan

    if (-not (Get-Command slotgate -ErrorAction SilentlyContinue)) {
        Write-Host "`nslotgate is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install slotgate" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $artifacts = cargo test -p faction-system-tests --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: building system tests (exit code $LASTEXITCODE)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $binary = $artifacts |
        ForEach-Object { $_ | ConvertFrom-Json -ErrorAction SilentlyContinue } |
        Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'all_tests' -and $_.executable } |
        Select-Object -Last 1 -ExpandProperty executable

    if (-not $binary) {
        Write-Host "`nFailed: could not locate the all_tests test binary" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $jobs = (& $binary --list |
        Where-Object { $_ -match ': test$' } |
        ForEach-Object { $_ -replace ': test$', '' }) -join ','

    slotgate `
        --program $binary `
        --program-args '{job},--exact' `
        --jobs $jobs `
        --max-parallel $MaxParallel `
        --log-dir logs/slotgate

    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: faction system tests (exit code $LASTEXITCODE)" -ForegroundColor Red
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
        -p faction-core-validation `
        -p faction-protocol `
        -p faction-protocol-validation `
        --no-default-features `
        --lib
}

# ---------------------------------------------------------------------------
# Integration Tests
# ---------------------------------------------------------------------------

Invoke-Step "faction tests" {
    cargo test `
        -p faction `
        -p faction-core-validation `
        -p faction-protocol `
        -p faction-protocol-validation `
}

Invoke-Step "faction system unit tests" {
    cargo test -p faction-system-tests --lib --bins
}

Invoke-SystemTestsParallel

Write-Host "`nFaction core, validation, protocol and system tests passed!" -ForegroundColor Green
Pop-Location
exit 0
