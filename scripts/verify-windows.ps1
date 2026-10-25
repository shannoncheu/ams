param([switch]$Launch)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$os = Get-CimInstance Win32_OperatingSystem
if ($os.OSArchitecture -notmatch '64') {
    throw 'This validation script targets 64-bit Windows.'
}
$output = Join-Path $root ('validation/windows-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $output | Out-Null
Start-Transcript -Path (Join-Path $output 'build.log')
try {
    $os | Select-Object Caption, Version, BuildNumber, OSArchitecture
    Get-CimInstance Win32_Processor | Select-Object Name
    Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion
    foreach ($tool in @('rustc', 'cargo', 'cl.exe', 'link.exe')) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            throw "Missing $tool. Run in VS 2022 Developer PowerShell with Rust on PATH. See docs/WINDOWS.md."
        }
    }
    if (-not $env:GPUI_FXC_PATH) {
        $fxc = Get-Command fxc.exe -ErrorAction SilentlyContinue
        if ($fxc) {
            $env:GPUI_FXC_PATH = $fxc.Source
        } else {
            $sdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
            $fxc = Get-ChildItem "$sdk/*/x64/fxc.exe" -ErrorAction SilentlyContinue |
                Sort-Object FullName -Descending | Select-Object -First 1
            if (-not $fxc) { throw 'Windows SDK fxc.exe missing. See docs/WINDOWS.md.' }
            $env:GPUI_FXC_PATH = $fxc.FullName
        }
    }
    if (-not (Test-Path $env:GPUI_FXC_PATH)) { throw 'GPUI_FXC_PATH does not exist.' }
    Write-Output "FXC: $env:GPUI_FXC_PATH"
    & rustc -Vv
    if ($LASTEXITCODE -ne 0) { throw 'rustc failed' }
    & cargo -V
    & rustup show
    if (Test-Path (Join-Path $root '.git')) {
        & git rev-parse HEAD
        & git diff --stat
    } else {
        Write-Output 'Source archive; see docs/PROGRESS.zh-CN.md for the upstream baseline.'
    }
    $stages = @(
        @('metadata', '--locked', '--format-version', '1', '--no-deps'),
        @('check', '--locked'),
        @('test', '--locked'),
        @('build', '--release', '--locked')
    )
    foreach ($stage in $stages) {
        Write-Output ('cargo ' + ($stage -join ' '))
        & cargo @stage
        if ($LASTEXITCODE -ne 0) { throw "cargo $($stage[0]) failed ($LASTEXITCODE)" }
    }
    $exe = Join-Path $root 'target/release/gpui-apple-music-demo.exe'
    Get-FileHash -Algorithm SHA256 $exe | Format-List
    Write-Output "Build checks passed. Logs: $output"
    Write-Output 'Playback, visuals, transparency and performance still require interactive testing.'
    if ($Launch) {
        & $exe
        if ($LASTEXITCODE -ne 0) { throw "Player exited with $LASTEXITCODE" }
    }
} finally {
    Stop-Transcript
}
