# Fix variable/function names after Vibe -> Machine rename
# Run from the faction/ root directory.

$files = Get-ChildItem -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch "\\target\\" }

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw

    # Fix variable bindings: let mut Machine = -> let mut machine =
    $content = $content -replace "(?<=let\s+mut\s+)Machine(?=\s+=)", "machine"

    # Fix variable bindings: let Machine = -> let machine =
    $content = $content -replace "(?<=let\s+)Machine(?=\s+=)", "machine"

    # Fix function parameter names: fn p1(vibe: -> fn p1(machine:
    $content = $content -replace "(?<=\()vibe(?=\s*:)", "machine"

    # Fix function names with "vibe" in them: test_vibe, vibe_in_phase1, etc.
    $content = $content -replace "\btest_vibe\b", "test_machine"
    $content = $content -replace "\bvibe_in_phase1\b", "machine_in_phase1"
    $content = $content -replace "\bvibe_in_phase2\b", "machine_in_phase2"

    Set-Content $f.FullName -Value $content -NoNewline
}

Write-Host "Done. Variable/function names fixed."
