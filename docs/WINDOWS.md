# Windows 10 build and interactive validation

Use Windows 10 x64. Record the exact OS build; the current Fusion Acrylic path
requires build 17763 or later. Windows 10 22H2 is the preferred initial test host.
Linux checks do not validate Windows behavior.

## Development dependencies

Install Git, Rust through rustup, and Visual Studio 2022 Build Tools. In the
Visual Studio Installer, select **Desktop development with C++**, the MSVC v143
x64/x86 tools, and **Windows 10 SDK** (including `fxc.exe`). Local playback needs no web runtime. The experimental Google login window needs
Microsoft Edge WebView2 Runtime. Avoid selecting unrelated workloads.

If using winget, Git and rustup can be installed with:

```powershell
winget install --id Git.Git --exact
winget install --id Rustlang.Rustup --exact
```

Open a new **Developer PowerShell for VS 2022**, then select the native toolchain:

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc --profile minimal
rustup default stable-x86_64-pc-windows-msvc
rustc -Vv
cargo -V
where.exe cl
where.exe link
```

The rustc host must be `x86_64-pc-windows-msvc`. The verify script finds `fxc.exe`
inside the Windows SDK automatically, or accepts a process-local `GPUI_FXC_PATH`.
It does not install software or change system configuration.

## Verify original upstream before the modified application

In a separate directory, check out the unmodified baseline:

```powershell
git clone https://github.com/cradiy/gpui-apple-music-demo.git player-baseline
cd player-baseline
git checkout fe2564a40e10db4ba48a5f3bc00eaa16366aa261
cargo check --locked
cargo test --locked
cargo build --release --locked
cargo run --release --locked
```

If Release reports a missing shader compiler, set the path to your installed SDK
in this terminal, then repeat the failed command:

```powershell
$env:GPUI_FXC_PATH = (Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/*/x64/fxc.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1).FullName
```

For this modified checkout, from its root run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ./scripts/verify-windows.ps1 -Launch
```

ExecutionPolicy Bypass applies only to that process. The script records OS, GPU,
compiler, command output and binary SHA256 under `validation/`, stopping on
failed Cargo commands. A successful build is not an interactive test pass.

## Interactive checks

- Open the library, import legally available local MP3, FLAC and WAV files, and
  verify audible playback, pause/resume, previous/next, shuffle, repeat and volume.
- Seek using both pointer and keyboard; verify end-of-track behavior and position
  after restart. Test M4A separately with a codec supported by the audio backend.
- Import line and word timed LRC; verify timing, seeking by lyric, scrolling,
  transitions and the empty-lyrics state.
- Compare the player against the visual reference at a recorded window size/DPI;
  verify the circular cover, slate panel, controls and wave track.
- Enter/exit the mini player; verify shared queue/position and no duplicated audio.
- Verify dragging, resizing, maximize/restore, minimize, Snap, multi-monitor DPI,
  transparent areas and Acrylic on the real Windows host. Record any fallback.
- Measure idle, playback, lyrics, mini player and hidden-main-window CPU, private
  bytes, working set, GPU engine and GPU memory. Do not substitute Linux figures.

Return the build log, exact Windows build, any crash log, and pass/fail results
for these interactions. No account password or Google token is needed for local
player validation.
