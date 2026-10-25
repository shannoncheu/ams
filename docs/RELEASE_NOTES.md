Experimental Windows-only WebView2 Google login helper with a pure GPUI sidebar/page.
Session cookies are saved in Windows Credential Manager, never stdout/config/logs.
Logout clears saved sessions; cancelled/failed login reports an error. The login
profile is temporary and removed on close. Google may reject embedded sign-in;
no bypass, user-agent disguise, passwords or audio stream extraction is used.

Search/home/library/likes/history use unofficial endpoints and display basic text
metadata. No real Google account has verified login, session restoration or
private library access. Cover grids, playlist mutations, and online audio
remain unavailable. Linux does not run this Windows helper. WebView2 Runtime is
required for login, while local playback remains independent of it.

Also includes slate sidebar/upload cleanup, explicit volume popup, library return,
native restored-frame removal, opaque default backdrop and per-user Setup/Portable.
Windows Server 2022 CI is not a Windows 10 physical compatibility test. The Linux
Wayland mini/main remap issue remains open. Builds are unsigned previews.

Download MusicPlayer-Setup-x64.exe or the complete MusicPlayer-Portable-x64.zip.
All dependency notices and covered source ship in the bundles. Verify SHA256SUMS.

The GPUI page also offers an explicit system-browser link to the official YouTube Music player as the supported listening alternative. This does not start native app audio playback.
