use std::{fs, io::Write, path::PathBuf};
const MAX_BYTES: usize = 256 * 1024;
fn path() -> Result<PathBuf, String> {
    Ok(super::config_dir()?.join("crash.log"))
}

pub fn scrub(input: &str) -> String {
    let mut out = input.to_owned();
    for (var, fallback) in [("USERPROFILE", "HOME"), ("USERNAME", "USER")] {
        if let Ok(value) = std::env::var(var).or_else(|_| std::env::var(fallback)) {
            if value.len() > 1 {
                out = out.replace(&value, "[user]");
            }
        }
    }
    for prefix in ["rtmp://", "rtmps://"] {
        let mut redacted = String::with_capacity(out.len());
        let mut rest = out.as_str();
        while let Some(index) = rest.find(prefix) {
            redacted.push_str(&rest[..index]);
            redacted.push_str(prefix);
            redacted.push_str("[redacted]");
            let after = &rest[index + prefix.len()..];
            if let Some(end) = after.find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'')) {
                rest = &after[end..];
            } else {
                rest = "";
            }
        }
        redacted.push_str(rest);
        out = redacted;
    }
    let mut clean = String::with_capacity(out.len());
    let mut token = String::new();
    for c in out.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '=') {
            token.push(c);
        } else {
            if token.len() >= 20 {
                clean.push_str("[redacted]");
            } else {
                clean.push_str(&token);
            }
            token.clear();
            clean.push(c);
        }
    }
    if token.len() >= 20 {
        clean.push_str("[redacted]");
    } else {
        clean.push_str(&token);
    }
    clean
}

pub fn append(message: &str) -> Result<(), String> {
    let path = path()?;
    let parent = path.parent().ok_or("Crash log path has no parent folder")?;
    fs::create_dir_all(parent).map_err(|e| format!("Could not create config folder: {e}"))?;
    let body = format!(
        "---- crash {} ----\napp: {}\nos: {}/{}\n{}\n\n",
        unix_ts(),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        scrub(message)
    );
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Could not open crash log: {e}"))?;
    file.write_all(body.as_bytes())
        .map_err(|e| format!("Could not write crash log: {e}"))?;
    let mut raw = fs::read(&path).map_err(|e| format!("Could not read crash log: {e}"))?;
    if raw.len() > MAX_BYTES {
        raw.drain(..raw.len() - MAX_BYTES);
        if let Some(index) = raw.windows(10).position(|w| w == b"---- crash") {
            raw.drain(..index);
        }
        fs::write(path, raw).map_err(|e| format!("Could not trim crash log: {e}"))?;
    }
    Ok(())
}
pub fn read() -> Result<String, String> {
    let path = path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(path)
        .map(|s| scrub(&s))
        .map_err(|e| format!("Could not read crash log: {e}"))
}
fn unix_ts() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn scrub_removes_ingest_keys_and_long_tokens() {
        let result =
            super::scrub("server rtmps://ingest.example/live/secret ABCDEFGHIJKLMNOPQRSTUV");
        assert!(result.contains("rtmps://[redacted]"));
        assert!(!result.contains("secret"));
        assert!(!result.contains("ABCDEFGHIJKLMNOPQRSTUV"));
    }
}
