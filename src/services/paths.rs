//! Portable mode is opt-in via a marker beside the executable or --portable.
//! Never change HOME or system settings to redirect application storage.
use std::{path::PathBuf, sync::OnceLock};

fn portable_root() -> Option<&'static PathBuf> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let exe = std::env::current_exe().ok()?;
        let directory = exe.parent()?.to_path_buf();
        (directory.join("portable.flag").is_file()
            || std::env::args_os().any(|arg| arg == "--portable"))
        .then_some(directory)
    })
    .as_ref()
}

pub fn config_dir() -> Option<PathBuf> {
    portable_root()
        .map(|root| root.join("data/config"))
        .or_else(|| dirs::config_dir().map(|root| root.join("romus-demo-local-player")))
}

pub fn state_dir() -> Option<PathBuf> {
    portable_root()
        .map(|root| root.join("data/state"))
        .or_else(|| {
            dirs::state_dir()
                .or_else(dirs::data_local_dir)
                .map(|root| root.join("romus"))
        })
}

pub fn cache_dir() -> Option<PathBuf> {
    portable_root()
        .map(|root| root.join("data/cache"))
        .or_else(|| dirs::cache_dir().map(|root| root.join("romus")))
}

pub fn artwork_dir() -> PathBuf {
    cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("artwork")
}

pub fn client_config() -> Option<PathBuf> {
    if portable_root().is_none() {
        // Preserve the previous client configuration if it exists.
        if let Some(home) = dirs::home_dir() {
            let legacy = home.join(".config/romus-demo-local-player/config.toml");
            if legacy.is_file() {
                return Some(legacy);
            }
        }
    }
    Some(config_dir()?.join("config.toml"))
}
