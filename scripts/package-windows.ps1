$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)
$dist = Join-Path (Get-Location) 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$bundle = Join-Path $dist 'GPUI-Music-Player-Windows-x64'
if (Test-Path $bundle) { Remove-Item -Recurse -Force $bundle }
New-Item -ItemType Directory $bundle | Out-Null
$exe = Join-Path $bundle 'GPUI-Music-Player.exe'
$sourceExe = 'target/x86_64-pc-windows-msvc/release/gpui-apple-music-demo.exe'
if (-not (Test-Path $sourceExe)) { $sourceExe = 'target/release/gpui-apple-music-demo.exe' }
Copy-Item $sourceExe $exe

# Always bundle imported VC runtimes, even if preinstalled in runner System32.
# Other Windows system DLLs and API sets are provided by Windows 10 itself.
$queue = [System.Collections.Generic.Queue[string]]::new()
$queue.Enqueue($exe)
$seen = @{}
$imports = @()
$runtimeDlls = @()
while ($queue.Count) {
    $binary = $queue.Dequeue()
    $lines = & dumpbin.exe /DEPENDENTS $binary
    if ($LASTEXITCODE -ne 0) { throw "Cannot inspect $binary" }
    foreach ($line in $lines) {
        if ($line -notmatch '^\s+([a-zA-Z0-9_.-]+\.dll)\s*$') { continue }
        $dll = $Matches[1].ToLowerInvariant()
        $imports += $dll
        if ($seen[$dll]) { continue }
        $seen[$dll] = $true
        if ($dll -match '^(vcruntime|msvcp|concrt|vcomp)\d+.*\.dll$') {
            $candidates = @()
            if ($env:VCToolsRedistDir) { $candidates += "$env:VCToolsRedistDir/x64/Microsoft.VC*.CRT/$dll" }
            if ($env:VSINSTALLDIR) { $candidates += "$env:VSINSTALLDIR/VC/Redist/MSVC/*/x64/Microsoft.VC*.CRT/$dll" }
            $source = $candidates | ForEach-Object { Get-ChildItem $_ -ErrorAction SilentlyContinue } |
                Sort-Object FullName -Descending | Select-Object -First 1
            if (-not $source) { throw "Required runtime missing: $dll" }
            Copy-Item $source.FullName (Join-Path $bundle $dll)
            $queue.Enqueue((Join-Path $bundle $dll))
            $runtimeDlls += $dll
        } elseif ($dll -notmatch '^(api-ms-|ext-ms-)' -and -not (Test-Path "$env:SystemRoot/System32/$dll")) {
            throw "Non-system dependency must be explicitly bundled: $dll"
        }
    }
}
Copy-Item LICENSE $bundle
Copy-Item README.md (Join-Path $bundle 'README.md')
Copy-Item docs/PORTABLE.md (Join-Path $bundle 'START-HERE.md')
Copy-Item dist/THIRD_PARTY_NOTICES.txt $bundle
New-Item -ItemType Directory (Join-Path $bundle 'licenses') | Out-Null
Copy-Item assets/fonts/OFL.txt (Join-Path $bundle 'licenses/NotoSansSC-OFL.txt')
Copy-Item assets/fonts/SOURCE.md (Join-Path $bundle 'licenses/NotoSansSC-SOURCE.md')
Copy-Item dist/THIRD_PARTY_SOURCES.zip (Join-Path $bundle 'licenses/THIRD_PARTY_SOURCES.zip')
# A conventional EXE bundle stores data in the normal per-user application dirs.
Compress-Archive -Path "$bundle/*" -DestinationPath "$dist/GPUI-Music-Player-Windows-x64.zip" -Force
# Fonts, SVG icons and compiled shaders are inside the EXE; data is created on first run.
Set-Content -Encoding ascii -Path (Join-Path $bundle 'portable.flag') -Value 'portable'
Compress-Archive -Path "$bundle/*" -DestinationPath "$dist/GPUI-Music-Player-Portable-x64.zip" -Force
Copy-Item $exe "$dist/GPUI-Music-Player.exe"
foreach ($dll in $runtimeDlls) { Copy-Item (Join-Path $bundle $dll) (Join-Path $dist $dll) }
@{
    architecture = 'x64'; runtimeLinkage = 'crt-static requested';
    importedDlls = @($imports | Sort-Object -Unique); bundledDlls = $runtimeDlls;
    fontsEmbedded = @('Noto Sans SC variable, OFL-1.1'); iconsEmbedded = $true;
    win10PhysicalTested = $false; buildOS = [Environment]::OSVersion.VersionString
} | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 "$dist/runtime-dependencies.json"
Get-ChildItem $dist -File | Where-Object { $_.Extension -in '.exe', '.dll', '.zip' } |
    ForEach-Object { $hash = Get-FileHash -Algorithm SHA256 $_.FullName; "$($hash.Hash.ToLowerInvariant())  $($_.Name)" } |
    Set-Content -Encoding ascii "$dist/SHA256SUMS.txt"
