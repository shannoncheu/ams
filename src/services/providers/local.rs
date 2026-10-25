use super::{
    MusicProvider, PlaybackTarget, PlaylistId, ProviderCapabilities, ProviderError, ProviderFuture,
    ProviderPlaylist, ProviderTrack, SearchRequest, TrackId, TrackPage,
};
use crate::services::{api::TrackResponse, local::LocalStore};

/// Adapts the existing catalog without changing its persisted IDs or schema.
/// Blocking tag and filesystem operations execute on Tokio's blocking pool.
#[derive(Clone)]
pub struct LocalMusicProvider {
    store: LocalStore,
}

impl LocalMusicProvider {
    pub fn new(store: LocalStore) -> Self {
        Self { store }
    }

    fn local_id(id: TrackId) -> Result<String, ProviderError> {
        if id.provider != "local" {
            return Err(ProviderError::WrongProvider);
        }
        Ok(id.value)
    }
}

impl MusicProvider for LocalMusicProvider {
    fn id(&self) -> &'static str {
        "local"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            library: true,
            playlist_read: true,
            playlist_write: true,
            lyrics: true,
            native_playback: true,
        }
    }

    fn search(&self, request: SearchRequest) -> ProviderFuture<'_, TrackPage> {
        let store = self.store.clone();
        Box::pin(async move {
            let offset = decode_cursor(request.cursor.as_deref())?;
            blocking(move || {
                let query = request.query.trim().to_lowercase();
                let tracks = store.tracks()?;
                let mut matches = tracks.into_iter().filter(|track| {
                    query.is_empty()
                        || [&track.title, &track.artist, &track.album]
                            .into_iter()
                            .flatten()
                            .any(|text| text.to_lowercase().contains(&query))
                });
                let limit = request.limit.clamp(1, 100) as usize;
                let tracks = matches
                    .by_ref()
                    .skip(offset)
                    .take(limit)
                    .map(convert)
                    .collect::<Vec<_>>();
                let next_cursor = matches
                    .next()
                    .and_then(|_| offset.checked_add(tracks.len()))
                    .map(|offset| format!("local:{offset}"));
                Ok(TrackPage {
                    tracks,
                    next_cursor,
                })
            })
            .await
        })
    }

    fn track(&self, id: TrackId) -> ProviderFuture<'_, ProviderTrack> {
        let store = self.store.clone();
        Box::pin(async move {
            let id = Self::local_id(id)?;
            blocking(move || Ok(convert(store.track(&id)?))).await
        })
    }

    fn resolve_playback(&self, id: TrackId) -> ProviderFuture<'_, PlaybackTarget> {
        let store = self.store.clone();
        Box::pin(async move {
            let id = Self::local_id(id)?;
            blocking(move || Ok(PlaybackTarget::LocalFile(store.track_path(&id)?))).await
        })
    }

    fn lyrics(&self, id: TrackId) -> ProviderFuture<'_, Option<String>> {
        let store = self.store.clone();
        Box::pin(async move {
            let id = Self::local_id(id)?;
            blocking(move || Ok(store.track_metadata(&id)?.lyrics)).await
        })
    }

    fn playlists(&self) -> ProviderFuture<'_, Vec<ProviderPlaylist>> {
        let store = self.store.clone();
        Box::pin(blocking(move || {
            Ok(store
                .playlists()?
                .into_iter()
                .map(convert_playlist)
                .collect())
        }))
    }

    fn create_playlist(
        &self,
        name: String,
        description: Option<String>,
    ) -> ProviderFuture<'_, ProviderPlaylist> {
        let store = self.store.clone();
        Box::pin(blocking(move || {
            Ok(convert_playlist(store.create_playlist(name, description)?))
        }))
    }
}

fn convert_playlist(playlist: crate::services::api::PlaylistResponse) -> ProviderPlaylist {
    ProviderPlaylist {
        id: PlaylistId {
            provider: "local".to_owned(),
            value: playlist.id,
        },
        name: playlist.name,
        description: playlist.description,
        track_count: u64::try_from(playlist.track_count).ok(),
    }
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, ProviderError> + Send + 'static,
) -> Result<T, ProviderError> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| ProviderError::Worker(error.to_string()))?
}

fn convert(track: TrackResponse) -> ProviderTrack {
    ProviderTrack {
        id: TrackId {
            provider: "local".to_owned(),
            value: track.id,
        },
        title: track.title,
        artist: track.artist,
        album: track.album,
        duration: track
            .duration_ms
            .and_then(|ms| u64::try_from(ms).ok())
            .map(std::time::Duration::from_millis),
        has_artwork: track.has_artwork,
    }
}

fn decode_cursor(cursor: Option<&str>) -> Result<usize, ProviderError> {
    match cursor {
        None => Ok(0),
        Some(cursor) => cursor
            .strip_prefix("local:")
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|value| value.parse().ok())
            .ok_or(ProviderError::InvalidCursor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_rejects_foreign_malformed_and_overflowing_values() {
        for cursor in [
            "youtube:20",
            "local:-1",
            "local:+1",
            "local:",
            "local:1.5",
            "local:99999999999999999999999999999999999999",
        ] {
            assert!(matches!(
                decode_cursor(Some(cursor)),
                Err(ProviderError::InvalidCursor)
            ));
        }
        assert_eq!(decode_cursor(None).unwrap(), 0);
        assert_eq!(decode_cursor(Some("local:75")).unwrap(), 75);
    }

    #[test]
    fn foreign_ids_cannot_reach_local_file_resolution() {
        let id = TrackId {
            provider: "youtube".to_owned(),
            value: "remote-media".to_owned(),
        };
        assert!(matches!(
            LocalMusicProvider::local_id(id),
            Err(ProviderError::WrongProvider)
        ));
    }
}
