use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Youtube,
    Kick,
    Twitch,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Youtube => "youtube",
            Platform::Kick => "kick",
            Platform::Twitch => "twitch",
        }
    }

    pub fn from_str_value(value: &str) -> Result<Self, String> {
        match value {
            "youtube" => Ok(Platform::Youtube),
            "kick" => Ok(Platform::Kick),
            "twitch" => Ok(Platform::Twitch),
            other => Err(format!("unknown platform: {other}")),
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
            youtube: "rtmp://a.rtmp.youtube.com/live2".into(),
            kick: String::new(),
            twitch: "rtmp://live.twitch.tv/app".into(),
        }
    }
}

impl IngestUrls {
    pub fn get(&self, platform: &Platform) -> &str {
        match platform {
            Platform::Youtube => &self.youtube,
            Platform::Kick => &self.kick,
            Platform::Twitch => &self.twitch,
        }
    }

    pub fn set(&mut self, platform: &Platform, url: String) {
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
            platform: Platform::Twitch,
            ingest_urls: IngestUrls::default(),
            video: VideoSettings::default(),
            audio: AudioDefaults::default(),
            general: GeneralSettings::default(),
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("config dir unavailable: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        let config = AppConfig::default();
        save_config(app, &config)?;
        return Ok(config);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("failed to read settings: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("failed to parse settings: {e}"))
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create config dir: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(config)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("failed to write settings: {e}"))
}
