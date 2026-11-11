use crate::config::{
    load_config, save_config, AppConfig, AudioDefaults, GeneralSettings, Platform, VideoSettings,
};
use crate::crash;
use crate::keychain;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub state: String,
    pub live: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDestination {
    pub platform: Platform,
    pub ingest_url: String,
    pub has_stream_key: bool,
}

#[tauri::command]
pub fn get_app_state() -> AppState {
    AppState {
        state: "idle".into(),
        live: false,
    }
}

#[tauri::command]
pub fn get_stream_destination(app: AppHandle) -> Result<StreamDestination, String> {
    let config = load_config(&app)?;
    let has = keychain::has_stream_key(&config.platform)?;
    Ok(StreamDestination {
        platform: config.platform.clone(),
        ingest_url: config.ingest_urls.get(&config.platform).to_string(),
        has_stream_key: has,
    })
}

#[tauri::command]
pub fn set_stream_destination(
    app: AppHandle,
    platform: Platform,
    ingest_url: String,
) -> Result<StreamDestination, String> {
    let mut config = load_config(&app)?;
    config.platform = platform.clone();
    config
        .ingest_urls
        .set(&platform, ingest_url.trim().to_string());
    save_config(&app, &config)?;
    let has = keychain::has_stream_key(&config.platform)?;
    let ingest_url = config.ingest_urls.get(&config.platform).to_string();
    Ok(StreamDestination {
        platform: config.platform,
        ingest_url,
        has_stream_key: has,
    })
}

#[tauri::command]
pub fn set_stream_key(platform: String, key: String) -> Result<bool, String> {
    let platform = Platform::from_str_value(&platform)?;
    keychain::set_stream_key(&platform, key.trim())?;
    Ok(true)
}

#[tauri::command]
pub fn has_stream_key(platform: String) -> Result<bool, String> {
    let platform = Platform::from_str_value(&platform)?;
    keychain::has_stream_key(&platform)
}

#[tauri::command]
pub fn clear_stream_key(platform: String) -> Result<bool, String> {
    let platform = Platform::from_str_value(&platform)?;
    keychain::clear_stream_key(&platform)?;
    Ok(false)
}

#[tauri::command]
pub fn get_video_settings(app: AppHandle) -> Result<VideoSettings, String> {
    Ok(load_config(&app)?.video)
}

#[tauri::command]
pub fn set_video_settings(app: AppHandle, settings: VideoSettings) -> Result<VideoSettings, String> {
    let mut config = load_config(&app)?;
    config.video = settings;
    save_config(&app, &config)?;
    Ok(config.video)
}

#[tauri::command]
pub fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    load_config(&app)
}

#[tauri::command]
pub fn set_audio_defaults(
    app: AppHandle,
    audio: AudioDefaults,
) -> Result<AudioDefaults, String> {
    let mut config = load_config(&app)?;
    config.audio = audio;
    save_config(&app, &config)?;
    Ok(config.audio)
}

#[tauri::command]
pub fn set_general_settings(
    app: AppHandle,
    general: GeneralSettings,
) -> Result<GeneralSettings, String> {
    let mut config = load_config(&app)?;
    config.general = general;
    save_config(&app, &config)?;
    Ok(config.general)
}

#[tauri::command]
pub fn append_crash_log(app: AppHandle, message: String) -> Result<bool, String> {
    crash::append_crash_log(&app, &message)?;
    Ok(true)
}

#[tauri::command]
pub fn read_crash_log(app: AppHandle) -> Result<String, String> {
    crash::read_crash_log(&app)
}

#[tauri::command]
pub fn detect_gpu() -> Result<crate::gpu::GpuInfo, String> {
    crate::gpu::detect_gpu()
}
