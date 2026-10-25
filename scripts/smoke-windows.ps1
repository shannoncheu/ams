$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)
$root = (Get-Location).Path
$extract = Join-Path $env:RUNNER_TEMP 'gpui-portable-test'
if (Test-Path $extract) { Remove-Item $extract -Recurse -Force }
Expand-Archive dist/GPUI-Music-Player-Portable-x64.zip $extract
$cwd = Join-Path $env:RUNNER_TEMP 'gpui-unrelated-cwd'
New-Item -ItemType Directory -Force $cwd | Out-Null
$reportDir = Join-Path $root 'validation/windows-smoke'
New-Item -ItemType Directory -Force $reportDir | Out-Null
$process = Start-Process (Join-Path $extract 'GPUI-Music-Player.exe') -ArgumentList '--smoke-test' -WorkingDirectory $cwd -PassThru
$peakWorkingSet = 0L
$windowSeen = $false
$deadline = (Get-Date).AddSeconds(120)
while (-not $process.HasExited -and (Get-Date) -lt $deadline) {
    $process.Refresh()
    $peakWorkingSet = [Math]::Max($peakWorkingSet, $process.WorkingSet64)
    if ($process.MainWindowHandle -ne [IntPtr]::Zero) { $windowSeen = $true }
    Start-Sleep -Milliseconds 200
}
if (-not $process.HasExited) { Stop-Process -Id $process.Id; throw 'Portable startup timeout' }
$process.WaitForExit()
$code = $process.ExitCode
Get-ChildItem "$extract/data" -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object Extension -eq '.log' | Copy-Item -Destination $reportDir
@{
    exitCode = $code; nativeWindowObserved = $windowSeen; peakWorkingSetBytes = $peakWorkingSet;
    portableConfigCreated = (Test-Path "$extract/data/config/config.toml");
    runner = 'windows-2022'; win10PhysicalTested = $false;
    scope = 'CI startup/storage smoke test, not audio, visual or Windows 10 acceptance';
    gpuMeasurement = $null
} | ConvertTo-Json | Set-Content -Encoding utf8 "$root/dist/windows-smoke.json"
if ($code -ne 0) { throw "Portable EXE exited with $code" }
if (-not $windowSeen) { throw 'No native player window was observed' }
if (-not (Test-Path "$extract/data/config/config.toml")) { throw 'Portable config not created beside EXE' }
