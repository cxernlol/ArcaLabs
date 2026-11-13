use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Youtube,
    Kick,
    #[default]
    Twitch,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Youtube => "youtube",
            Self::Kick => "kick",
            Self::Twitch => "twitch",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    pub automatic: bool,
    pub encoder: String,
    pub resolution: String,
    pub frame_rate: u32,
    pub bitrate_kbps: u32,
    pub keyframe_interval_sec: u32,
}
impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            automatic: true,
            encoder: "NVENC".into(),
            resolution: "1920x1080".into(),
            frame_rate: 60,
            bitrate_kbps: 6000,
            keyframe_interval_sec: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestUrls {
    pub youtube: String,
    pub kick: String,
    pub twitch: String,
}
impl Default for IngestUrls {
    fn default() -> Self {
        Self {
            youtube: "rtmps://a.rtmps.youtube.com/live2".into(),
            kick: String::new(),
            twitch: "rtmps://live.twitch.tv/app".into(),
        }
    }
}
impl IngestUrls {
    pub fn get(&self, platform: Platform) -> &str {
        match platform {
            Platform::Youtube => &self.youtube,
            Platform::Kick => &self.kick,
            Platform::Twitch => &self.twitch,
        }
    }
    pub fn get_mut(&mut self, platform: Platform) -> &mut String {
        match platform {
            Platform::Youtube => &mut self.youtube,
            Platform::Kick => &mut self.kick,
            Platform::Twitch => &mut self.twitch,
        }
    }
    pub fn set(&mut self, platform: Platform, url: String) {
        match platform {
            Platform::Youtube => self.youtube = url,
            Platform::Kick => self.kick = url,
            Platform::Twitch => self.twitch = url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDefaults {
    pub default_mic_id: String,
    pub default_desktop_id: String,
}
impl Default for AudioDefaults {
    fn default() -> Self {
        Self {
            default_mic_id: "default-mic".into(),
            default_desktop_id: "default-desktop".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub start_minimized: bool,
    pub confirm_before_stop: bool,
}
impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            confirm_before_stop: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub platform: Platform,
    pub ingest_urls: IngestUrls,
    pub video: VideoSettings,
    #[serde(default)]
    pub audio: AudioDefaults,
    #[serde(default)]
    pub general: GeneralSettings,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            platform: Platform::default(),
            ingest_urls: IngestUrls::default(),
            video: VideoSettings::default(),
            audio: AudioDefaults::default(),
            general: GeneralSettings::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let path = path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        serde_json::from_slice(
            &fs::read(path).map_err(|e| format!("Could not read settings: {e}"))?,
        )
        .map_err(|e| format!("Settings file is invalid and was left untouched: {e}"))
    }
    pub fn save(&self) -> Result<(), String> {
        let path = path()?;
        let parent = path.parent().ok_or("Settings path has no parent folder")?;
        fs::create_dir_all(parent).map_err(|e| format!("Could not create config folder: {e}"))?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| format!("Could not encode settings: {e}"))?;
        static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let temp = parent.join(format!(
            ".settings.json.tmp-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Could not create temporary settings file: {e}"))?;
        if let Err(error) =
            std::io::Write::write_all(&mut file, &bytes).and_then(|_| file.sync_all())
        {
            let _ = fs::remove_file(&temp);
            return Err(format!("Could not write settings safely: {error}"));
        }
        drop(file);
        super::replace_file(&temp, &path).inspect_err(|_| {
            let _ = fs::remove_file(&temp);
        })
    }
}
fn path() -> Result<PathBuf, String> {
    Ok(super::config_dir()?.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_without_new_fields_keep_old_defaults() {
        let parsed: AppConfig = serde_json::from_str(
            r#"{"platform":"twitch","ingestUrls":{"youtube":"yt","kick":"","twitch":"tw"},"video":{"automatic":true,"encoder":"NVENC","resolution":"1920x1080","frameRate":60,"bitrateKbps":6000,"keyframeIntervalSec":2}}"#,
        )
        .unwrap();
        assert!(parsed.general.confirm_before_stop);
        assert_eq!(parsed.audio.default_mic_id, "default-mic");
    }
}
