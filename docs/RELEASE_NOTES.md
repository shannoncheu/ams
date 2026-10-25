Native Rust/GPUI music player preview with a slate/ice-blue palette, bundled Noto
Sans SC, always-visible transparent caption icons, readable lyric weights and a
portable data directory. Existing audio and lyric engines are retained.

Download the complete Portable ZIP for portable storage; the ordinary Windows ZIP
includes licenses and any required DLLs. Fonts/icons/shaders are embedded. Verify
SHA256SUMS.txt. Builds are unsigned and require Windows x64 with a usable graphics
driver. No administrator rights or SDK is required to run the packaged application.

Windows artifacts are compiled on GitHub Actions windows-2022 (Server 2022).
They are NOT a Windows 10 physical-machine compatibility certification. Real
Win10 playback, Acrylic/DPI/Snap/taskbar behavior, font appearance and CPU/GPU/RAM
remain to be measured. Linux virtual audio retests had intermittent ALSA/Pulse
I/O errors. YouTube login/online metadata/playback are not implemented.

Windows 10 baseline: 1903 or newer (22H2 recommended), x64. The executable imports the system ICU library; earlier Windows 10 builds are not supported by this preview. Physical-machine validation remains pending.
