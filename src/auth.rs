//! OAuth (device-code) for xAI Grok. No API keys on the happy path.
//! Tokens live in ~/.config/elfy/auth.json (mode 0600).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
const DEVICE_URL: &str = "https://auth.x.ai/oauth2/device/code";
const TOKEN_URL: &str = "https://auth.x.ai/oauth2/token";
const SCOPE: &str = "openid profile email offline_access api:access";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64, // unix seconds
    pub provider: String,
}

impl TokenSet {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now + 60 >= self.expires_at
    }
}

fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().context("no config dir")?;
    let dir = base.join("elfy");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn auth_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("auth.json"))
}

pub fn load_tokens() -> Option<TokenSet> {
    let path = auth_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_tokens(tokens: &TokenSet) -> Result<()> {
    let path = auth_path()?;
    let raw = serde_json::to_string_pretty(tokens)?;
    fs::write(&path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

pub fn clear_tokens() -> Result<()> {
    let path = auth_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn is_logged_in() -> bool {
    load_tokens().is_some()
}

/// Agents the user can connect via OAuth. Grok is live; others share the same slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Grok,
    Claude,
}

impl AgentKind {
    pub const ALL: [AgentKind; 2] = [AgentKind::Grok, AgentKind::Claude];

    pub fn label(self) -> &'static str {
        match self {
            AgentKind::Grok => "Grok",
            AgentKind::Claude => "Claude",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            AgentKind::Grok => "OAuth · SuperGrok",
            AgentKind::Claude => "OAuth · Pro/Max",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            AgentKind::Grok => "xai",
            AgentKind::Claude => "claude",
        }
    }

    pub fn live(self) -> bool {
        matches!(self, AgentKind::Grok)
    }
}

pub fn connected_agent() -> Option<String> {
    load_tokens().map(|t| {
        if t.provider == "claude" {
            "Claude".into()
        } else {
            "Grok".into()
        }
    })
}

/// Device-code OAuth. Returns human-readable status lines for the TUI.
/// Blocks while polling; caller should show verification_uri + user_code first.
pub fn login_device_start() -> Result<DevicePending> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(DEVICE_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", SCOPE),
        ])
        .send()
        .context("device code request")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        bail!("device code failed ({status}): {body}");
    }

    #[derive(Deserialize)]
    struct DeviceResp {
        device_code: String,
        user_code: String,
        verification_uri: String,
        #[serde(default)]
        verification_uri_complete: Option<String>,
        expires_in: u64,
        interval: Option<u64>,
    }

    let d: DeviceResp = resp.json().context("parse device response")?;
    Ok(DevicePending {
        device_code: d.device_code,
        user_code: d.user_code,
        verification_uri: d
            .verification_uri_complete
            .unwrap_or(d.verification_uri),
        expires_in: d.expires_in,
        interval: d.interval.unwrap_or(5),
    })
}

pub struct DevicePending {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

impl DevicePending {
    /// Poll until tokens arrive or timeout. Blocks.
    pub fn wait_for_tokens(&self) -> Result<TokenSet> {
        let client = reqwest::blocking::Client::new();
        let deadline = SystemTime::now() + Duration::from_secs(self.expires_in);
        let interval = Duration::from_secs(self.interval.max(3));

        loop {
            if SystemTime::now() > deadline {
                bail!("login timed out — run again");
            }
            thread::sleep(interval);

            let resp = client
                .post(TOKEN_URL)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", self.device_code.as_str()),
                    ("client_id", CLIENT_ID),
                ])
                .send()
                .context("token poll")?;

            let status = resp.status();
            let body = resp.text().unwrap_or_default();

            if status.is_success() {
                return parse_token_response(&body);
            }

            // pending / slow_down are expected
            if body.contains("authorization_pending") || body.contains("slow_down") {
                continue;
            }
            if body.contains("expired_token") || body.contains("access_denied") {
                bail!("login denied or expired: {body}");
            }
            // unknown — keep trying a bit unless hard error
            if status.as_u16() >= 500 {
                continue;
            }
            bail!("token error ({status}): {body}");
        }
    }
}

fn parse_token_response(body: &str) -> Result<TokenSet> {
    #[derive(Deserialize)]
    struct Tok {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
    }
    let t: Tok = serde_json::from_str(body).context("parse token json")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(TokenSet {
        access_token: t.access_token,
        refresh_token: t.refresh_token,
        expires_at: now + t.expires_in.unwrap_or(3600),
        provider: "xai".into(),
    })
}

pub fn refresh_if_needed(tokens: &TokenSet) -> Result<TokenSet> {
    if !tokens.is_expired() {
        return Ok(tokens.clone());
    }
    let Some(refresh) = tokens.refresh_token.as_ref() else {
        bail!("session expired — run Login (OAuth) again");
    };
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.as_str()),
            ("client_id", CLIENT_ID),
        ])
        .send()
        .context("refresh request")?;

    if !resp.status().is_success() {
        let body = resp.text().unwrap_or_default();
        bail!("refresh failed: {body}");
    }
    let body = resp.text()?;
    let mut next = parse_token_response(&body)?;
    if next.refresh_token.is_none() {
        next.refresh_token = tokens.refresh_token.clone();
    }
    save_tokens(&next)?;
    Ok(next)
}

/// Full login: start device flow, return (uri, code) for display; caller polls.
pub fn login() -> Result<DevicePending> {
    login_device_start()
}

pub fn logout() -> Result<()> {
    clear_tokens()
}

/// Open the system browser. Returns true if a process spawned.
pub fn open_browser(url: &str) -> bool {
    let mut attempts: Vec<std::process::Command> = Vec::new();
    if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        attempts.push(c);
    } else if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", url]);
        attempts.push(c);
    } else {
        for bin in ["xdg-open", "gio", "firefox", "chromium", "google-chrome"] {
            let mut c = std::process::Command::new(bin);
            if bin == "gio" {
                c.args(["open", url]);
            } else {
                c.arg(url);
            }
            attempts.push(c);
        }
    }
    for mut c in attempts {
        c.stdin(std::process::Stdio::null());
        c.stdout(std::process::Stdio::null());
        c.stderr(std::process::Stdio::null());
        if c.spawn().is_ok() {
            return true;
        }
    }
    false
}

/// Copy text to the system clipboard. Returns true if a clipboard tool ran.
pub fn copy_clipboard(text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let tools: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];
    for (bin, args) in tools {
        let mut c = Command::new(bin);
        c.args(*args);
        c.stdin(Stdio::piped());
        c.stdout(Stdio::null());
        c.stderr(Stdio::null());
        if let Ok(mut child) = c.spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return true;
        }
    }
    false
}
