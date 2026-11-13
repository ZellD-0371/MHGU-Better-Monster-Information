$ErrorActionPreference='Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    New-Item -ItemType Directory -Path target/host -Force | Out-Null
    rustc +nightly --edition 2024 --test tools/check_native.rs -Awarnings -o target/host/check_native.exe
    if ($LASTEXITCODE -ne 0) { throw 'Could not build native model tests' }
    & ./target/host/check_native.exe
    if ($LASTEXITCODE -ne 0) { throw 'Native model tests failed' }
} finally { Pop-Location }
