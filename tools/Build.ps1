param([ValidateSet('standard','low-memory')][string]$Variant='standard')
$ErrorActionPreference='Stop'
$taskOldVariant=$env:BMI_VARIANT
$taskOldPath=$env:PATH
$taskOldLink=$env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER
$taskOldOffline=$env:CARGO_NET_OFFLINE
Push-Location (Join-Path $PSScriptRoot '..')
try {
    py -3.10 -X utf8 tools/prepare_languages.py --check
    if ($LASTEXITCODE -ne 0) { throw 'Language/font resources need regeneration' }
    py -3.10 -X utf8 tools/prepare_numbers.py --check
    if ($LASTEXITCODE -ne 0) { throw 'Numeric resources need regeneration' }
    $env:BMI_VARIANT=$Variant
    $env:PATH='D:\Utils\devkitPro\devkitARM\bin;'+$env:PATH
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER='D:\Utils\devkitPro\devkitARM\bin\arm-none-eabi-gcc.exe'
    $env:CARGO_NET_OFFLINE='true'
    # Windows PowerShell 5.1 turns cargo's stderr progress into errors under 'Stop'; judge by exit code and output instead.
    $ErrorActionPreference='Continue'
    $taskBuildOutput=(& cargo +nightly forge build --release 2>&1 | Out-String)
    $taskCargoExit=$LASTEXITCODE
    $ErrorActionPreference='Stop'
    Write-Host $taskBuildOutput
    if ($taskCargoExit -ne 0 -or $taskBuildOutput -notmatch 'Output NRO:') { throw 'Native build failed' }
    & (Join-Path $PSScriptRoot 'Check-Instructions.ps1')
    $taskBuilt=Join-Path (Get-Location) "dist/build/$Variant"
    New-Item -ItemType Directory -Path $taskBuilt -Force | Out-Null
    Copy-Item -LiteralPath better_monster_information.nro -Destination $taskBuilt
    @{variant=$Variant;sha256=(Get-FileHash better_monster_information.nro -Algorithm SHA256).Hash} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskBuilt 'build.json') -Encoding utf8
} finally {
    Pop-Location
    $env:BMI_VARIANT=$taskOldVariant
    $env:PATH=$taskOldPath
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER=$taskOldLink
    $env:CARGO_NET_OFFLINE=$taskOldOffline
}
