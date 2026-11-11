mod commands;
mod config;
mod crash;
mod gpu;
mod keychain;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::Emitter;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StreamStats {
    cpu_percent: u8,
    fps: u32,
    kbps: u32,
    dropped: u32,
    elapsed_ms: u64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            // shortcut: panic hook holds AppHandle; trim if we add a crash reporter service
            static PANIC_HANDLE: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
            if let Ok(mut slot) = PANIC_HANDLE.lock() {
                *slot = Some(app.handle().clone());
            }
            let prev = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let msg = info.to_string();
                if let Ok(guard) = PANIC_HANDLE.lock() {
                    if let Some(app) = guard.as_ref() {
                        let _ = crash::append_crash_log(app, &msg);
                    }
                }
                prev(info);
            }));

            thread::spawn(move || {
                let stats = StreamStats {
                    cpu_percent: 4,
                    fps: 60,
                    kbps: 0,
                    dropped: 0,
                    elapsed_ms: 0,
                };
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let _ = handle.emit("stream:stats", stats.clone());
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::get_stream_destination,
            commands::set_stream_destination,
            commands::set_stream_key,
            commands::has_stream_key,
            commands::clear_stream_key,
            commands::get_video_settings,
            commands::set_video_settings,
            commands::get_app_config,
            commands::set_audio_defaults,
            commands::set_general_settings,
            commands::append_crash_log,
            commands::read_crash_log,
            commands::detect_gpu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
