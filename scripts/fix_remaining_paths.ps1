# Fix remaining path references after Vibe → Machine rename
# Run from the faction/ root directory.

$files = Get-ChildItem -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch "\\target\\" }

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw

    # Capital module path: faction::Machine:: → faction::machine::
    $content = $content -replace "faction::Machine::", "faction::machine::"

    # Inline faction::vibe_* paths (not just use statements)
    $content = $content -replace "faction::vibe_", "faction::machine_"

    # faction_validation paths
    $content = $content -replace "faction_validation::vibe_", "faction_validation::machine_"

    Set-Content $f.FullName -Value $content -NoNewline
}

Write-Host "Done. Remaining paths fixed."
