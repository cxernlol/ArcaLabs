use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};

const SCENE_FILE: &str = "scenes.json";
const SCHEMA_VERSION: u32 = 1;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneCollection {
    pub version: u32,
    pub active_scene_id: String,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub sources: Vec<SceneSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSource {
    pub id: String,
    pub name: String,
    pub kind: SourceKind,
    pub display_id: Option<String>,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    Display,
    Window,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDisplay {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub height: u32,
}

impl Default for SceneCollection {
    fn default() -> Self {
        let scene = Scene {
            id: make_id("scene"),
            name: "Main".into(),
            sources: Vec::new(),
        };
        Self {
            version: SCHEMA_VERSION,
            active_scene_id: scene.id.clone(),
            scenes: vec![scene],
        }
    }
}

#[derive(Default)]
pub struct SceneStore {
    data: Mutex<Option<SceneCollection>>,
}

impl SceneStore {
    pub fn load(&self, app: &AppHandle) -> Result<SceneCollection, String> {
        let mut data = self.lock()?;
        if data.is_none() {
            let path = scene_path(app)?;
            let collection = if path.exists() {
                let bytes = fs::read(&path)
                    .map_err(|error| format!("Could not read scenes: {error}"))?;
                let collection: SceneCollection = serde_json::from_slice(&bytes)
                    .map_err(|error| format!("Scenes file is invalid and was left untouched: {error}"))?;
                validate_collection(&collection)?;
                collection
            } else {
                let collection = SceneCollection::default();
                write_collection(&path, &collection)?;
                collection
            };
            *data = Some(collection);
        }
        data.as_ref()
            .cloned()
            .ok_or_else(|| "Scene data is unavailable".into())
    }

    fn update(
        &self,
        app: &AppHandle,
        update: impl FnOnce(&mut SceneCollection) -> Result<(), String>,
    ) -> Result<SceneCollection, String> {
        let mut data = self.lock()?;
        if data.is_none() {
            drop(data);
            self.load(app)?;
            data = self.lock()?;
        }
        let current = data
            .as_ref()
            .ok_or_else(|| "Scene data is unavailable".to_owned())?;
        let mut next = current.clone();
        update(&mut next)?;
        next.version = SCHEMA_VERSION;
        validate_collection(&next)?;
        write_collection(&scene_path(app)?, &next)?;
        *data = Some(next.clone());
        Ok(next)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<SceneCollection>>, String> {
        self.data
            .lock()
            .map_err(|_| "Scene state is unavailable".into())
    }
}

#[tauri::command]
pub fn get_scenes(app: AppHandle, store: State<'_, SceneStore>) -> Result<SceneCollection, String> {
    store.load(&app)
}

#[tauri::command]
pub fn create_scene(
    app: AppHandle,
    store: State<'_, SceneStore>,
    name: String,
) -> Result<SceneCollection, String> {
    let name = valid_name(&name)?;
    store.update(&app, move |collection| {
        ensure_unique_scene_name(collection, &name, None)?;
        let scene = Scene {
            id: make_id("scene"),
            name,
            sources: Vec::new(),
        };
        collection.active_scene_id = scene.id.clone();
        collection.scenes.push(scene);
        Ok(())
    })
}

#[tauri::command]
pub fn rename_scene(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    name: String,
) -> Result<SceneCollection, String> {
    let name = valid_name(&name)?;
    store.update(&app, move |collection| {
        ensure_unique_scene_name(collection, &name, Some(&scene_id))?;
        let scene = collection
            .scenes
            .iter_mut()
            .find(|scene| scene.id == scene_id)
            .ok_or_else(|| "Scene no longer exists".to_owned())?;
        scene.name = name;
        Ok(())
    })
}

#[tauri::command]
pub fn set_active_scene(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        if !collection.scenes.iter().any(|scene| scene.id == scene_id) {
            return Err("Scene no longer exists".into());
        }
        collection.active_scene_id = scene_id;
        Ok(())
    })
}

#[tauri::command]
pub fn delete_scene(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        if collection.scenes.len() == 1 {
            return Err("Keep at least one scene".into());
        }
        let index = collection
            .scenes
            .iter()
            .position(|scene| scene.id == scene_id)
            .ok_or_else(|| "Scene no longer exists".to_owned())?;
        collection.scenes.remove(index);
        if collection.active_scene_id == scene_id {
            collection.active_scene_id = collection.scenes[index.min(collection.scenes.len() - 1)]
                .id
                .clone();
        }
        Ok(())
    })
}

#[tauri::command]
pub fn reorder_scene(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    destination: usize,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        let from = collection
            .scenes
            .iter()
            .position(|scene| scene.id == scene_id)
            .ok_or_else(|| "Scene no longer exists".to_owned())?;
        if destination >= collection.scenes.len() {
            return Err("Scene position is out of range".into());
        }
        let scene = collection.scenes.remove(from);
        collection.scenes.insert(destination, scene);
        Ok(())
    })
}

#[tauri::command]
pub fn rename_source(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    source_id: String,
    name: String,
) -> Result<SceneCollection, String> {
    let name = valid_name(&name)?;
    store.update(&app, move |collection| {
        let scene = find_scene_mut(collection, &scene_id)?;
        ensure_unique_source_name(scene, &name, Some(&source_id))?;
        let source = find_source_mut(scene, &source_id)?;
        source.name = name;
        Ok(())
    })
}

#[tauri::command]
pub fn set_source_visibility(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    source_id: String,
    visible: bool,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        find_source_mut(find_scene_mut(collection, &scene_id)?, &source_id)?.visible = visible;
        Ok(())
    })
}

#[tauri::command]
pub fn set_source_transform(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    source_id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Result<SceneCollection, String> {
    validate_transform(x, y, width, height)?;
    store.update(&app, move |collection| {
        let source = find_source_mut(find_scene_mut(collection, &scene_id)?, &source_id)?;
        source.x = x;
        source.y = y;
        source.width = width;
        source.height = height;
        Ok(())
    })
}

#[tauri::command]
pub fn reorder_source(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    source_id: String,
    destination: usize,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        let scene = find_scene_mut(collection, &scene_id)?;
        let from = scene
            .sources
            .iter()
            .position(|source| source.id == source_id)
            .ok_or_else(|| "Source no longer exists".to_owned())?;
        if destination >= scene.sources.len() {
            return Err("Source position is out of range".into());
        }
        let source = scene.sources.remove(from);
        scene.sources.insert(destination, source);
        Ok(())
    })
}

#[tauri::command]
pub fn remove_source(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    source_id: String,
) -> Result<SceneCollection, String> {
    store.update(&app, move |collection| {
        let scene = find_scene_mut(collection, &scene_id)?;
        let original_len = scene.sources.len();
        scene.sources.retain(|source| source.id != source_id);
        if scene.sources.len() == original_len {
            return Err("Source no longer exists".into());
        }
        Ok(())
    })
}

#[tauri::command]
pub fn get_capture_displays() -> Result<Vec<CaptureDisplay>, String> {
    enumerate_displays()
}

#[tauri::command]
pub fn add_display_source(
    app: AppHandle,
    store: State<'_, SceneStore>,
    scene_id: String,
    display_id: String,
) -> Result<SceneCollection, String> {
    let display = enumerate_displays()?
        .into_iter()
        .find(|display| display.id == display_id)
        .ok_or_else(|| "That display is no longer available. Refresh the list and try again.".to_owned())?;
    add_source(
        &app,
        &store,
        scene_id,
        display.label,
        SourceKind::Display,
        Some(display.id),
    )
}

#[cfg(windows)]
fn enumerate_displays() -> Result<Vec<CaptureDisplay>, String> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }
        .map_err(|error| format!("Could not enumerate displays: {error}"))?;
    let mut displays = Vec::new();
    let mut adapter_index = 0;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(adapter_index) } {
        let mut output_index = 0;
        while let Ok(output) = unsafe { adapter.EnumOutputs(output_index) } {
            if let Ok(desc) = unsafe { output.GetDesc() } {
                if desc.AttachedToDesktop.as_bool() {
                    let id = String::from_utf16_lossy(
                        &desc.DeviceName[..desc.DeviceName.iter().position(|c| *c == 0).unwrap_or(desc.DeviceName.len())],
                    );
                    let width = (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left).max(0) as u32;
                    let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top).max(0) as u32;
                    if !id.is_empty() && width > 0 && height > 0 {
                        displays.push(CaptureDisplay {
                            label: format!("Display {}", displays.len() + 1),
                            id,
                            width,
                            height,
                        });
                    }
                }
            }
            output_index += 1;
        }
        adapter_index += 1;
    }
    Ok(displays)
}

#[cfg(not(windows))]
fn enumerate_displays() -> Result<Vec<CaptureDisplay>, String> {
    Err("Display capture is available on Windows only.".into())
}

pub(crate) fn add_source(
    app: &AppHandle,
    store: &SceneStore,
    scene_id: String,
    name: String,
    kind: SourceKind,
    display_id: Option<String>,
) -> Result<SceneCollection, String> {
    let name = valid_name(&name)?;
    if (kind == SourceKind::Display) != display_id.is_some() {
        return Err("Display sources require a display ID; window sources must be reselected after restart".into());
    }
    store.update(app, move |collection| {
        let scene = find_scene_mut(collection, &scene_id)?;
        ensure_unique_source_name(scene, &name, None)?;
        let has_source = !scene.sources.is_empty();
        let (x, y, width, height) = if has_source {
            (0.75, 0.75, 0.25, 0.25)
        } else {
            (0.0, 0.0, 1.0, 1.0)
        };
        scene.sources.push(SceneSource {
            id: make_id("source"),
            name,
            kind,
            display_id,
            visible: true,
            x,
            y,
            width,
            height,
        });
        Ok(())
    })
}

fn find_scene_mut<'a>(collection: &'a mut SceneCollection, id: &str) -> Result<&'a mut Scene, String> {
    collection
        .scenes
        .iter_mut()
        .find(|scene| scene.id == id)
        .ok_or_else(|| "Scene no longer exists".into())
}

fn find_source_mut<'a>(scene: &'a mut Scene, id: &str) -> Result<&'a mut SceneSource, String> {
    scene
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| "Source no longer exists".into())
}

fn ensure_unique_scene_name(
    collection: &SceneCollection,
    name: &str,
    except_id: Option<&str>,
) -> Result<(), String> {
    if collection.scenes.iter().any(|scene| {
        Some(scene.id.as_str()) != except_id && scene.name.eq_ignore_ascii_case(name)
    }) {
        return Err("A scene with that name already exists".into());
    }
    Ok(())
}

fn ensure_unique_source_name(scene: &Scene, name: &str, except_id: Option<&str>) -> Result<(), String> {
    if scene.sources.iter().any(|source| {
        Some(source.id.as_str()) != except_id && source.name.eq_ignore_ascii_case(name)
    }) {
        return Err("A source with that name already exists in this scene".into());
    }
    Ok(())
}

fn valid_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err("Names must contain 1–64 visible characters".into());
    }
    Ok(name.to_owned())
}

fn validate_transform(x: f32, y: f32, width: f32, height: f32) -> Result<(), String> {
    if [x, y, width, height].iter().any(|value| !value.is_finite())
        || x < 0.0
        || y < 0.0
        || width <= 0.0
        || height <= 0.0
        || x + width > 1.0
        || y + height > 1.0
    {
        return Err("Source bounds must fit within the scene canvas".into());
    }
    Ok(())
}

fn validate_collection(collection: &SceneCollection) -> Result<(), String> {
    if collection.version != SCHEMA_VERSION || collection.scenes.is_empty() {
        return Err("Scene file has an unsupported version or no scenes".into());
    }
    let mut scene_ids = Vec::new();
    for scene in &collection.scenes {
        valid_name(&scene.name)?;
        ensure_unique_scene_name_against(&scene_ids, &scene.name)?;
        if scene_ids.iter().any(|id: &String| id == &scene.id) {
            return Err("Scene file contains duplicate scene IDs".into());
        }
        scene_ids.push(scene.id.clone());
        let mut source_ids = Vec::new();
        let mut source_names = Vec::new();
        for source in &scene.sources {
            valid_name(&source.name)?;
            validate_transform(source.x, source.y, source.width, source.height)?;
            if source_ids.iter().any(|id: &String| id == &source.id) {
                return Err("Scene file contains duplicate source IDs".into());
            }
            ensure_unique_source_name_against(&source_names, &source.name)?;
            if (source.kind == SourceKind::Display) != source.display_id.is_some() {
                return Err("Scene file contains an invalid capture target".into());
            }
            source_ids.push(source.id.clone());
            source_names.push(source.name.clone());
        }
    }
    if !scene_ids.iter().any(|id| id == &collection.active_scene_id) {
        return Err("Scene file has no valid active scene".into());
    }
    Ok(())
}

fn ensure_unique_scene_name_against(names: &[String], name: &str) -> Result<(), String> {
    if names.iter().any(|existing| existing.eq_ignore_ascii_case(name)) {
        return Err("Scene file contains duplicate scene names".into());
    }
    Ok(())
}

fn ensure_unique_source_name_against(names: &[String], name: &str) -> Result<(), String> {
    if names.iter().any(|existing| existing.eq_ignore_ascii_case(name)) {
        return Err("Scene file contains duplicate source names".into());
    }
    Ok(())
}

fn make_id(prefix: &str) -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{time:x}-{sequence:x}")
}

fn scene_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("App config folder is unavailable: {error}"))?;
    Ok(dir.join(SCENE_FILE))
}

fn write_collection(path: &PathBuf, collection: &SceneCollection) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Scene config path has no parent folder".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| format!("Could not create scene config folder: {error}"))?;
    let bytes = serde_json::to_vec_pretty(collection)
        .map_err(|error| format!("Could not encode scenes: {error}"))?;
    let temp = parent.join(format!(".{SCENE_FILE}.tmp-{}-{}", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed)));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| format!("Could not create temporary scene file: {error}"))?;
    let write_result = file.write_all(&bytes).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(format!("Could not write scenes safely: {error}"));
    }
    replace_file(&temp, path).map_err(|error| {
        let _ = fs::remove_file(&temp);
        error
    })
}

#[cfg(windows)]
fn replace_file(temp: &std::path::Path, destination: &std::path::Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW, MOVE_FILE_FLAGS};
    let temp: Vec<u16> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            windows::core::PCWSTR(temp.as_ptr()),
            windows::core::PCWSTR(destination.as_ptr()),
            MOVE_FILE_FLAGS(MOVEFILE_REPLACE_EXISTING.0 | MOVEFILE_WRITE_THROUGH.0),
        )
    }
    .map_err(|error| format!("Could not replace scene file safely: {error}"))
}

#[cfg(not(windows))]
fn replace_file(temp: &std::path::Path, destination: &std::path::Path) -> Result<(), String> {
    fs::rename(temp, destination)
        .map_err(|error| format!("Could not replace scene file safely: {error}"))
}
