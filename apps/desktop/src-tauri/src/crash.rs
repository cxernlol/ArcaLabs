use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const MAX_BYTES: u64 = 256 * 1024;

fn crash_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("config dir unavailable: {e}"))?;
    Ok(dir.join("crash.log"))
}

/// Strip secrets and user-identifying path segments before any write.
pub fn scrub(input: &str) -> String {
    let mut out = input.to_string();

    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        if !home.is_empty() {
            out = out.replace(&home, "[home]");
        }
    }
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        if user.len() > 1 {
            out = out.replace(&user, "[user]");
        }
    }

    out = redact_prefix_urls(&out, "rtmp://");
    out = redact_prefix_urls(&out, "rtmps://");
    out = redact_long_tokens(&out);
    out
}

fn redact_prefix_urls(input: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(idx) = rest.find(prefix) {
        out.push_str(&rest[..idx]);
        out.push_str(prefix);
        out.push_str("[redacted]");
        let after = &rest[idx + prefix.len()..];
        if let Some(end) = after.find(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
            rest = &after[end..];
        } else {
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

fn redact_long_tokens(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() {
            let c = bytes[i];
            if c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'=') {
                i += 1;
            } else {
                break;
            }
        }
        if i - start >= 20 {
            out.push_str("[redacted]");
        } else {
            out.push_str(&input[start..i]);
        }
        if i < bytes.len() {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

pub fn append_crash_log(app: &AppHandle, message: &str) -> Result<(), String> {
    let path = crash_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create config dir: {e}"))?;
    }

    let version = env!("CARGO_PKG_VERSION");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let ts = unix_ts();
    let body = scrub(message);
    let block = format!(
        "---- crash {ts} ----\napp: {version}\nos: {os}/{arch}\n{body}\n\n"
    );

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("failed to open crash.log: {e}"))?;
    file.write_all(block.as_bytes())
        .map_err(|e| format!("failed to write crash.log: {e}"))?;
    drop(file);
    trim_file(&path)
}

pub fn read_crash_log(app: &AppHandle) -> Result<String, String> {
    let path = crash_path(app)?;
    if !path.exists() {
        return Ok(String::new());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("failed to read crash.log: {e}"))?;
    Ok(scrub(&raw))
}

fn trim_file(path: &PathBuf) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| format!("crash.log stat failed: {e}"))?;
    if meta.len() <= MAX_BYTES {
        return Ok(());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("crash.log read failed: {e}"))?;
    let keep = &raw[raw.len().saturating_sub(MAX_BYTES as usize)..];
    let cut = keep.find("---- crash").unwrap_or(0);
    fs::write(path, &keep[cut..]).map_err(|e| format!("crash.log trim failed: {e}"))
}

fn unix_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}
