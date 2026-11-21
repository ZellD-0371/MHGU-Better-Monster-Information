# Install both complete resource sets in the user-designated environments.
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskTargets=@(
 @{Variant='standard';Stage='dist/native/Better-Monster-Information';Destination='D:/Games/Bok/MHGU/mods/Better-Monster-Information'},
 @{Variant='low-memory';Stage='dist/native-low/Better-Monster-Information';Destination='C:/Users/ZellD/AppData/Roaming/Ryujinx/mods/contents/0100770008dd8000/Better-Monster-Information'}
)
foreach($task in $taskTargets){
 $taskSource=Join-Path $taskRoot $task.Stage
 $taskAudit=Get-Content -LiteralPath (Join-Path $taskRoot "dist/build/$($task.Variant)/build.json") -Raw | ConvertFrom-Json
 $taskBinary=Join-Path $taskSource 'romfs/nativeNX/plugins/better_monster_information.nro'
 if($taskAudit.variant -ne $task.Variant -or (Get-FileHash -LiteralPath $taskBinary).Hash -ne $taskAudit.sha256){throw "Stale staging binary: $($task.Variant)"}
 $taskChanged=0;$taskVerified=0
 foreach($taskFile in Get-ChildItem -LiteralPath $taskSource -File -Recurse){
  $taskRelative=$taskFile.FullName.Substring($taskSource.Length).TrimStart([char[]]'\/')
  $taskDestination=Join-Path $task.Destination $taskRelative
  $taskHash=(Get-FileHash -LiteralPath $taskFile.FullName).Hash
  $taskSame=(Test-Path -LiteralPath $taskDestination) -and ((Get-FileHash -LiteralPath $taskDestination).Hash -eq $taskHash)
  if(!$taskSame){
   New-Item -ItemType Directory -Path (Split-Path $taskDestination) -Force | Out-Null
   Copy-Item -LiteralPath $taskFile.FullName -Destination $taskDestination -Force
   if((Get-FileHash -LiteralPath $taskDestination).Hash -ne $taskHash){throw "Installed file mismatch: $taskDestination"}
   $taskChanged++
  }
  $taskVerified++
 }
 Write-Output "$($task.Variant): installed and verified $taskVerified files ($taskChanged updated) -> $($task.Destination)"
}
