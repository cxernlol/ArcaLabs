use arca_core::{
    config::AppConfig,
    scenes::{SceneCollection, SceneStore},
};
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(windows)]
mod video_stream;
#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        CreateWindowExW, SW_HIDE, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW, SetWindowPos,
        ShowWindow, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    },
};

const BLACK: egui::Color32 = egui::Color32::from_rgb(0x0B, 0x0B, 0x0D);
const WHITE: egui::Color32 = egui::Color32::from_rgb(0xF2, 0xF4, 0xF8);
const LIME: egui::Color32 = egui::Color32::from_rgb(0xC6, 0xF1, 0x4A);
const PANEL: egui::Color32 = egui::Color32::from_rgb(0x17, 0x18, 0x1C);
const SUNKEN: egui::Color32 = egui::Color32::from_rgb(0x10, 0x11, 0x13);
const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2D, 0x30, 0x33);
const MUTED: egui::Color32 = egui::Color32::from_rgb(0xA3, 0xA4, 0xAC);
const SELECTED: egui::Color32 = egui::Color32::from_rgb(0x26, 0x2D, 0x1B);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Studio,
    Settings,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    General,
    Stream,
    Video,
    Audio,
    Hotkeys,
}

struct ArcaApp {
    scenes: SceneCollection,
    scene_store: SceneStore,
    config: AppConfig,
    stream_key: String,
    key_saved: bool,
    screen: Screen,
    tab: SettingsTab,
    error: Option<String>,
    rename_scene: Option<(String, String)>,
    delete_scene: Option<String>,
    capture_picker: Option<arca_core::scenes::SourceKind>,
    capture_picker_source: Option<String>,
    capture_targets: Vec<arca_capture::windows::CaptureTarget>,
    audio_picker: Option<arca_audio::AudioSourceKind>,
    audio_candidates: Vec<arca_audio::AudioSource>,
    audio_tracks: Vec<arca_audio::AudioSource>,
    audio_meter: Option<arca_audio::wasapi::AudioMeter>,
    hardware_encoder: Result<arca_encode::HardwareEncoder, String>,
    preview_hwnd: Option<isize>,
    capture_source_id: Option<String>,
    capture_session: Option<arca_capture::windows::CaptureSession>,
    capture_frames: std::sync::Arc<arca_capture::windows::CaptureFrames>,
    capture_frame_rate: Option<u32>,
    capture_error: Option<String>,
    #[cfg(windows)]
    video_stream: Option<video_stream::VideoStreamSession>,
    #[cfg(windows)]
    stream_capture_identity: Option<(String, String)>,
}

impl ArcaApp {
    fn new() -> Self {
        let scene_store = SceneStore;
        let scenes = scene_store.load().unwrap_or_else(|error| {
            eprintln!("Could not load scenes: {error}");
            SceneCollection::default()
        });
        let config = AppConfig::load().unwrap_or_else(|error| {
            eprintln!("Could not load settings: {error}");
            AppConfig::default()
        });
        let key_saved = matches!(
            arca_core::keychain::get_stream_key(config.platform),
            Ok(Some(_))
        );
        Self {
            scenes,
            scene_store,
            config,
            stream_key: String::new(),
            key_saved,
            screen: Screen::Studio,
            tab: SettingsTab::General,
            error: None,
            rename_scene: None,
            delete_scene: None,
            capture_picker: None,
            capture_picker_source: None,
            capture_targets: arca_capture::windows::enumerate_targets().unwrap_or_default(),
            audio_picker: None,
            audio_candidates: Vec::new(),
            audio_tracks: Vec::new(),
            audio_meter: None,
            hardware_encoder: arca_encode::discover_hardware_encoder()
                .map_err(|error| error.to_string()),
            preview_hwnd: None,
            capture_source_id: None,
            capture_session: None,
            capture_frames: arca_capture::windows::CaptureFrames::new(3)
                .expect("capture queue capacity is non-zero"),
            capture_frame_rate: None,
            capture_error: None,
            #[cfg(windows)]
            video_stream: None,
            #[cfg(windows)]
            stream_capture_identity: None,
        }
    }

    fn save_scenes(&mut self) {
        if let Err(error) = self.scene_store.save(&self.scenes) {
            self.error = Some(error);
        }
    }

    fn stream_status(&self) -> String {
        #[cfg(windows)]
        if let Some(stream) = &self.video_stream {
            return stream.status();
        }
        "Offline".into()
    }

    fn stream_active(&self) -> bool {
        #[cfg(windows)]
        {
            self.video_stream.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    #[cfg(windows)]
    fn start_video_stream(&mut self) -> Result<(), String> {
        if let Err(error) = &self.hardware_encoder {
            return Err(format!("Hardware H.264 is unavailable: {error}"));
        }
        if !self
            .capture_session
            .as_ref()
            .is_some_and(|session| !session.is_finished())
        {
            return Err("Add a display or window source and select its target first.".into());
        }
        if !self.scenes.scenes.iter().any(|scene| {
            scene.id == self.scenes.active_scene_id
                && scene
                    .sources
                    .iter()
                    .any(|source| source.visible && source.capture_target_id.is_some())
        }) {
            return Err("The active scene needs a visible display or window source.".into());
        }
        let platform = self.config.platform;
        let ingest_url = self.config.ingest_urls.get(platform).to_owned();
        arca_publish::validate_ingest_url(&ingest_url).map_err(|error| error.to_string())?;
        let stream_key = arca_core::keychain::get_stream_key(platform)?
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| "Add a stream key in Settings before going live.".to_owned())?;
        let source_identity = self
            .scenes
            .scenes
            .iter()
            .find(|scene| scene.id == self.scenes.active_scene_id)
            .and_then(|scene| {
                scene.sources.iter().find_map(|source| {
                    (source.visible)
                        .then(|| {
                            source
                                .capture_target_id
                                .as_ref()
                                .map(|target| (source.id.clone(), target.clone()))
                        })
                        .flatten()
                })
            })
            .ok_or_else(|| "The active scene needs a visible capture target.".to_owned())?;
        if self.capture_source_id.as_deref() != Some(source_identity.0.as_str()) {
            return Err("Wait for the selected capture source to become ready.".into());
        }
        let (width, height) = self
            .config
            .video
            .resolution
            .split_once('x')
            .and_then(|(width, height)| {
                Some((width.parse::<u32>().ok()?, height.parse::<u32>().ok()?))
            })
            .ok_or_else(|| "Choose a valid output resolution in Settings.".to_owned())?;

        self.video_stream = Some(video_stream::VideoStreamSession::start(
            self.capture_frames.clone(),
            ingest_url,
            stream_key,
            arca_encode::H264Settings {
                width,
                height,
                frame_rate: self.config.video.frame_rate,
                bitrate: self.config.video.bitrate_kbps.saturating_mul(1000),
            },
            self.audio_tracks.clone(),
        )?);
        self.stream_capture_identity = Some(source_identity);
        Ok(())
    }

    #[cfg(windows)]
    fn stop_video_stream(&mut self, message: &str) {
        if let Some(stream) = &self.video_stream {
            stream.stop();
        }
        self.video_stream = None;
        self.stream_capture_identity = None;
        self.error = Some(message.to_owned());
    }

    fn hide_native_preview(&mut self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.preview_hwnd {
            unsafe {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_HIDE);
            }
        }
    }

    fn update_native_preview(
        &mut self,
        frame: &eframe::Frame,
        rect: egui::Rect,
        ctx: &egui::Context,
    ) {
        #[cfg(windows)]
        {
            let selected_capture = self
                .scenes
                .scenes
                .iter()
                .find(|scene| scene.id == self.scenes.active_scene_id)
                .and_then(|scene| {
                    scene.sources.iter().find_map(|source| {
                        (source.visible)
                            .then(|| {
                                source
                                    .capture_target_id
                                    .as_ref()
                                    .map(|target| (source.id.clone(), target.clone()))
                            })
                            .flatten()
                    })
                });
            if self.video_stream.is_some() && selected_capture != self.stream_capture_identity {
                if let Some(stream) = &self.video_stream {
                    stream.stop();
                }
                self.video_stream = None;
                self.stream_capture_identity = None;
                self.error =
                    Some("Stream stopped because the active capture source changed.".into());
            }
            let source = self
                .scenes
                .scenes
                .iter()
                .find(|scene| scene.id == self.scenes.active_scene_id)
                .and_then(|scene| {
                    scene
                        .sources
                        .iter()
                        .find(|source| source.visible && source.capture_target_id.is_some())
                });
            let Some(source) = source else {
                self.capture_session = None;
                self.capture_source_id = None;
                self.hide_native_preview();
                return;
            };
            let source_id = source.id.clone();
            let target_id = source.capture_target_id.clone().unwrap_or_default();
            let target_name = source.name.clone();
            let target = self
                .capture_targets
                .iter()
                .find(|target| target.id == target_id)
                .or_else(|| {
                    self.capture_targets
                        .iter()
                        .find(|target| target.label == target_name)
                })
                .cloned();
            let Some(target) = target else {
                self.capture_session = None;
                self.capture_source_id = None;
                self.capture_error = Some(
                    "This capture target is no longer available. Select the window again.".into(),
                );
                if self.video_stream.is_some() {
                    self.stop_video_stream(
                        "Stream stopped because its capture target disappeared.",
                    );
                }
                self.hide_native_preview();
                return;
            };

            let frame_rate = self.config.video.frame_rate;
            if self.capture_source_id.as_deref() != Some(&source_id)
                || self.capture_frame_rate != Some(frame_rate)
            {
                if let Some(session) = self.capture_session.take() {
                    let _ = session.stop();
                }
                self.capture_error = None;
                let parent = match frame.window_handle().map(|handle| handle.as_raw()) {
                    Ok(RawWindowHandle::Win32(handle)) => HWND(handle.hwnd.get() as *mut _),
                    Ok(_) => {
                        self.capture_error =
                            Some("Native preview requires a Windows window handle.".into());
                        return;
                    }
                    Err(error) => {
                        self.capture_error =
                            Some(format!("Could not access the app window: {error}"));
                        return;
                    }
                };
                if self.preview_hwnd.is_none() {
                    let child = unsafe {
                        CreateWindowExW(
                            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                            windows::core::w!("STATIC"),
                            windows::core::w!(""),
                            WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
                            0,
                            0,
                            1,
                            1,
                            Some(parent),
                            None,
                            None,
                            None,
                        )
                    };
                    match child {
                        Ok(hwnd) => self.preview_hwnd = Some(hwnd.0 as isize),
                        Err(error) => {
                            self.capture_error = Some(format!(
                                "Could not create the native preview surface: {error}"
                            ));
                            return;
                        }
                    }
                }
                let hwnd = self.preview_hwnd.unwrap();
                self.capture_source_id = Some(source_id);
                match arca_capture::windows::start_capture(
                    &target,
                    hwnd,
                    self.capture_frames.clone(),
                    frame_rate,
                ) {
                    Ok(session) => {
                        self.capture_frame_rate = Some(frame_rate);
                        self.capture_session = Some(session);
                    }
                    Err(error) => self.capture_error = Some(error),
                }
            } else if self
                .capture_session
                .as_ref()
                .is_some_and(|session| session.is_finished())
            {
                self.capture_error = self
                    .capture_session
                    .take()
                    .and_then(|session| session.stop().err())
                    .map(|error| format!("Capture stopped: {error}"))
                    .or_else(|| Some("Capture stopped unexpectedly.".into()));
                if self.video_stream.is_some() {
                    self.stop_video_stream("Stream stopped because capture ended.");
                }
            }

            if let Some(hwnd) = self.preview_hwnd {
                let scale = ctx.pixels_per_point();
                let source_ratio = target.width as f32 / target.height.max(1) as f32;
                let mut size = rect.size() * scale;
                if size.x / size.y.max(1.0) > source_ratio {
                    size.x = size.y * source_ratio;
                } else {
                    size.y = size.x / source_ratio;
                }
                let origin = rect.min * scale + (rect.size() * scale - size) * 0.5;
                let result = unsafe {
                    SetWindowPos(
                        HWND(hwnd as *mut _),
                        None,
                        origin.x.round() as i32,
                        origin.y.round() as i32,
                        size.x.round().max(1.0) as i32,
                        size.y.round().max(1.0) as i32,
                        SWP_NOACTIVATE | SWP_NOZORDER | SWP_SHOWWINDOW,
                    )
                };
                if let Err(error) = result {
                    self.capture_error =
                        Some(format!("Could not position the native preview: {error}"));
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (frame, rect, ctx);
        }
    }

    fn theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BLACK;
        visuals.window_fill = PANEL;
        visuals.extreme_bg_color = SUNKEN;
        visuals.faint_bg_color = PANEL;
        visuals.selection.bg_fill = SELECTED;
        visuals.selection.stroke.color = LIME;
        visuals.widgets.active.bg_fill = LIME;
        visuals.widgets.active.fg_stroke.color = BLACK;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x27, 0x29, 0x2C);
        visuals.widgets.inactive.bg_fill = PANEL;
        visuals.widgets.noninteractive.bg_fill = PANEL;
        visuals.widgets.noninteractive.bg_stroke.color = BORDER;
        visuals.widgets.inactive.bg_stroke.color = BORDER;
        visuals.widgets.hovered.bg_stroke.color = MUTED;
        visuals.widgets.active.bg_stroke.color = LIME;
        visuals.widgets.inactive.fg_stroke.color = WHITE;
        visuals.widgets.hovered.fg_stroke.color = WHITE;
        visuals.widgets.noninteractive.fg_stroke.color = WHITE;
        visuals.window_corner_radius = egui::CornerRadius::same(8);
        visuals.menu_corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        visuals.window_shadow = egui::Shadow::NONE;
        visuals.popup_shadow = egui::Shadow::NONE;
        ctx.set_visuals(visuals);
        let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.window_margin = egui::Margin::same(14);
        ctx.set_style_of(egui::Theme::Dark, style);
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(if self.screen == Screen::Studio {
                    "Studio"
                } else {
                    "Settings"
                })
                .size(18.0)
                .strong()
                .color(WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(self.stream_status()).color(MUTED));
                if self.screen == Screen::Studio
                    && ui
                        .add_enabled(!self.stream_active(), egui::Button::new("Settings"))
                        .clicked()
                {
                    self.screen = Screen::Settings;
                }
                if self.screen == Screen::Settings && ui.button("← Back").clicked() {
                    self.screen = Screen::Studio;
                }
            });
        });
        ui.separator();
    }

    fn studio(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        #[cfg(windows)]
        let (mut start_stream, mut stop_stream) = (false, false);
        let available = ui.available_size();
        let control_height = (available.y * 0.31)
            .clamp(190.0, 300.0)
            .min(available.y * 0.60);
        let preview_height = (available.y - control_height - 22.0).max(120.0);
        let preview = egui::Frame::new()
            .fill(SUNKEN)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_height(preview_height);
                ui.set_max_height(preview_height);
                ui.vertical_centered(|ui| {
                    if let Some(error) = &self.capture_error {
                        ui.add_space((preview_height * 0.36).max(10.0));
                        ui.heading("Preview unavailable");
                        ui.label(error);
                    } else if self.capture_session.is_none() {
                        ui.add_space((preview_height * 0.36).max(10.0));
                        ui.heading("Preview unavailable");
                        let needs_target = self
                            .scenes
                            .scenes
                            .iter()
                            .find(|scene| scene.id == self.scenes.active_scene_id)
                            .is_some_and(|scene| {
                                scene.sources.iter().any(|source| {
                                    source.visible && source.capture_target_id.is_none()
                                })
                            });
                        ui.label(if needs_target {
                            "Select a target for this source below."
                        } else {
                            "Add a display or window source to preview it here."
                        });
                    }
                });
            });
        self.update_native_preview(frame, preview.response.rect.shrink(1.0), ui.ctx());
        ui.add_space(10.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), control_height),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                ui.columns(4, |columns| {
                    for column in columns.iter() {
                        column.painter().rect(
                            column.max_rect().shrink(0.5),
                            egui::CornerRadius::same(8),
                            PANEL,
                            egui::Stroke::new(1.0, BORDER),
                            egui::StrokeKind::Inside,
                        );
                    }
                    for column in columns.iter_mut() {
                        column.add_space(8.0);
                    }
                    columns[0].heading(format!("Scenes · {}", self.scenes.scenes.len()));
                    columns[0].separator();
                    egui::ScrollArea::vertical().id_salt("scene-list").show(
                        &mut columns[0],
                        |ui| {
                            let mut active = self.scenes.active_scene_id.clone();
                            let scene_rows: Vec<_> = self
                                .scenes
                                .scenes
                                .iter()
                                .map(|scene| (scene.id.clone(), scene.name.clone()))
                                .collect();
                            for (scene_id, scene_name) in scene_rows {
                                let response = ui.selectable_label(scene_id == active, &scene_name);
                                if response.clicked() {
                                    active = scene_id.clone();
                                }
                                response.context_menu(|ui| {
                                    if ui.button("Rename").clicked() {
                                        self.rename_scene =
                                            Some((scene_id.clone(), scene_name.clone()));
                                        ui.close();
                                    }
                                    if ui
                                        .add_enabled(
                                            self.scenes.scenes.len() > 1,
                                            egui::Button::new("Delete"),
                                        )
                                        .clicked()
                                    {
                                        self.delete_scene = Some(scene_id.clone());
                                        ui.close();
                                    }
                                });
                            }
                            if active != self.scenes.active_scene_id {
                                self.scenes.active_scene_id = active;
                                self.save_scenes();
                            }
                        },
                    );
                    if columns[0].button("+ Add scene").clicked() {
                        let scene = arca_core::scenes::Scene {
                            id: arca_core::scenes::make_id("scene"),
                            name: next_scene_name(&self.scenes),
                            sources: Vec::new(),
                        };
                        self.scenes.active_scene_id = scene.id.clone();
                        self.scenes.scenes.push(scene);
                        self.save_scenes();
                    }

                    columns[1].heading("Sources");
                    columns[1].separator();
                    let active_sources = self
                        .scenes
                        .scenes
                        .iter()
                        .find(|s| s.id == self.scenes.active_scene_id)
                        .map(|s| s.sources.clone())
                        .unwrap_or_default();
                    let mut add_capture_kind = None;
                    let mut bind_capture_source = None;
                    if active_sources.is_empty() {
                        columns[1].vertical_centered(|ui| {
                            ui.add_space(22.0);
                            ui.label("No sources in this scene");
                            ui.small("Add a display or window to capture.");
                        });
                    } else {
                        let mut visibility_change = None;
                        let mut remove_source = None;
                        for source in active_sources {
                            columns[1].horizontal(|ui| {
                                ui.label(&source.name);
                                if ui
                                    .small_button(if source.capture_target_id.is_some() {
                                        "Change target"
                                    } else {
                                        "Select target"
                                    })
                                    .clicked()
                                {
                                    bind_capture_source = Some((source.id.clone(), source.kind));
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("Remove").clicked() {
                                            remove_source = Some(source.id.clone());
                                        }
                                        if ui
                                            .small_button(if source.visible {
                                                "Hide"
                                            } else {
                                                "Show"
                                            })
                                            .clicked()
                                        {
                                            visibility_change =
                                                Some((source.id.clone(), !source.visible));
                                        }
                                    },
                                );
                            });
                        }
                        if let Some(source_id) = remove_source {
                            if let Some(scene) = self
                                .scenes
                                .scenes
                                .iter_mut()
                                .find(|scene| scene.id == self.scenes.active_scene_id)
                            {
                                scene.sources.retain(|source| source.id != source_id);
                            }
                            self.save_scenes();
                        } else if let Some((source_id, visible)) = visibility_change {
                            if let Some(source) = self
                                .scenes
                                .scenes
                                .iter_mut()
                                .find(|scene| scene.id == self.scenes.active_scene_id)
                                .and_then(|scene| {
                                    scene
                                        .sources
                                        .iter_mut()
                                        .find(|source| source.id == source_id)
                                })
                            {
                                source.visible = visible;
                            }
                            self.save_scenes();
                        }
                    }
                    columns[1].horizontal(|ui| {
                        if ui.button("+ Display").clicked() {
                            add_capture_kind = Some(arca_core::scenes::SourceKind::Display);
                        }
                        if ui.button("+ Window or game").clicked() {
                            add_capture_kind = Some(arca_core::scenes::SourceKind::Window);
                        }
                    });
                    if let Some(kind) = add_capture_kind {
                        self.capture_picker = Some(kind);
                        self.capture_picker_source = None;
                        self.capture_targets = match arca_capture::windows::enumerate_targets() {
                            Ok(targets) => targets,
                            Err(error) => {
                                self.error = Some(error);
                                Vec::new()
                            }
                        };
                    }
                    if let Some((source_id, kind)) = bind_capture_source {
                        self.capture_picker = Some(kind);
                        self.capture_picker_source = Some(source_id);
                        self.capture_targets = match arca_capture::windows::enumerate_targets() {
                            Ok(targets) => targets,
                            Err(error) => {
                                self.error = Some(error);
                                Vec::new()
                            }
                        };
                    }

                    columns[2].heading("Audio");
                    columns[2].separator();
                    columns[2].label("Selected inputs are mixed into the stream when you go live.");
                    let mut add_audio_kind = None;
                    columns[2].horizontal(|ui| {
                        if ui.button("+ Mic/Aux").clicked() {
                            add_audio_kind = Some(arca_audio::AudioSourceKind::Microphone);
                        }
                        if ui.button("+ Desktop").clicked() {
                            add_audio_kind = Some(arca_audio::AudioSourceKind::Desktop);
                        }
                        if ui.button("+ App").clicked() {
                            add_audio_kind = Some(arca_audio::AudioSourceKind::Application);
                        }
                    });
                    if let Some(kind) = add_audio_kind {
                        self.audio_picker = Some(kind);
                        let sources = match kind {
                            arca_audio::AudioSourceKind::Microphone => {
                                arca_audio::wasapi::enumerate_microphones()
                            }
                            arca_audio::AudioSourceKind::Desktop => {
                                arca_audio::wasapi::enumerate_desktop_audio()
                            }
                            arca_audio::AudioSourceKind::Application => {
                                arca_audio::wasapi::enumerate_active_applications()
                            }
                        };
                        self.audio_candidates = match sources {
                            Ok(sources) => sources,
                            Err(error) => {
                                self.error = Some(error.to_string());
                                Vec::new()
                            }
                        };
                    }
                    let levels = self
                        .audio_meter
                        .as_ref()
                        .map(|meter| meter.snapshot())
                        .unwrap_or_default();
                    let mut remove_audio = None;
                    for source in &self.audio_tracks {
                        columns[2].horizontal(|ui| {
                            ui.label(&source.label);
                            if let Some(level) = levels.iter().find(|level| level.id == source.id) {
                                let peak = level.peak.unwrap_or_default().clamp(0.0, 1.0) as f32;
                                ui.add(
                                    egui::ProgressBar::new(peak)
                                        .desired_width(100.0)
                                        .show_percentage(),
                                );
                            }
                            if ui.small_button("Remove").clicked() {
                                remove_audio = Some(source.id.clone());
                            }
                        });
                    }
                    if let Some(id) = remove_audio {
                        self.audio_tracks.retain(|source| source.id != id);
                        self.audio_meter = arca_audio::wasapi::start_meter(&self.audio_tracks).ok();
                    }

                    columns[3].vertical_centered(|ui| {
                        ui.add_space(35.0);
                        #[cfg(windows)]
                        if let Some(stream) = &self.video_stream {
                            ui.label(egui::RichText::new(stream.status()).color(
                                if stream.is_failed() {
                                    egui::Color32::from_rgb(0xFF, 0x65, 0x65)
                                } else {
                                    LIME
                                },
                            ));
                            if ui
                                .button(if stream.is_failed() {
                                    "Dismiss"
                                } else {
                                    "Stop stream"
                                })
                                .clicked()
                            {
                                stop_stream = true;
                            }
                        } else {
                            let ready = self.hardware_encoder.is_ok();
                            if ui
                                .add_enabled(
                                    ready,
                                    egui::Button::new("Go Live")
                                        .fill(LIME)
                                        .min_size(egui::vec2(150.0, 42.0)),
                                )
                                .clicked()
                            {
                                start_stream = true;
                            }
                            if let Err(error) = &self.hardware_encoder {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new(error).small().color(MUTED));
                            }
                        }
                        #[cfg(not(windows))]
                        {
                            ui.add_enabled(
                                false,
                                egui::Button::new("Go Live").min_size(egui::vec2(150.0, 42.0)),
                            );
                            ui.label("Live publishing is supported on Windows.");
                        }
                    });
                });
            },
        );
        #[cfg(windows)]
        {
            if stop_stream {
                if let Some(stream) = &self.video_stream {
                    stream.stop();
                }
                self.video_stream = None;
                self.stream_capture_identity = None;
            }
            if start_stream {
                if let Err(error) = self.start_video_stream() {
                    self.error = Some(error);
                }
            }
        }
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for (tab, label) in [(SettingsTab::General,"General"),(SettingsTab::Stream,"Stream"),(SettingsTab::Video,"Video"),(SettingsTab::Audio,"Audio"),(SettingsTab::Hotkeys,"Hotkeys")] {
                    if ui.selectable_label(self.tab == tab, label).clicked() { self.tab = tab; }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.heading(match self.tab { SettingsTab::General => "General", SettingsTab::Stream => "Stream", SettingsTab::Video => "Video", SettingsTab::Audio => "Audio", SettingsTab::Hotkeys => "Hotkeys" });
                ui.add_space(12.0);
                match self.tab {
                    SettingsTab::General => {
                        ui.label("Settings are stored on this PC. Crash logs stay local.");
                        ui.checkbox(&mut self.config.general.start_minimized, "Start minimized");
                        ui.checkbox(&mut self.config.general.confirm_before_stop, "Confirm before stopping a stream");
                        if ui.button("Save settings").clicked() { if let Err(e) = self.config.save() { self.error = Some(e); } }
                    }
                    SettingsTab::Stream => {
                        ui.label("Connect with an RTMPS ingest URL and stream key only.");
                        egui::ComboBox::from_label("Platform").selected_text(match self.config.platform {
                            arca_core::config::Platform::Youtube => "YouTube",
                            arca_core::config::Platform::Kick => "Kick",
                            arca_core::config::Platform::Twitch => "Twitch",
                        }).show_ui(ui, |ui| {
                            for (platform, label) in [
                                (arca_core::config::Platform::Youtube, "YouTube"),
                                (arca_core::config::Platform::Kick, "Kick"),
                                (arca_core::config::Platform::Twitch, "Twitch"),
                            ] { ui.selectable_value(&mut self.config.platform, platform, label); }
                        });
                        let platform = self.config.platform;
                        ui.label("RTMPS ingest URL");
                        ui.text_edit_singleline(self.config.ingest_urls.get_mut(platform));
                        ui.label("Stream key");
                        ui.label(if self.key_saved { "A key is saved in the Windows credential vault." } else { "No key is saved for this platform." });
                        ui.add(egui::TextEdit::singleline(&mut self.stream_key).password(true));
                        ui.horizontal(|ui| {
                            if ui.button("Save stream settings").clicked() {
                                if let Err(error) = arca_publish::validate_ingest_url(self.config.ingest_urls.get(platform)) {
                                    self.error = Some(error.to_string());
                                } else {
                                    match arca_core::keychain::set_stream_key(platform, &self.stream_key) {
                                        Ok(()) => {
                                            self.stream_key.clear();
                                            self.key_saved = true;
                                            if let Err(error) = self.config.save() { self.error = Some(error); }
                                        }
                                        Err(error) => self.error = Some(error),
                                    }
                                }
                            }
                            if ui.button("Clear saved key").clicked() {
                                if let Err(error) = arca_core::keychain::clear_stream_key(platform) { self.error = Some(error); } else { self.key_saved = false; }
                            }
                        });
                        ui.small("The key is stored in the Windows credential vault and is never written to settings or logs.");
                    }
                    SettingsTab::Video => {
                        ui.label("Hardware encoding is required; there is no software fallback.");
                        ui.label(match &self.hardware_encoder {
                            Ok(encoder) => format!("Hardware H.264: {} detected", match encoder {
                                arca_encode::HardwareEncoder::Nvidia => "NVIDIA NVENC",
                                arca_encode::HardwareEncoder::Amd => "AMD AMF",
                                arca_encode::HardwareEncoder::Intel => "Intel Quick Sync",
                                arca_encode::HardwareEncoder::Other => "Media Foundation hardware encoder",
                            }),
                            Err(error) => format!("Hardware H.264 unavailable: {error}"),
                        });
                        egui::ComboBox::from_label("Output size").selected_text(&self.config.video.resolution).show_ui(ui, |ui| {
                            for size in ["1920x1080", "1600x900", "1280x720"] {
                                ui.selectable_value(&mut self.config.video.resolution, size.into(), size);
                            }
                        });
                        egui::ComboBox::from_label("Frame rate").selected_text(format!("{} fps", self.config.video.frame_rate)).show_ui(ui, |ui| {
                            for fps in [30, 60] { ui.selectable_value(&mut self.config.video.frame_rate, fps, format!("{fps} fps")); }
                        });
                        ui.add(egui::Slider::new(&mut self.config.video.bitrate_kbps, 1000..=12000).text("Bitrate (kb/s)"));
                        if ui.button("Save video settings").clicked() { if let Err(error) = self.config.save() { self.error = Some(error); } }
                    }
                    SettingsTab::Audio => { ui.label("WASAPI microphone, desktop, and active-application audio."); ui.label("Application audio requires Windows build 20348 or later."); }
                    SettingsTab::Hotkeys => { ui.label("Hotkeys are not implemented yet."); }
                }
            });
        });
    }
}

impl eframe::App for ArcaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Self::theme(ui.ctx());
        self.header(ui);
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                match self.screen {
                    Screen::Studio => self.studio(ui, _frame),
                    Screen::Settings => {
                        self.hide_native_preview();
                        self.settings(ui)
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(self.stream_status());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(if self.stream_active() {
                            "Stream is active."
                        } else {
                            "Offline · no stream is being sent."
                        });
                    });
                });
            },
        );
        if let Some(error) = self.error.clone() {
            egui::Window::new("ArcaLabs")
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.label(&error);
                    if ui.button("Dismiss").clicked() {
                        self.error = None;
                    }
                });
        }
        if let Some((scene_id, draft)) = self.rename_scene.clone() {
            let mut draft = draft;
            let mut save = false;
            let mut cancel = false;
            egui::Window::new("Rename scene")
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.text_edit_singleline(&mut draft);
                    ui.horizontal(|ui| {
                        save = ui.button("Save").clicked();
                        cancel = ui.button("Cancel").clicked();
                    });
                });
            if save {
                if let Some(scene) = self
                    .scenes
                    .scenes
                    .iter_mut()
                    .find(|scene| scene.id == scene_id)
                {
                    scene.name = draft.clone();
                }
                self.save_scenes();
                if self.error.is_none() {
                    self.rename_scene = None;
                } else if let Some(scene) = self
                    .scenes
                    .scenes
                    .iter_mut()
                    .find(|scene| scene.id == scene_id)
                {
                    scene.name = self
                        .rename_scene
                        .as_ref()
                        .map(|(_, name)| name.clone())
                        .unwrap_or_default();
                }
            } else if cancel {
                self.rename_scene = None;
            } else {
                self.rename_scene = Some((scene_id, draft));
            }
        }
        if let Some(kind) = self.capture_picker {
            let mut selected = None;
            let mut cancel = false;
            let heading = match kind {
                arca_core::scenes::SourceKind::Display => "Choose a display",
                arca_core::scenes::SourceKind::Window => "Choose a window or game",
            };
            egui::Window::new(heading)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    let targets: Vec<_> = self
                        .capture_targets
                        .iter()
                        .filter(|target| match kind {
                            arca_core::scenes::SourceKind::Display => {
                                target.id.starts_with("display:")
                            }
                            arca_core::scenes::SourceKind::Window => {
                                target.id.starts_with("window:")
                            }
                        })
                        .cloned()
                        .collect();
                    if targets.is_empty() {
                        ui.label("No available targets were found. Open a window, then try again.");
                    }
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for target in targets {
                                let label = format!(
                                    "{}  ·  {} × {}",
                                    target.label, target.width, target.height
                                );
                                if ui.selectable_label(false, label).clicked() {
                                    selected = Some(target);
                                }
                            }
                        });
                    cancel = ui.button("Cancel").clicked();
                });
            if let Some(target) = selected {
                if let Some(scene) = self
                    .scenes
                    .scenes
                    .iter_mut()
                    .find(|scene| scene.id == self.scenes.active_scene_id)
                {
                    if let Some(source_id) = self.capture_picker_source.take() {
                        if let Some(source) = scene.sources.iter_mut().find(|s| s.id == source_id) {
                            source.name = target.label;
                            source.kind = kind;
                            source.capture_target_id = Some(target.id);
                        }
                    } else {
                        scene.sources.push(arca_core::scenes::SceneSource {
                            id: arca_core::scenes::make_id("source"),
                            name: target.label,
                            kind,
                            capture_target_id: Some(target.id),
                            visible: true,
                            x: 0.0,
                            y: 0.0,
                            width: 1.0,
                            height: 1.0,
                        });
                    }
                }
                self.save_scenes();
                self.capture_source_id = None;
                self.capture_picker = None;
                self.capture_picker_source = None;
                self.capture_targets.clear();
            } else if cancel {
                self.capture_picker = None;
                self.capture_picker_source = None;
                self.capture_targets.clear();
            }
        }
        if let Some(kind) = self.audio_picker {
            let mut selected = None;
            let mut cancel = false;
            let heading = match kind {
                arca_audio::AudioSourceKind::Microphone => "Choose a microphone",
                arca_audio::AudioSourceKind::Desktop => "Choose desktop audio",
                arca_audio::AudioSourceKind::Application => "Choose an active application",
            };
            egui::Window::new(heading)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    if self.audio_candidates.is_empty() {
                        ui.label("No matching audio sources are active. Check the device or play audio, then retry.");
                    }
                    for source in &self.audio_candidates {
                        if ui.selectable_label(false, &source.label).clicked() {
                            selected = Some(source.clone());
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        cancel = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.audio_picker = None;
                    }
                });
            if let Some(source) = selected {
                if self.audio_tracks.iter().any(|track| track.id == source.id) {
                    self.error = Some("That audio source is already in the mixer.".into());
                } else {
                    self.audio_tracks.push(source);
                    match arca_audio::wasapi::start_meter(&self.audio_tracks) {
                        Ok(meter) => self.audio_meter = Some(meter),
                        Err(error) => self.error = Some(error.to_string()),
                    }
                }
                self.audio_picker = None;
                self.audio_candidates.clear();
            } else if cancel {
                let sources = match kind {
                    arca_audio::AudioSourceKind::Microphone => {
                        arca_audio::wasapi::enumerate_microphones()
                    }
                    arca_audio::AudioSourceKind::Desktop => {
                        arca_audio::wasapi::enumerate_desktop_audio()
                    }
                    arca_audio::AudioSourceKind::Application => {
                        arca_audio::wasapi::enumerate_active_applications()
                    }
                };
                match sources {
                    Ok(sources) => self.audio_candidates = sources,
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
        }
        if let Some(scene_id) = self.delete_scene.clone() {
            let scene_name = self
                .scenes
                .scenes
                .iter()
                .find(|scene| scene.id == scene_id)
                .map(|scene| scene.name.clone())
                .unwrap_or_default();
            let mut remove = false;
            let mut cancel = false;
            egui::Window::new("Delete scene?")
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Remove “{scene_name}” and its source list?"));
                    ui.horizontal(|ui| {
                        remove = ui.button("Delete").clicked();
                        cancel = ui.button("Cancel").clicked();
                    });
                });
            if remove && self.scenes.scenes.len() > 1 {
                let previous = self.scenes.clone();
                self.scenes.scenes.retain(|scene| scene.id != scene_id);
                if self.scenes.active_scene_id == scene_id {
                    self.scenes.active_scene_id = self.scenes.scenes[0].id.clone();
                }
                self.save_scenes();
                if self.error.is_some() {
                    self.scenes = previous;
                } else {
                    self.delete_scene = None;
                }
            } else if cancel {
                self.delete_scene = None;
            }
        }
    }
}

fn next_scene_name(scenes: &SceneCollection) -> String {
    let mut index = scenes.scenes.len() + 1;
    loop {
        let name = format!("Scene {index}");
        if !scenes
            .scenes
            .iter()
            .any(|scene| scene.name.eq_ignore_ascii_case(&name))
        {
            return name;
        }
        index += 1;
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1372.0, 882.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ArcaLabs",
        options,
        Box::new(|_| Ok(Box::new(ArcaApp::new()))),
    )
}
