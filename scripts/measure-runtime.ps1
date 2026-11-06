param([Parameter(Mandatory=$true)][int]$RootProcessId,[int]$Seconds=10)
# Read-only sampling of the app and WebView2 descendants. Does not change power settings.
$snapshot=Get-CimInstance Win32_Process
$ids=[System.Collections.Generic.HashSet[int]]::new()
[void]$ids.Add($RootProcessId)
do {
  $changed=$false
  foreach($item in $snapshot){if($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)){$changed=$true}}
} while($changed)
function Read-Sample {
  $cpu=0.0;$memory=0L;$count=0
  foreach($processId in $ids){$process=Get-Process -Id $processId -ErrorAction SilentlyContinue;if($process){$cpu+=$process.TotalProcessorTime.TotalSeconds;$memory+=$process.WorkingSet64;$count++}}
  @{cpu=$cpu;memory=$memory;count=$count}
}
$before=Read-Sample
Start-Sleep -Seconds $Seconds
$after=Read-Sample
$cpuPercent=100*($after.cpu-$before.cpu)/$Seconds/[Environment]::ProcessorCount
$private=0L
foreach($counter in (Get-CimInstance Win32_PerfFormattedData_PerfProc_Process)){if($ids.Contains([int]$counter.IDProcess)){$private+=$counter.WorkingSetPrivate}}
@{processes=$after.count;private_working_set_bytes=$private;working_set_bytes=$after.memory;cpu_percent=[Math]::Round($cpuPercent,3);interval_seconds=$Seconds} | ConvertTo-Json -Compress
