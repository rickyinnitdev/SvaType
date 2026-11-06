$ErrorActionPreference='Stop'
$packageRoot=Join-Path $PSScriptRoot '../target/packages'
foreach($line in Get-Content (Join-Path $packageRoot 'SHA256SUMS.txt')){
  $parts=$line -split '  ',2
  $actual=(Get-FileHash -LiteralPath (Join-Path $packageRoot $parts[1]) -Algorithm SHA256).Hash.ToLowerInvariant()
  if($actual -ne $parts[0]){throw "Package hash mismatch: $($parts[1])"}
}
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip=[System.IO.Compression.ZipFile]::OpenRead((Join-Path $packageRoot 'SvaType-Portable-x64.zip'))
try{
 $entries=@($zip.Entries.FullName | ForEach-Object {$_.Replace('\','/')})
 foreach($required in @('SvaType.exe','portable.txt','README.txt','content/index.json','content/vocabulary/animals.json')){
  if(-not ($entries | Where-Object {$_ -eq "SvaType-Portable-x64/$required"})){throw "Missing portable resource: $required"}
 }
 if($entries | Where-Object {$_ -match '(?i)\.(db|db-wal|db-shm|svabak)$' -or $_ -match '/data/'}){throw 'Learner data leaked into portable package'}
 Write-Output "Package hashes and portable resources passed ($($entries.Count) entries, no learner database)."
}finally{$zip.Dispose()}
