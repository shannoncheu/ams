use std::{fs, io::Write, process::Command};

use gpui_apple_music_demo::services::{
    local::LocalStore,
    providers::{LocalMusicProvider, MusicProvider, PlaybackTarget, ProviderError, SearchRequest},
};

/// LocalStore is process-global. Run the roundtrip in a child with its own
/// catalog path instead of mutating environment variables in parallel tests.
#[test]
fn isolated_local_catalog_roundtrip() {
    if std::env::var_os("ROMUS_PROVIDER_TEST_CHILD").is_some() {
        run_roundtrip();
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    for name in ["fixture-first.wav", "fixture-second.wav"] {
        let mut file = fs::File::create(directory.path().join(name)).unwrap();
        let bytes = 8_000_u32 * 2;
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(36 + bytes).to_le_bytes()).unwrap();
        file.write_all(b"WAVEfmt ").unwrap();
        file.write_all(&16_u32.to_le_bytes()).unwrap();
        file.write_all(&1_u16.to_le_bytes()).unwrap(); // PCM
        file.write_all(&1_u16.to_le_bytes()).unwrap(); // mono
        file.write_all(&8_000_u32.to_le_bytes()).unwrap();
        file.write_all(&16_000_u32.to_le_bytes()).unwrap();
        file.write_all(&2_u16.to_le_bytes()).unwrap();
        file.write_all(&16_u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&bytes.to_le_bytes()).unwrap();
        file.write_all(&vec![0; bytes as usize]).unwrap();
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "isolated_local_catalog_roundtrip", "--nocapture"])
        .env("ROMUS_PROVIDER_TEST_CHILD", "1")
        .env("ROMUS_LOCAL_CONFIG", directory.path().join("library.toml"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_roundtrip() {
    let path = std::path::PathBuf::from(std::env::var_os("ROMUS_LOCAL_CONFIG").unwrap());
    let directory = path.parent().unwrap();
    let store = LocalStore::shared().unwrap();
    store
        .add_paths(vec![
            directory.join("fixture-first.wav"),
            directory.join("fixture-second.wav"),
        ])
        .unwrap();
    let provider: Box<dyn MusicProvider> = Box::new(LocalMusicProvider::new(store.clone()));
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let first = provider
                .search(SearchRequest {
                    query: "fixture".into(),
                    cursor: None,
                    limit: 1,
                })
                .await
                .unwrap();
            assert_eq!(first.tracks.len(), 1);
            let first_id = first.tracks[0].id.clone();
            let second = provider
                .search(SearchRequest {
                    query: "fixture".into(),
                    cursor: first.next_cursor,
                    limit: 1,
                })
                .await
                .unwrap();
            assert_eq!(second.tracks.len(), 1);
            assert_ne!(first_id, second.tracks[0].id);
            assert!(second.next_cursor.is_none());
            let track = provider.track(first_id.clone()).await.unwrap();
            assert_eq!(track.duration, Some(std::time::Duration::from_secs(1)));
            let PlaybackTarget::LocalFile(audio) =
                provider.resolve_playback(first_id.clone()).await.unwrap();
            assert!(audio.is_file());
            assert_eq!(provider.lyrics(first_id.clone()).await.unwrap(), None);
            let lyric = "[00:00.00]A measured interval";
            store
                .set_lyrics(&first_id.value, Some(lyric.to_owned()))
                .unwrap();
            assert_eq!(
                provider.lyrics(first_id.clone()).await.unwrap().as_deref(),
                Some(lyric)
            );
            let playlist = provider
                .create_playlist("Listening check".into(), None)
                .await
                .unwrap();
            assert!(
                provider
                    .playlists()
                    .await
                    .unwrap()
                    .iter()
                    .any(|item| item.id == playlist.id)
            );
            let mut foreign_id = first_id;
            foreign_id.provider = "youtube".into();
            assert!(matches!(
                provider.resolve_playback(foreign_id).await,
                Err(ProviderError::WrongProvider)
            ));
        });
}
