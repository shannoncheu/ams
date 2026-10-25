//! Windows-only login helper. No passwords or session values cross process stdout.
#[derive(Default)]
pub struct YoutubeState {
    pub busy: bool,
    pub message: String,
    pub rows: Vec<String>,
}

pub fn perform(action: &str) -> Result<Vec<String>, &'static str> {
    #[cfg(target_os = "windows")]
    {
        windows::perform(action)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = action;
        Err("YouTube login requires Windows 10 and WebView2 Runtime.")
    }
}

#[cfg(target_os = "windows")]
pub fn login_helper() -> i32 {
    windows::login().unwrap_or(1)
}

#[cfg(target_os = "windows")]
mod windows {
    use serde_json::{Value, json};
    use sha1::{Digest, Sha1};
    use std::{
        process::{Command, Stdio},
        time::{SystemTime, UNIX_EPOCH},
    };
    const SERVICE: &str = "AMS.YouTubeMusic";
    fn entry(name: &str) -> Result<keyring::Entry, &'static str> {
        keyring::Entry::new(SERVICE, name).map_err(|_| "Credential Manager is unavailable.")
    }
    fn load() -> Result<String, &'static str> {
        let count: usize = entry("chunks")?
            .get_password()
            .map_err(|_| "Sign in to Google first.")?
            .parse()
            .map_err(|_| "Saved session is invalid. Sign out and retry.")?;
        if count == 0 || count > 32 {
            return Err("Saved session is invalid.");
        }
        let mut cookie = String::new();
        for i in 0..count {
            cookie.push_str(
                &entry(&format!("session-{i}"))?
                    .get_password()
                    .map_err(|_| "Saved session is incomplete. Sign in again.")?,
            );
        }
        Ok(cookie)
    }
    fn forget() -> Result<(), &'static str> {
        for i in 0..32 {
            match entry(&format!("session-{i}"))?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(_) => return Err("Could not clear the session from Credential Manager."),
            }
        }
        match entry("chunks")?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Could not clear the saved session."),
        }
    }
    fn save(cookie: &str) -> Result<(), &'static str> {
        // Cookie values are ASCII. Split below Credential Manager's blob size limit.
        if !cookie.is_ascii() || cookie.len() > 32 * 1000 {
            return Err("Session size is unsupported.");
        }
        forget()?;
        for (i, part) in cookie.as_bytes().chunks(1000).enumerate() {
            if entry(&format!("session-{i}"))?
                .set_password(std::str::from_utf8(part).unwrap())
                .is_err()
            {
                let _ = forget();
                return Err("Could not securely save the session.");
            }
        }
        entry("chunks")?
            .set_password(&cookie.len().div_ceil(1000).to_string())
            .map_err(|_| "Could not commit the saved session.")
    }
    pub fn perform(action: &str) -> Result<Vec<String>, &'static str> {
        if action == "logout" {
            forget()?;
            return Ok(vec![]);
        }
        if action == "login" {
            let exe = std::env::current_exe().map_err(|_| "Cannot locate the login helper.")?;
            let status = Command::new(exe)
                .arg("--google-login-helper")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|_| "Could not start the login window.")?;
            if status.code() == Some(2) {
                return Err("Login cancelled. You can retry.");
            }
            if !status.success() {
                return Err(
                    "Login failed. Check WebView2 Runtime; Google may reject embedded sign-in. No bypass is attempted.",
                );
            }
        }
        let cookie = load()?;
        if action == "restore" {
            return browse(&cookie, "library");
        }
        browse(&cookie, action)
    }
    fn secret_header(value: &str) -> Result<reqwest::header::HeaderValue, &'static str> {
        let mut header = reqwest::header::HeaderValue::from_str(value)
            .map_err(|_| "Saved session is invalid.")?;
        header.set_sensitive(true);
        Ok(header)
    }
    fn browse(cookie: &str, page: &str) -> Result<Vec<String>, &'static str> {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .map_err(|_| "Network client unavailable.")?;
        let html = http
            .get("https://music.youtube.com/")
            .header("Cookie", secret_header(cookie)?)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|_| "YouTube is unavailable. Check your connection and retry.")?;
        let marker = "\"INNERTUBE_CLIENT_VERSION\":\"";
        let version = html
            .split_once(marker)
            .and_then(|(_, tail)| tail.split('"').next())
            .filter(|v| !v.is_empty())
            .ok_or("YouTube changed its response. This integration needs an update.")?;
        let sid = cookie
            .split(';')
            .find_map(|part| part.trim().strip_prefix("SAPISID="))
            .ok_or("Session expired. Sign out and sign in again.")?;
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "System clock is invalid.")?
            .as_secs();
        let hash = format!(
            "{:x}",
            Sha1::digest(format!("{time} {sid} https://music.youtube.com").as_bytes())
        );
        let id = match page {
            "home" => "FEmusic_home",
            "likes" => "FEmusic_liked_videos",
            "history" => "FEmusic_history",
            _ => "FEmusic_library_landing",
        };
        let mut payload =
            json!({"context":{"client":{"clientName":"WEB_REMIX","clientVersion":version}}});
        let endpoint = if let Some(query) = page.strip_prefix("search:") {
            if query.trim().is_empty() || query.len() > 1024 {
                return Err("Enter a search query of up to 1024 bytes.");
            }
            payload["query"] = json!(query.trim());
            "search"
        } else {
            payload["browseId"] = json!(id);
            "browse"
        };
        let response = http
            .post(format!("https://music.youtube.com/youtubei/v1/{endpoint}"))
            .header("Cookie", secret_header(cookie)?)
            .header("Origin", "https://music.youtube.com")
            .header(
                "Authorization",
                secret_header(&format!("SAPISIDHASH {time}_{hash}"))?,
            )
            .json(&payload)
            .send()
            .map_err(|_| "YouTube request failed. Retry later.")?;
        if matches!(response.status().as_u16(), 401 | 403) {
            return Err("Session expired or access was refused. Sign in again.");
        }
        let data: Value = response
            .error_for_status()
            .and_then(|r| r.json())
            .map_err(|_| "YouTube service is unavailable or returned an unsupported response.")?;
        if data.get("error").is_some() {
            return Err("YouTube refused this request.");
        }
        fn texts(value: &Value, out: &mut Vec<String>) {
            if out.len() >= 150 {
                return;
            }
            match value {
                Value::Object(map) => {
                    if let Some(text) = map.get("text").and_then(Value::as_str) {
                        if !text.trim().is_empty() && !out.iter().any(|s| s == text) {
                            out.push(text.to_owned());
                        }
                    }
                    for child in map.values() {
                        texts(child, out);
                    }
                }
                Value::Array(array) => {
                    for child in array {
                        texts(child, out);
                    }
                }
                _ => {}
            }
        }
        let mut rows = Vec::new();
        texts(&data, &mut rows);
        if rows.is_empty() {
            return Err("No supported content returned. Login validity is not confirmed.");
        }
        Ok(rows)
    }
    pub fn login() -> Result<i32, Box<dyn std::error::Error>> {
        use tao::{
            event::{Event, WindowEvent},
            event_loop::{ControlFlow, EventLoopBuilder},
            platform::run_return::EventLoopExtRunReturn,
            window::WindowBuilder,
        };
        let profile = std::env::temp_dir().join(format!("ams-login-{}", uuid::Uuid::new_v4()));
        let result = (|| -> Result<i32, Box<dyn std::error::Error>> {
            let mut events = EventLoopBuilder::<()>::with_user_event().build();
            let window = WindowBuilder::new()
                .with_title("Google sign-in — AMS")
                .build(&events)?;
            let notify = events.create_proxy();
            let mut context = wry::WebContext::new(Some(profile.clone()));
            let view = wry::WebViewBuilder::new_with_web_context(&mut context)
                .with_url("https://accounts.google.com/ServiceLogin?continue=https%3A%2F%2Fmusic.youtube.com%2F")
                .with_on_page_load_handler(move |stage, url| {
                    if matches!(stage, wry::PageLoadEvent::Finished) && url.starts_with("https://music.youtube.com/") { let _ = notify.send_event(()); }
                }).build(&window)?;
            let mut code = 2;
            let smoke = std::env::args_os().any(|arg| arg == "--google-login-helper-smoke");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
            events.run_return(|event, _, control| {
                *control = if smoke {
                    ControlFlow::WaitUntil(deadline)
                } else {
                    ControlFlow::Wait
                };
                if smoke && std::time::Instant::now() >= deadline {
                    *control = ControlFlow::Exit;
                    return;
                }
                match event {
                    Event::UserEvent(()) => {
                        if let Ok(cookies) = view.cookies_for_url("https://music.youtube.com/") {
                            if cookies.iter().any(|c| c.name() == "SAPISID") {
                                let value = cookies
                                    .iter()
                                    .map(|c| format!("{}={}", c.name(), c.value()))
                                    .collect::<Vec<_>>()
                                    .join("; ");
                                code = if save(&value).is_ok() { 0 } else { 1 };
                                *control = ControlFlow::Exit;
                            }
                        }
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control = ControlFlow::Exit,
                    _ => {}
                }
            });
            drop(view);
            drop(context);
            drop(window);
            drop(events);
            Ok(code)
        })();
        // WebView profile must never become part of portable data or build artifacts.
        for _ in 0..30 {
            if !profile.exists() || std::fs::remove_dir_all(&profile).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        if profile.exists() {
            return Err("Could not clear temporary login profile".into());
        }
        result
    }
}
