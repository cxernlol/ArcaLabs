use crate::config::Platform;
use keyring::Entry;

const SERVICE: &str = "arcalabs";
fn entry(platform: Platform) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("stream-key/{}", platform.as_str()))
        .map_err(|e| format!("OS keychain unavailable: {e}"))
}
pub fn set_stream_key(platform: Platform, key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("Stream key cannot be empty".into());
    }
    entry(platform)?
        .set_password(key)
        .map_err(|e| format!("Could not save stream key: {e}"))
}
pub fn get_stream_key(platform: Platform) -> Result<Option<String>, String> {
    match entry(platform)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Could not read stream key: {e}")),
    }
}
pub fn clear_stream_key(platform: Platform) -> Result<(), String> {
    match entry(platform)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Could not clear stream key: {e}")),
    }
}
