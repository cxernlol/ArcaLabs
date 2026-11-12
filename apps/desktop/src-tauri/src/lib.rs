mod audio;
mod capture;
mod commands;
mod config;
mod crash;
mod gpu;
mod keychain;
mod scenes;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(audio::AudioMixer::default())
        .manage(audio::AudioMeter::default())
        .manage(scenes::SceneStore::default())
        .manage(capture::CaptureRuntime::default())
        .setup(|app| {
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
            audio::get_audio_sources,
            audio::get_audio_tracks,
            audio::add_audio_track,
            audio::set_audio_track_settings,
            audio::remove_audio_track,
            audio::start_audio_meter,
            audio::stop_audio_meter,
            audio::get_audio_input_levels,
            scenes::get_scenes,
            scenes::create_scene,
            scenes::rename_scene,
            scenes::set_active_scene,
            scenes::delete_scene,
            scenes::reorder_scene,
            scenes::rename_source,
            scenes::set_source_visibility,
            scenes::set_source_transform,
            scenes::reorder_source,
            scenes::remove_source,
            scenes::get_capture_displays,
            scenes::add_display_source,
            capture::add_window_source,
            capture::reselect_window_source,
            capture::get_preview_frame,
            capture::stop_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
