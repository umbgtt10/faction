# Fix module path references across the workspace
# Run from the faction/ root directory.

$files = Get-ChildItem -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch "\\target\\" }

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw

    # Replace module path prefixes
    $content = $content -replace "crate::vibe_", "crate::machine_"
    $content = $content -replace "use super::vibe_", "use super::machine_"
    $content = $content -replace "use faction::vibe_", "use faction::machine_"
    $content = $content -replace "use faction::no_op_vibe_observer", "use faction::no_op_machine_observer"

    Set-Content $f.FullName -Value $content -NoNewline
}

Write-Host "Done. Module paths updated."
