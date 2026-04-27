$files = Get-ChildItem -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch '\\target\\' }

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw

    # Method calls on the machine variable
    $content = $content -replace 'Machine\.snapshot\b', 'machine.snapshot'
    $content = $content -replace 'Machine\.apply\b',    'machine.apply'
    $content = $content -replace 'Machine\.config\b',    'machine.config'

    # Variable reference in helper function calls: p1(&Machine), p2(&Machine)
    $content = $content -replace 'p1\(&Machine\)', 'p1(&machine)'
    $content = $content -replace 'p2\(&Machine\)', 'p2(&machine)'

    Set-Content $f.FullName -Value $content -NoNewline
}

Write-Host 'Done. Variable usages fixed.'
