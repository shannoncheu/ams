# Music sources

`services::providers::MusicProvider` is an object-safe asynchronous catalog and
playback-resolution interface. Track and playlist IDs include their provider
namespace. It reports capabilities and typed failures; metadata access alone
does not authorize playback.

`LocalMusicProvider` adapts the existing `LocalStore`: search/library reads,
track metadata, lyrics, local playback paths, playlist listing and creation.
Filesystem and metadata operations run on Tokio's blocking pool. The existing
playback UI still uses its original services; migrating those call sites is a
separate step after local behavior is verified. Existing files and IDs are
unchanged. Local search still scans the original catalog before pagination; this
adapter does not yet provide an indexed large-library implementation.

Provider cursors are opaque and source-specific. Local cursors are validated
before catalog work. YouTube page tokens must remain YouTube cursors rather than
being converted into local offsets. Unsupported operations return errors and do
not update playback state.

## Official YouTube integration boundary

The next source adapter will use the YouTube Data API for video/channel/playlist
search and available metadata. Authorized playlist reading/writing requires a
configured Google desktop OAuth client, the system browser, a loopback callback,
PKCE/state checks and appropriately scoped consent. Tokens must be protected
with Windows Credential Manager or DPAPI, separately from the current local
session placeholders. Data refresh/deletion, revocation and quota errors need
explicit handling.

YouTube playlists and video search are not a complete YouTube Music catalog or
personal music library API. Channel names must not be silently presented as
verified recording artists. Missing album, duration or lyric information must
remain unavailable until obtained from an authorized source.

There is no general official API supplying YouTube Music audio to this GPUI/CPAL
engine. This milestone contains **no YouTube playback implementation**, account
login or live API test. A future metadata-only adapter must return
`PlaybackUnavailable` for native playback; opening an official browser link is a
separate external action. No extractor, cookie authentication, internal API,
DRM workaround or WebView is used.

References:

- https://developers.google.com/identity/protocols/oauth2/native-app
- https://developers.google.com/youtube/v3/docs/search/list
- https://developers.google.com/youtube/v3/docs/playlists/list
- https://developers.google.com/youtube/terms/developer-policies
- https://developers.google.com/youtube/iframe_api_reference
