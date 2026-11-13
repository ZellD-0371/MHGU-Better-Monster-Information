$ErrorActionPreference='Stop'
$taskOldPath=$env:PATH
$taskOldLink=$env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER
$taskOldOffline=$env:CARGO_NET_OFFLINE
Push-Location (Join-Path $PSScriptRoot '..')
try {
    $env:PATH='D:\Utils\devkitPro\devkitARM\bin;'+$env:PATH
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER='D:\Utils\devkitPro\devkitARM\bin\arm-none-eabi-gcc.exe'
    $env:CARGO_NET_OFFLINE='true'
    $taskBuildOutput=(& cargo +nightly forge build --release 2>&1 | Out-String)
    Write-Host $taskBuildOutput
    if ($LASTEXITCODE -ne 0 -or $taskBuildOutput -notmatch 'Output NRO:') { throw 'Native build failed' }
    & (Join-Path $PSScriptRoot 'Check-Instructions.ps1')
} finally {
    Pop-Location
    $env:PATH=$taskOldPath
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER=$taskOldLink
    $env:CARGO_NET_OFFLINE=$taskOldOffline
}
