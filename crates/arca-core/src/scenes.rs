use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

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
    #[serde(default, alias = "displayId")]
    pub capture_target_id: Option<String>,
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
pub struct SceneStore;
impl SceneStore {
    pub fn load(&self) -> Result<SceneCollection, String> {
        let path = scene_path()?;
        if !path.exists() {
            let data = SceneCollection::default();
            self.save(&data)?;
            return Ok(data);
        }
        let data: SceneCollection = serde_json::from_slice(
            &fs::read(path).map_err(|e| format!("Could not read scenes: {e}"))?,
        )
        .map_err(|e| format!("Scenes file is invalid and was left untouched: {e}"))?;
        validate(&data)?;
        Ok(data)
    }
    pub fn save(&self, data: &SceneCollection) -> Result<(), String> {
        validate(data)?;
        let path = scene_path()?;
        let parent = path
            .parent()
            .ok_or("Scene config path has no parent folder")?;
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create scene config folder: {e}"))?;
        let bytes =
            serde_json::to_vec_pretty(data).map_err(|e| format!("Could not encode scenes: {e}"))?;
        let temp = parent.join(format!(
            ".{SCENE_FILE}.tmp-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Could not create temporary scene file: {e}"))?;
        if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&temp);
            return Err(format!("Could not write scenes safely: {error}"));
        }
        drop(file);
        super::replace_file(&temp, &path).inspect_err(|_| {
            let _ = fs::remove_file(&temp);
        })
    }
}

pub fn validate(data: &SceneCollection) -> Result<(), String> {
    if data.version != SCHEMA_VERSION || data.scenes.is_empty() {
        return Err("Scene file has an unsupported version or no scenes".into());
    }
    let mut scene_ids = Vec::new();
    let mut scene_names = Vec::new();
    for scene in &data.scenes {
        valid_name(&scene.name)?;
        if scene_ids.contains(&scene.id) {
            return Err("Scene file contains duplicate scene IDs".into());
        }
        if scene_names
            .iter()
            .any(|n: &String| n.eq_ignore_ascii_case(&scene.name))
        {
            return Err("Scene file contains duplicate scene names".into());
        }
        scene_ids.push(scene.id.clone());
        scene_names.push(scene.name.clone());
        let mut source_ids = Vec::new();
        let mut source_names = Vec::new();
        for source in &scene.sources {
            valid_name(&source.name)?;
            if source_ids.contains(&source.id) {
                return Err("Scene file contains duplicate source IDs".into());
            }
            if source_names
                .iter()
                .any(|n: &String| n.eq_ignore_ascii_case(&source.name))
            {
                return Err("Scene file contains duplicate source names".into());
            }
            if [source.x, source.y, source.width, source.height]
                .iter()
                .any(|n| !n.is_finite())
                || source.x < 0.0
                || source.y < 0.0
                || source.width <= 0.0
                || source.height <= 0.0
                || source.x + source.width > 1.0
                || source.y + source.height > 1.0
            {
                return Err("Source bounds must fit within the scene canvas".into());
            }
            if source.kind == SourceKind::Display && source.capture_target_id.is_none() {
                return Err("Scene file contains an invalid capture target".into());
            }
            source_ids.push(source.id.clone());
            source_names.push(source.name.clone());
        }
    }
    if !scene_ids.contains(&data.active_scene_id) {
        return Err("Scene file has no valid active scene".into());
    }
    Ok(())
}

pub fn valid_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err("Names must contain 1–64 visible characters".into());
    }
    Ok(name.to_owned())
}

pub fn make_id(prefix: &str) -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{prefix}-{time:x}-{:x}",
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}
fn scene_path() -> Result<PathBuf, String> {
    Ok(super::config_dir()?.join(SCENE_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_out_of_canvas_and_nan_transforms() {
        let mut data = SceneCollection::default();
        data.scenes[0].sources.push(SceneSource {
            id: "s".into(),
            name: "Display".into(),
            kind: SourceKind::Display,
            capture_target_id: Some("display:0".into()),
            visible: true,
            x: 0.9,
            y: 0.0,
            width: 0.2,
            height: 1.0,
        });
        assert!(validate(&data).is_err());
        data.scenes[0].sources[0].x = f32::NAN;
        assert!(validate(&data).is_err());
    }
    #[test]
    fn rejects_duplicate_names_case_insensitively() {
        let mut data = SceneCollection::default();
        data.scenes.push(Scene {
            id: "other".into(),
            name: "main".into(),
            sources: vec![],
        });
        assert!(
            validate(&data)
                .unwrap_err()
                .contains("duplicate scene names")
        );
    }

    #[test]
    fn reads_legacy_display_id_without_losing_the_capture_target() {
        let json = r#"{"version":1,"activeSceneId":"scene-1","scenes":[{"id":"scene-1","name":"Main","sources":[{"id":"source-1","name":"Display 1","kind":"display","displayId":"display:0","visible":true,"x":0.0,"y":0.0,"width":1.0,"height":1.0}]}]}"#;
        let data: SceneCollection = serde_json::from_str(json).unwrap();
        validate(&data).unwrap();
        assert_eq!(
            data.scenes[0].sources[0].capture_target_id.as_deref(),
            Some("display:0")
        );
    }
}
