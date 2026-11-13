param([ValidateSet('standard','low-memory')][string]$Variant='standard')
$ErrorActionPreference='Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    py -3.10 -X utf8 tools/prepare_languages.py --check
    if ($LASTEXITCODE -ne 0) { throw 'Language/font resources failed verification' }
    New-Item -ItemType Directory -Path target/host -Force | Out-Null
    $taskTestCfg=if($Variant -eq 'low-memory'){@('--cfg','bmi_low_memory')}else{@()}
    rustc +nightly --edition 2024 --test tools/check_native.rs -Awarnings @taskTestCfg -o target/host/check_native.exe
    if ($LASTEXITCODE -ne 0) { throw 'Could not build native model tests' }
    & ./target/host/check_native.exe
    if ($LASTEXITCODE -ne 0) { throw 'Native model tests failed' }
} finally { Pop-Location }
