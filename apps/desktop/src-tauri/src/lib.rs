use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub state: String,
    pub live: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamStats {
    pub cpu_percent: u8,
    pub fps: u32,
    pub kbps: u32,
    pub dropped: u32,
    pub elapsed_ms: u64,
}

#[tauri::command]
fn get_app_state() -> AppState {
    AppState {
        state: "idle".into(),
        live: false,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
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
        .invoke_handler(tauri::generate_handler![get_app_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
