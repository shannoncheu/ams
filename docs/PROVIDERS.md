# Music sources and experimental Windows login

The existing MusicProvider interface and local adapter remain unchanged. The
experimental YouTube session/catalog service is isolated from the audio engine;
its metadata responses never create playable local queue entries.

The previous desktop OAuth proposal has been removed. No Client ID/Secret,
loopback callback, OAuth dependency or token state was implemented in this app,
so existing local-library authentication must not be removed.

## Independent WebView2 login helper

Only the Windows login child process uses Wry/Tao/WebView2; the main player uses
GPUI. It opens accounts.google.com. User passwords stay in Google's page. The
helper reads its own resulting youtube.com session cookies and saves them in
Windows Credential Manager, using bounded chunks for Credential Manager limits.
Credentials never travel through stdout, command arguments, logs, Git or artifact
files. Login has no password input in GPUI. Close cancels; logout clears vault
entries. Temporary WebView profiles are removed after window/context destruction,
with a bounded cleanup retry. A cleanup failure returns a failed login result.

Google can reject embedded browsers. That failure must not be bypassed by user
agent disguise or copied accounts/cookies. WebView2 Runtime is required separately;
local playback does not depend on it. Windows CI tests helper open/close/cancel,
not a real Google account. Successful credential storage is not proof that the
server accepted a catalog request.

Search/home/library/likes/history use unofficial authenticated browse requests. Requests
run off the GPUI thread, have timeouts, bounded displayed rows and generic errors
that omit response bodies/credentials. Expired/refused sessions require login
again. The current UI renders returned text metadata only: no cover grid,
playlist mutations, complete pagination, or online audio playback is implemented.
No real account has verified these endpoints. Do not claim private library access
or login success until an actual account test passes.

## ytm-rs review and licensing

Reviewed https://github.com/grapedjuice/ytm-rs : login.rs, vault.rs, webmusic.rs,
ytmapi.rs, backend.rs, Cargo.toml and LICENSE. Its GPL-3.0 code uses a
separate Wry/Tao login process, web session cookies, a credential vault and
unofficial APIs. Some playback paths call extraction tools; these are not adopted.

MIT code can be combined with GPL-3.0 subject to GPL obligations on the combined
work, retaining MIT notices. It cannot simply copy GPL modules and distribute the
combined application as MIT-only. This project does not copy, link or redistribute
ytm-rs source. Its helper is independently written using the public
Wry/Tao/keyring APIs. Those dependencies have their own MIT/Apache notices,
collected into distribution licenses. Architecture ideas and API usage do not
require copying the GPL implementation. No upstream assets are reused.

References:
- https://github.com/grapedjuice/ytm-rs
- https://docs.rs/wry/0.57.0/wry/
- https://docs.rs/tao/0.37.1/tao/
- https://docs.rs/keyring/3.6.3/keyring/
- https://developers.google.com/youtube/terms/developer-policies

There is no general official native YouTube Music audio API for this CPAL player.
No DRM bypass, stream extraction or subscription impersonation is implemented.

The GPUI page also offers an explicit system-browser link to the official YouTube Music player as the supported listening alternative. This does not start native app audio playback.
