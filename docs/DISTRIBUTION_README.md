# AMS Music Player — Windows x64 preview

Run GPUI-Music-Player.exe. No SDK, development tools, font installation or
administrator rights are required. Keep all supplied DLLs beside the EXE.

The complete Portable ZIP includes portable.flag. Extract it into a writable
folder; application data is created in data/config, data/state and data/cache.
Logs are in data/config/logs. Removing portable.flag switches to normal per-user
storage. No music files are copied into the portable directory; re-import paths
that no longer exist after moving to another computer.

Open the library with the icon at the upper left or by clicking the empty cover.
Hover the volume icon for its slider. Click the mini cover to expand the player.
The top-right icons minimize, maximize/restore and close the native window.

Fonts, SVG control icons and compiled shaders are inside the EXE. Font copyright
and OFL terms are in licenses/NotoSansSC-OFL.txt. Other attribution is in
THIRD_PARTY_NOTICES.txt. Corresponding unmodified covered source is in
licenses/THIRD_PARTY_SOURCES.zip.

The app uses an optional Windows 10 Acrylic path with an opaque slate fallback.
To disable transparency, set [appearance] backdrop = "opaque" in config.toml and
restart. No Windows 11 Mica is required.

This unsigned preview was built on GitHub Actions Windows Server 2022. It has not
passed Windows 10 physical-machine compatibility, audio stability, long-duration
resource or high-DPI acceptance. Linux virtual-audio retests showed intermittent
I/O errors. Google login, online YouTube metadata and native YouTube Music audio
playback are not implemented.

Source, complete README, screenshots and actual CI results:
https://github.com/shannoncheu/ams

Windows 10 baseline: 1903 or newer (22H2 recommended), x64. The executable imports the system ICU library; earlier Windows 10 builds are not supported by this preview. Physical-machine validation remains pending.
