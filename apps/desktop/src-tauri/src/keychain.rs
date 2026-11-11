use crate::config::Platform;
use keyring::Entry;

const SERVICE: &str = "arcalabs";

fn entry(platform: &Platform) -> Result<Entry, String> {
    let account = format!("stream-key/{}", platform.as_str());
    Entry::new(SERVICE, &account).map_err(|e| format!("keychain unavailable: {e}"))
}

pub fn set_stream_key(platform: &Platform, key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("stream key cannot be empty".into());
    }
    entry(platform)?
        .set_password(key)
        .map_err(|e| format!("failed to save stream key: {e}"))
}

pub fn has_stream_key(platform: &Platform) -> Result<bool, String> {
    match entry(platform)?.get_password() {
        Ok(value) => Ok(!value.is_empty()),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("failed to read stream key presence: {e}")),
    }
}

pub fn clear_stream_key(platform: &Platform) -> Result<(), String> {
    match entry(platform)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("failed to clear stream key: {e}")),
    }
}
