//! Provider contracts separate catalog access from audio transport. A provider
//! must resolve an authorized playback target before the audio engine is opened.
//! YouTube Data API metadata does not imply native audio playback permission.

mod local;
pub use local::LocalMusicProvider;

use std::{future::Future, path::PathBuf, pin::Pin, time::Duration};

use super::api::ApiError;

pub type ProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackId {
    pub provider: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistId {
    pub provider: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ProviderPlaylist {
    pub id: PlaylistId,
    pub name: String,
    pub description: Option<String>,
    pub track_count: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
pub struct ProviderCapabilities {
    pub search: bool,
    pub library: bool,
    pub playlist_read: bool,
    pub playlist_write: bool,
    pub lyrics: bool,
    pub native_playback: bool,
}

#[derive(Clone, Debug)]
pub struct ProviderTrack {
    pub id: TrackId,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<Duration>,
    pub has_artwork: bool,
}

#[derive(Clone, Debug)]
pub struct SearchRequest {
    pub query: String,
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Clone, Debug)]
pub struct TrackPage {
    pub tracks: Vec<ProviderTrack>,
    pub next_cursor: Option<String>,
}

/// Metadata-only sources must return PlaybackUnavailable. Opening an official
/// website is an explicit external action, never an engine playback target.
#[derive(Clone, Debug)]
pub enum PlaybackTarget {
    LocalFile(PathBuf),
}

#[derive(Debug)]
pub enum ProviderError {
    Unsupported(&'static str),
    PlaybackUnavailable,
    WrongProvider,
    InvalidCursor,
    AuthenticationRequired,
    PermissionDenied,
    QuotaExceeded,
    Source(ApiError),
    Worker(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(operation) => write!(f, "Provider does not support {operation}"),
            Self::PlaybackUnavailable => {
                write!(f, "Native playback is unavailable for this source")
            }
            Self::WrongProvider => write!(f, "Track belongs to another provider"),
            Self::InvalidCursor => write!(f, "Invalid provider page cursor"),
            Self::AuthenticationRequired => write!(f, "Provider sign-in required"),
            Self::PermissionDenied => write!(f, "Provider permission denied"),
            Self::QuotaExceeded => write!(f, "Provider quota exceeded"),
            Self::Source(error) => error.fmt(f),
            Self::Worker(error) => write!(f, "Provider worker failed: {error}"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<ApiError> for ProviderError {
    fn from(error: ApiError) -> Self {
        Self::Source(error)
    }
}

/// Object-safe async interface. No GPUI entities or window handles cross into
/// provider work; the existing PlaybackController remains the playback owner.
pub trait MusicProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> ProviderCapabilities;
    fn search(&self, request: SearchRequest) -> ProviderFuture<'_, TrackPage>;
    fn track(&self, id: TrackId) -> ProviderFuture<'_, ProviderTrack>;
    fn resolve_playback(&self, id: TrackId) -> ProviderFuture<'_, PlaybackTarget>;
    fn lyrics(&self, id: TrackId) -> ProviderFuture<'_, Option<String>>;

    fn playlists(&self) -> ProviderFuture<'_, Vec<ProviderPlaylist>> {
        Box::pin(async { Err(ProviderError::Unsupported("playlist listing")) })
    }

    fn create_playlist(
        &self,
        _name: String,
        _description: Option<String>,
    ) -> ProviderFuture<'_, ProviderPlaylist> {
        Box::pin(async { Err(ProviderError::Unsupported("playlist creation")) })
    }
}
