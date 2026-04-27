# Rename Vibe → Machine across the entire workspace
# Run from the faction/ root directory.

$files = Get-ChildItem -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch "\\target\\" }

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw

    # Long compounds first, then standalone Vibe
    $content = $content -replace "VibeScenarioHarness", "MachineScenarioHarness"
    $content = $content -replace "NoOpVibeObserver",    "NoOpMachineObserver"
    $content = $content -replace "VibeConfig",          "MachineConfig"
    $content = $content -replace "VibeInput",           "MachineInput"
    $content = $content -replace "VibeOutput",          "MachineOutput"
    $content = $content -replace "VibeSnapshot",        "MachineSnapshot"
    $content = $content -replace "VibeState",           "MachineState"
    $content = $content -replace "VibeTransition",      "MachineTransition"
    $content = $content -replace "VibeObserver",        "MachineObserver"
    $content = $content -replace "\bvibe_check\b",      "snapshot"
    $content = $content -replace "\bVibe\b",            "Machine"
    $content = $content -replace "\bpunch\b",           "step"
    $content = $content -replace "\bdeal\b",            "accept"

    Set-Content $f.FullName -Value $content -NoNewline
}

Write-Host "Done. All .rs files updated."
