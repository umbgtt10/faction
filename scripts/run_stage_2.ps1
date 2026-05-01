# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the Apache License, Version 2.0
# http://www.apache.org/licenses/LICENSE-2.0
#
# Install crap4rust:
#   cargo install crap4rust

$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Crap4RustGate {
    param(
        [string]$Label,
        [string[]]$Packages,
        [string]$Features = "",
        [switch]$NoDefaultFeatures,
        [switch]$IncludeTestTargets,
        [double]$Threshold = 15,
        [switch]$UseProjectThreshold,
        [string[]]$ExcludePaths = @()
    )

    cargo install cargo-crap4rust
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed to install crap4rust" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    Write-Host "$Label..." -ForegroundColor Cyan

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path
    $args = @("--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }
    if ($Features -ne "") {
        $args += @("--features", $Features)
    }
    if ($NoDefaultFeatures) {
        $args += "--no-default-features"
    }
    if ($IncludeTestTargets) {
        $args += "--include-test-targets"
    }
    foreach ($excludePath in $ExcludePaths) {
        $args += @("--exclude-path", $excludePath)
    }
    $args += @("--warn-only", "--threshold", $Threshold.ToString())

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo crap4rust @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $summaryLine = $output | Select-String -Pattern "summary:\s+total_functions=.*crappy_functions=(\d+)"
    if (-not $summaryLine) {
        Write-Host "`nFailed: $Label (could not parse crap4rust summary)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $crappyCount = [int]$summaryLine.Matches[0].Groups[1].Value

    if ($UseProjectThreshold) {
        $verdictLine = $output | Select-String -Pattern "verdict=(clean|warn|crappy)"
        if (-not $verdictLine) {
            Write-Host "`nFailed: $Label (could not parse crap4rust verdict)" -ForegroundColor Red
            Pop-Location
            exit 1
        }
        $verdict = $verdictLine.Matches[0].Groups[1].Value
        if ($verdict -eq "crappy") {
            Write-Host "`nFailed: $Label (project verdict is crappy)" -ForegroundColor Red
            Pop-Location
            exit 1
        }
    } else {
        if ($crappyCount -gt 0) {
            Write-Host "`nFailed: $Label ($crappyCount crappy functions detected)" -ForegroundColor Red
            Pop-Location
            exit 1
        }
    }
}

function Invoke-FileRiskGate {
    param(
        [string]$Label,
        [string[]]$Packages,
        [double]$Threshold,
        [string[]]$AllowedFiles,
        [string]$Features = "",
        [switch]$NoDefaultFeatures,
        [switch]$IncludeTestTargets
    )

    Write-Host "$Label..." -ForegroundColor Cyan

    $toolManifest = (Resolve-Path "$PSScriptRoot\..\..\etheram-tools\file-risk\Cargo.toml").Path
    Write-Host "$toolManifest..." -ForegroundColor Cyan
    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path

    $args = @(
        "run",
        "--manifest-path",
        $toolManifest,
        "--",
        "--manifest-path",
        $manifestPath
    )
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }
    if ($Features -ne "") {
        $args += @("--features", $Features)
    }
    if ($NoDefaultFeatures) {
        $args += "--no-default-features"
    }
    if ($IncludeTestTargets) {
        $args += "--include-test-targets"
    }
    $args += @("--threshold", $Threshold.ToString(), "--top", "200")

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $visibleFiles = @(
        $output |
            Select-String -Pattern '^(node|validation|system-tests)\s+(\S+\.rs)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\.\d+$' |
            ForEach-Object { $_.Matches[0].Groups[2].Value }
    )

    $newOffenders = @($visibleFiles | Where-Object { $_ -notin $AllowedFiles } | Sort-Object -Unique)
    if ($newOffenders.Count -gt 0) {
        Write-Host "`nFailed: $Label (new offenders detected: $($newOffenders -join ', '))" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    if ($visibleFiles.Count -gt $AllowedFiles.Count) {
        Write-Host "`nFailed: $Label (offender count increased to $($visibleFiles.Count))" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

# ---------------------------------------------------------------------------
# CRAP gates
# ---------------------------------------------------------------------------

Invoke-Crap4RustGate "CRAP faction" @("faction", "faction-protocol")

# ---------------------------------------------------------------------------
# File-risk gates
# ---------------------------------------------------------------------------

Invoke-FileRiskGate "File risk faction" @("faction", "faction-protocol") 30 @()

# ---------------------------------------------------------------------------

Write-Host "`nStage 2 passed!" -ForegroundColor Green
Pop-Location
exit 0
