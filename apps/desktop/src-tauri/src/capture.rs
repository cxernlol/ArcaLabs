use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, thread, time::Duration};
use tauri::{AppHandle, State, WebviewWindow};

const PREVIEW_WIDTH: u32 = 1280;
const PREVIEW_HEIGHT: u32 = 720;

#[derive(Default)]
pub struct CaptureRuntime {
    #[cfg(windows)]
    window_items: Mutex<HashMap<String, windows::Graphics::Capture::GraphicsCaptureItem>>,
    #[cfg(windows)]
    preview: Mutex<Option<PreviewWorker>>,
}

impl Drop for CaptureRuntime {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Ok(worker) = self.preview.get_mut() {
            let mut worker = worker.take();
            stop_worker(&mut worker);
        }
    }
}

#[cfg(windows)]
struct PreviewWorker {
    scene_id: String,
    source_signature: String,
    stop: Arc<AtomicBool>,
    frame: Arc<Mutex<Option<Vec<u8>>>>,
    frame_updated: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
    thread: Option<thread::JoinHandle<()>>,
}

#[cfg(windows)]
enum PreviewTarget {
    Window(windows::Graphics::Capture::GraphicsCaptureItem),
    Display(String),
}

#[tauri::command]
pub fn get_preview_frame(
    app: AppHandle,
    store: State<'_, crate::scenes::SceneStore>,
    runtime: State<'_, CaptureRuntime>,
    window: WebviewWindow,
    scene_id: String,
) -> Result<tauri::ipc::Response, String> {
    #[cfg(windows)]
    {
        use crate::scenes::SourceKind;

        if window
            .is_minimized()
            .map_err(|cause| format!("Could not read window state: {cause}"))?
            || !window
                .is_visible()
                .map_err(|cause| format!("Could not read window state: {cause}"))?
        {
            stop_preview_worker(&runtime)?;
            return Ok(tauri::ipc::Response::new(Vec::<u8>::new()));
        }

        let collection = store.load(&app)?;
        let scene = collection
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .ok_or_else(|| "The selected scene no longer exists".to_owned())?;
        let sources = scene
            .sources
            .iter()
            .filter(|source| source.visible)
            .cloned()
            .collect::<Vec<_>>();
        if sources.is_empty() {
            stop_preview_worker(&runtime)?;
            return Ok(tauri::ipc::Response::new(Vec::<u8>::new()));
        }
        let source_signature = sources
            .iter()
            .map(|source| {
                format!(
                    "{}:{}:{}:{}:{}",
                    source.id, source.x, source.y, source.width, source.height
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let mut targets = Vec::with_capacity(sources.len());
        let window_items = runtime
            .window_items
            .lock()
            .map_err(|_| "Window capture state is unavailable".to_owned())?;
        for source in sources {
            let target = match source.kind {
                SourceKind::Window => PreviewTarget::Window(
                    window_items.get(&source.id).cloned().ok_or_else(|| {
                        format!(
                            "Select the window for {} to start its preview.",
                            source.name
                        )
                    })?,
                ),
                SourceKind::Display => PreviewTarget::Display(
                    source
                        .display_id
                        .clone()
                        .ok_or_else(|| "The display source has no saved display ID".to_owned())?,
                ),
            };
            targets.push((source, target));
        }
        drop(window_items);

        let mut worker = runtime
            .preview
            .lock()
            .map_err(|_| "Preview state is unavailable".to_owned())?;
        let same_target = worker.as_ref().is_some_and(|current| {
            current.scene_id == scene_id && current.source_signature == source_signature
        });
        if !same_target {
            stop_worker(&mut worker);
            let stop = Arc::new(AtomicBool::new(false));
            let frame = Arc::new(Mutex::new(None));
            let frame_updated = Arc::new(AtomicBool::new(false));
            let error = Arc::new(Mutex::new(None));
            let worker_stop = Arc::clone(&stop);
            let worker_frame = Arc::clone(&frame);
            let worker_frame_updated = Arc::clone(&frame_updated);
            let worker_error = Arc::clone(&error);
            let capture_thread = thread::Builder::new()
                .name("arcalabs-preview".into())
                .spawn(move || {
                    if let Err(cause) = capture_preview(
                        targets,
                        worker_stop,
                        worker_frame,
                        worker_frame_updated,
                        worker_error.clone(),
                    ) {
                        if let Ok(mut slot) = worker_error.lock() {
                            *slot = Some(cause);
                        }
                    }
                })
                .map_err(|cause| format!("Could not start preview capture: {cause}"))?;
            *worker = Some(PreviewWorker {
                scene_id: scene_id.clone(),
                source_signature,
                stop,
                frame,
                frame_updated,
                error,
                thread: Some(capture_thread),
            });
        }

        let current = worker
            .as_ref()
            .ok_or_else(|| "Preview capture did not start".to_owned())?;
        if let Some(cause) = current
            .error
            .lock()
            .map_err(|_| "Preview error state is unavailable".to_owned())?
            .clone()
        {
            return Err(cause);
        }
        let bytes = if current.frame_updated.swap(false, Ordering::AcqRel) {
            current
                .frame
                .lock()
                .map_err(|_| "Preview frame is unavailable".to_owned())?
                .clone()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(tauri::ipc::Response::new(bytes))
    }

    #[cfg(not(windows))]
    {
        let _ = (app, store, runtime, window, scene_id);
        Ok(tauri::ipc::Response::new(Vec::<u8>::new()))
    }
}

#[tauri::command]
pub fn stop_preview(
    runtime: State<'_, CaptureRuntime>,
    scene_id: Option<String>,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut worker = runtime
            .preview
            .lock()
            .map_err(|_| "Preview state is unavailable".to_owned())?;
        if scene_id.as_ref().is_none_or(|id| {
            worker
                .as_ref()
                .is_some_and(|current| &current.scene_id == id)
        }) {
            stop_worker(&mut worker);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (runtime, scene_id);
        Ok(())
    }
}

#[cfg(windows)]
fn stop_preview_worker(runtime: &CaptureRuntime) -> Result<(), String> {
    let mut worker = runtime
        .preview
        .lock()
        .map_err(|_| "Preview state is unavailable".to_owned())?;
    stop_worker(&mut worker);
    Ok(())
}

#[cfg(windows)]
fn stop_worker(worker: &mut Option<PreviewWorker>) {
    if let Some(mut current) = worker.take() {
        current.stop.store(true, Ordering::Release);
        if let Some(thread) = current.thread.take() {
            let _ = thread.join();
        }
    }
}

#[tauri::command]
pub async fn add_window_source(
    app: AppHandle,
    store: State<'_, crate::scenes::SceneStore>,
    runtime: State<'_, CaptureRuntime>,
    window: WebviewWindow,
    scene_id: String,
) -> Result<Option<crate::scenes::SceneCollection>, String> {
    #[cfg(windows)]
    {
        use crate::scenes::{SourceKind, add_source};
        let Some(item) = pick_window_target(window).await? else {
            return Ok(None);
        };

        let previous = store.load(&app)?;
        let next = add_source(
            &app,
            &store,
            scene_id.clone(),
            "Window capture".into(),
            SourceKind::Window,
            None,
        )?;
        let old_ids = previous
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .map(|scene| {
                scene
                    .sources
                    .iter()
                    .map(|source| source.id.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let source_id = next
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .and_then(|scene| {
                scene
                    .sources
                    .iter()
                    .find(|source| !old_ids.contains(&source.id.as_str()))
            })
            .map(|source| source.id.clone())
            .ok_or_else(|| "The selected window could not be added to this scene".to_owned())?;
        runtime
            .window_items
            .lock()
            .map_err(|_| "Window capture state is unavailable".to_owned())?
            .insert(source_id, item);
        Ok(Some(next))
    }

    #[cfg(not(windows))]
    {
        let _ = (app, store, runtime, window, scene_id);
        Err("Window capture is available on Windows only.".into())
    }
}

#[tauri::command]
pub async fn reselect_window_source(
    app: AppHandle,
    store: State<'_, crate::scenes::SceneStore>,
    runtime: State<'_, CaptureRuntime>,
    window: WebviewWindow,
    scene_id: String,
    source_id: String,
) -> Result<Option<crate::scenes::SceneCollection>, String> {
    let collection = store.load(&app)?;
    let is_window_source = collection
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .and_then(|scene| scene.sources.iter().find(|source| source.id == source_id))
        .is_some_and(|source| source.kind == crate::scenes::SourceKind::Window);
    if !is_window_source {
        return Err("Window source no longer exists".into());
    }

    #[cfg(windows)]
    {
        let Some(item) = pick_window_target(window).await? else {
            return Ok(None);
        };
        runtime
            .window_items
            .lock()
            .map_err(|_| "Window capture state is unavailable".to_owned())?
            .insert(source_id, item);
        Ok(Some(collection))
    }

    #[cfg(not(windows))]
    {
        let _ = (runtime, window);
        Err("Window capture is available on Windows only.".into())
    }
}

#[cfg(windows)]
async fn pick_window_target(
    window: WebviewWindow,
) -> Result<Option<windows::Graphics::Capture::GraphicsCaptureItem>, String> {
    use windows::Graphics::Capture::GraphicsCapturePicker;
    use windows::Win32::UI::Shell::IInitializeWithWindow;
    use windows::core::Interface;

    let picker_window = window.clone();
    let (send_operation, receive_operation) = mpsc::sync_channel(1);
    window
        .run_on_main_thread(move || {
            let operation = (|| {
                let picker = GraphicsCapturePicker::new().map_err(|error| {
                    format!("Could not open the Windows capture picker: {error}")
                })?;
                let initialize: IInitializeWithWindow = picker.cast().map_err(|error| {
                    format!("Could not attach the capture picker to Arcalabs: {error}")
                })?;
                let hwnd = picker_window
                    .hwnd()
                    .map_err(|error| format!("Arcalabs window is unavailable: {error}"))?;
                let hwnd = windows::Win32::Foundation::HWND(hwnd.0);
                unsafe { initialize.Initialize(hwnd) }.map_err(|error| {
                    format!("Could not attach the capture picker to Arcalabs: {error}")
                })?;
                picker
                    .PickSingleItemAsync()
                    .map_err(|error| format!("Could not start the Windows capture picker: {error}"))
            })();
            let _ = send_operation.send(operation);
        })
        .map_err(|error| format!("Could not open the Windows capture picker: {error}"))?;
    let operation = receive_operation
        .recv()
        .map_err(|_| "Windows capture picker did not respond".to_owned())??;
    match operation.await {
        Ok(item) => Ok(Some(item)),
        Err(error) if error.code().0 as u32 == 0x80004004 => Ok(None),
        Err(error) => Err(format!(
            "Windows could not select a capture target: {error}"
        )),
    }
}

#[cfg(not(windows))]
async fn pick_window_target(_: WebviewWindow) -> Result<Option<()>, String> {
    Err("Window capture is available on Windows only.".into())
}

#[cfg(windows)]
fn capture_preview(
    targets: Vec<(crate::scenes::SceneSource, PreviewTarget)>,
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<Vec<u8>>>>,
    frame_updated: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
) -> Result<(), String> {
    use windows::Graphics::Capture::{Direct3D11CaptureFrame, Direct3D11CaptureFramePool};
    use windows::Graphics::DirectX::DirectXPixelFormat;
    use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice,
    };
    use windows::Win32::Graphics::Dxgi::IDXGIDevice;
    use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
    use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
    use windows::core::Interface;

    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }
        .map_err(|cause| format!("Could not initialize Windows capture: {cause}"))?;
    let _apartment = WinRtApartment;

    let mut device = None;
    let mut context = None;
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            windows::Win32::Foundation::HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    }
    .map_err(|cause| format!("Could not create the preview graphics device: {cause}"))?;
    let device = device.ok_or_else(|| "Preview graphics device is unavailable".to_owned())?;
    let context = context.ok_or_else(|| "Preview graphics context is unavailable".to_owned())?;
    let dxgi_device: IDXGIDevice = device.cast().map_err(|cause| cause.to_string())?;
    let rt_device = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device) }
        .map_err(|cause| format!("Could not connect Windows capture to Direct3D: {cause}"))?
        .cast::<windows::Graphics::DirectX::Direct3D11::IDirect3DDevice>()
        .map_err(|cause| format!("Could not connect Windows capture to Direct3D: {cause}"))?;
    let imaging = create_wic_factory()?;
    let mut captures = Vec::with_capacity(targets.len());
    for (source, target) in targets {
        let item = match target {
            PreviewTarget::Window(item) => item,
            PreviewTarget::Display(display_id) => {
                let output = display_capture_output(&display_id)?;
                let output1: windows::Win32::Graphics::Dxgi::IDXGIOutput1 = output
                    .cast()
                    .map_err(|cause| format!("This display does not support capture: {cause}"))?;
                let duplication = unsafe { output1.DuplicateOutput(&device) }
                    .map_err(|cause| format!("Could not start display capture: {cause}"))?;
                captures.push(ActiveCaptureSource {
                    source,
                    backend: PreviewCaptureBackend::Display(duplication),
                    staging: None,
                    latest: None,
                });
                continue;
            }
        };
        let size = item
            .Size()
            .map_err(|cause| format!("Could not read capture size: {cause}"))?;
        if size.Width <= 0 || size.Height <= 0 {
            return Err(format!("{} has no visible capture area", source.name));
        }
        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &rt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )
        .map_err(|cause| {
            format!(
                "Could not create preview capture for {}: {cause}",
                source.name
            )
        })?;
        let session = pool.CreateCaptureSession(&item).map_err(|cause| {
            format!(
                "Could not create capture session for {}: {cause}",
                source.name
            )
        })?;
        session
            .StartCapture()
            .map_err(|cause| format!("Could not start capture for {}: {cause}", source.name))?;
        captures.push(ActiveCaptureSource {
            source,
            backend: PreviewCaptureBackend::Window { pool, session },
            staging: None,
            latest: None,
        });
    }

    while !stop.load(Ordering::Acquire) {
        let mut failed = None;
        let mut has_new_capture_frame = false;
        for capture in &mut captures {
            match &capture.backend {
                PreviewCaptureBackend::Window { pool, .. } => {
                    let mut newest: Option<Direct3D11CaptureFrame> = None;
                    while let Ok(frame) = pool.TryGetNextFrame() {
                        if let Some(previous) = newest.replace(frame) {
                            previous.Close().ok();
                        }
                    }
                    if let Some(frame) = newest {
                        let result = frame.Surface()
                            .map_err(|cause| format!("Could not access {}: {cause}", capture.source.name))
                            .and_then(|surface| {
                                use windows::core::Interface;
                                let access: windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess = surface
                                    .cast().map_err(|cause| cause.to_string())?;
                                let texture: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D = unsafe { access.GetInterface() }
                                    .map_err(|cause| cause.to_string())?;
                                read_texture_region(&texture, &device, &context, &mut capture.staging, &capture.source)
                            });
                        frame.Close().ok();
                        match result {
                            Ok(pixels) => {
                                capture.latest = Some(pixels);
                                has_new_capture_frame = true;
                            }
                            Err(cause) => failed = Some(cause),
                        }
                    }
                }
                PreviewCaptureBackend::Display(duplication) => {
                    let mut frame_info =
                        windows::Win32::Graphics::Dxgi::DXGI_OUTDUPL_FRAME_INFO::default();
                    let mut resource = None;
                    match unsafe { duplication.AcquireNextFrame(0, &mut frame_info, &mut resource) }
                    {
                        Ok(()) => {
                            let result = resource.ok_or_else(|| "Display capture returned no frame".to_owned())
                                .and_then(|resource| {
                                    use windows::core::Interface;
                                    let texture: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D = resource
                                        .cast().map_err(|cause| cause.to_string())?;
                                    read_texture_region(&texture, &device, &context, &mut capture.staging, &capture.source)
                                });
                            unsafe { duplication.ReleaseFrame() }.ok();
                            match result {
                                Ok(pixels) => {
                                    capture.latest = Some(pixels);
                                    has_new_capture_frame = true;
                                }
                                Err(cause) => failed = Some(cause),
                            }
                        }
                        Err(cause) if cause.code().0 as u32 == 0x887A0027 => {}
                        Err(cause) => failed = Some(format!("Display capture stopped: {cause}")),
                    }
                }
            }
            if failed.is_some() {
                break;
            }
        }
        if let Some(cause) = failed {
            if let Ok(mut slot) = error.lock() {
                *slot = Some(cause);
            }
            stop.store(true, Ordering::Release);
            continue;
        }
        if has_new_capture_frame {
            let composed = compose_scene(&captures);
            let scaled = scale_bgra_to_bgr(
                &composed,
                PREVIEW_WIDTH as usize * 4,
                PREVIEW_WIDTH,
                PREVIEW_HEIGHT,
            );
            match encode_jpeg(&imaging, &scaled.0, scaled.1, scaled.2) {
                Ok(bytes) => {
                    if let Ok(mut slot) = latest.lock() {
                        *slot = Some(bytes);
                        frame_updated.store(true, Ordering::Release);
                    }
                }
                Err(cause) => {
                    if let Ok(mut slot) = error.lock() {
                        *slot = Some(cause);
                    }
                    stop.store(true, Ordering::Release);
                }
            }
        }
        thread::sleep(Duration::from_millis(67));
    }
    for capture in captures {
        if let PreviewCaptureBackend::Window { pool, session } = capture.backend {
            session.Close().ok();
            pool.Close().ok();
        }
    }
    Ok(())
}

#[cfg(windows)]
struct ActiveCaptureSource {
    source: crate::scenes::SceneSource,
    backend: PreviewCaptureBackend,
    staging: Option<(
        u32,
        u32,
        windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
    )>,
    latest: Option<Vec<u8>>,
}

#[cfg(windows)]
enum PreviewCaptureBackend {
    Window {
        pool: windows::Graphics::Capture::Direct3D11CaptureFramePool,
        session: windows::Graphics::Capture::GraphicsCaptureSession,
    },
    Display(windows::Win32::Graphics::Dxgi::IDXGIOutputDuplication),
}

#[cfg(windows)]
fn read_texture_region(
    texture: &windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
    device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
    context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    staging: &mut Option<(
        u32,
        u32,
        windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
    )>,
    source: &crate::scenes::SceneSource,
) -> Result<Vec<u8>, String> {
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_STAGING,
    };
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { texture.GetDesc(&mut desc) };
    if staging
        .as_ref()
        .is_none_or(|(width, height, _)| *width != desc.Width || *height != desc.Height)
    {
        let copy_desc = D3D11_TEXTURE2D_DESC {
            Width: desc.Width,
            Height: desc.Height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut new_staging = None;
        unsafe { device.CreateTexture2D(&copy_desc, None, Some(&mut new_staging)) }
            .map_err(|cause| format!("Could not prepare preview frame: {cause}"))?;
        *staging = Some((
            desc.Width,
            desc.Height,
            new_staging.ok_or_else(|| "Preview frame buffer is unavailable".to_owned())?,
        ));
    }
    let staging_texture = &staging
        .as_ref()
        .ok_or_else(|| "Preview frame buffer is unavailable".to_owned())?
        .2;
    unsafe { context.CopyResource(staging_texture, texture) };
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe { context.Map(staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) }
        .map_err(|cause| format!("Could not read preview frame: {cause}"))?;
    let out_width = (source.width * PREVIEW_WIDTH as f32)
        .round()
        .clamp(1.0, PREVIEW_WIDTH as f32) as u32;
    let out_height = (source.height * PREVIEW_HEIGHT as f32)
        .round()
        .clamp(1.0, PREVIEW_HEIGHT as f32) as u32;
    let output = unsafe {
        let pixels = std::slice::from_raw_parts(
            mapped.pData.cast::<u8>(),
            mapped.RowPitch as usize * desc.Height as usize,
        );
        scale_bgra(
            pixels,
            mapped.RowPitch as usize,
            desc.Width,
            desc.Height,
            out_width,
            out_height,
        )
    };
    unsafe { context.Unmap(staging_texture, 0) };
    Ok(output)
}

#[cfg(windows)]
fn scale_bgra(
    source: &[u8],
    row_pitch: usize,
    width: u32,
    height: u32,
    out_width: u32,
    out_height: u32,
) -> Vec<u8> {
    let scale = (out_width as f32 / width as f32).min(out_height as f32 / height as f32);
    let scaled_width = (width as f32 * scale).round().max(1.0) as u32;
    let scaled_height = (height as f32 * scale).round().max(1.0) as u32;
    let offset_x = (out_width - scaled_width) / 2;
    let offset_y = (out_height - scaled_height) / 2;
    let mut output = vec![0; out_width as usize * out_height as usize * 4];
    for y in 0..scaled_height {
        let sy = (y as u64 * height as u64 / scaled_height as u64) as usize;
        for x in 0..scaled_width {
            let sx = (x as u64 * width as u64 / scaled_width as u64) as usize;
            let si = sy * row_pitch + sx * 4;
            let di = (((offset_y + y) as usize * out_width as usize) + (offset_x + x) as usize) * 4;
            output[di..di + 4].copy_from_slice(&source[si..si + 4]);
        }
    }
    output
}

#[cfg(windows)]
fn compose_scene(sources: &[ActiveCaptureSource]) -> Vec<u8> {
    let width = PREVIEW_WIDTH as usize;
    let height = PREVIEW_HEIGHT as usize;
    let mut canvas = vec![0; width * height * 4];
    for capture in sources.iter().rev() {
        let Some(layer) = &capture.latest else {
            continue;
        };
        let layer_width = (capture.source.width * PREVIEW_WIDTH as f32)
            .round()
            .clamp(1.0, PREVIEW_WIDTH as f32) as usize;
        let layer_height = (capture.source.height * PREVIEW_HEIGHT as f32)
            .round()
            .clamp(1.0, PREVIEW_HEIGHT as f32) as usize;
        let left = (capture.source.x * PREVIEW_WIDTH as f32)
            .round()
            .clamp(0.0, (PREVIEW_WIDTH - 1) as f32) as usize;
        let top = (capture.source.y * PREVIEW_HEIGHT as f32)
            .round()
            .clamp(0.0, (PREVIEW_HEIGHT - 1) as f32) as usize;
        for y in 0..layer_height.min(height - top) {
            for x in 0..layer_width.min(width - left) {
                let source_index = (y * layer_width + x) * 4;
                let target_index = ((top + y) * width + left + x) * 4;
                let alpha = layer[source_index + 3] as u32;
                for channel in 0..3 {
                    canvas[target_index + channel] = ((layer[source_index + channel] as u32
                        * alpha
                        + canvas[target_index + channel] as u32 * (255 - alpha))
                        / 255) as u8;
                }
                canvas[target_index + 3] = 255;
            }
        }
    }
    canvas
}

#[cfg(windows)]
fn display_capture_output(
    display_id: &str,
) -> Result<windows::Win32::Graphics::Dxgi::IDXGIOutput, String> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
    let factory_dxgi: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }
        .map_err(|cause| format!("Could not find display for capture: {cause}"))?;
    let mut adapter_index = 0;
    while let Ok(adapter) = unsafe { factory_dxgi.EnumAdapters1(adapter_index) } {
        let mut output_index = 0;
        while let Ok(output) = unsafe { adapter.EnumOutputs(output_index) } {
            let desc = unsafe { output.GetDesc() }.map_err(|cause| cause.to_string())?;
            let label = String::from_utf16_lossy(
                &desc.DeviceName[..desc
                    .DeviceName
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(desc.DeviceName.len())],
            );
            if label == display_id {
                return Ok(output);
            }
            output_index += 1;
        }
        adapter_index += 1;
    }
    Err(
        "The selected display is no longer connected. Refresh the display list and try again."
            .into(),
    )
}

#[cfg(windows)]
fn create_wic_factory() -> Result<windows::Win32::Graphics::Imaging::IWICImagingFactory, String> {
    use windows::Win32::Graphics::Imaging::CLSID_WICImagingFactory;
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
        .map_err(|cause| format!("Could not initialize preview image encoding: {cause}"))
}

#[cfg(windows)]
struct WinRtApartment;

#[cfg(windows)]
impl Drop for WinRtApartment {
    fn drop(&mut self) {
        unsafe { windows::Win32::System::WinRT::RoUninitialize() };
    }
}

#[cfg(windows)]
fn scale_bgra_to_bgr(
    source: &[u8],
    row_pitch: usize,
    width: u32,
    height: u32,
) -> (Vec<u8>, u32, u32) {
    let scale = (PREVIEW_WIDTH as f32 / width as f32)
        .min(PREVIEW_HEIGHT as f32 / height as f32)
        .min(1.0);
    let out_width = ((width as f32 * scale).round() as u32).max(1);
    let out_height = ((height as f32 * scale).round() as u32).max(1);
    let mut output = vec![0; out_width as usize * out_height as usize * 3];
    for y in 0..out_height {
        let sy = (y as u64 * height as u64 / out_height as u64) as usize;
        for x in 0..out_width {
            let sx = (x as u64 * width as u64 / out_width as u64) as usize;
            let source_index = sy * row_pitch + sx * 4;
            let target_index = (y as usize * out_width as usize + x as usize) * 3;
            output[target_index..target_index + 3]
                .copy_from_slice(&source[source_index..source_index + 3]);
        }
    }
    (output, out_width, out_height)
}

#[cfg(windows)]
fn encode_jpeg(
    imaging: &windows::Win32::Graphics::Imaging::IWICImagingFactory,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::Graphics::Imaging::{
        GUID_ContainerFormatJpeg, GUID_WICPixelFormat24bppBGR, WICBitmapEncoderNoCache,
    };
    use windows::Win32::System::Com::StructuredStorage::{
        CreateStreamOnHGlobal, GetHGlobalFromStream,
    };
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    let backing = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true) }
        .map_err(|cause| format!("Could not allocate preview frame: {cause}"))?;
    let stream = unsafe { imaging.CreateStream() }
        .map_err(|cause| format!("Could not create preview encoder stream: {cause}"))?;
    unsafe { stream.InitializeFromIStream(&backing) }
        .map_err(|cause| format!("Could not initialize preview encoder stream: {cause}"))?;
    let encoder = unsafe { imaging.CreateEncoder(&GUID_ContainerFormatJpeg, std::ptr::null()) }
        .map_err(|cause| format!("Could not create preview encoder: {cause}"))?;
    unsafe { encoder.Initialize(&stream, WICBitmapEncoderNoCache) }
        .map_err(|cause| format!("Could not initialize preview encoder: {cause}"))?;
    let mut frame = None;
    let mut options = None;
    unsafe { encoder.CreateNewFrame(&mut frame, &mut options) }
        .map_err(|cause| format!("Could not create preview image: {cause}"))?;
    let frame = frame.ok_or_else(|| "Preview image encoder returned no frame".to_owned())?;
    unsafe { frame.Initialize(options.as_ref()) }
        .map_err(|cause| format!("Could not initialize preview image: {cause}"))?;
    unsafe { frame.SetSize(width, height) }
        .map_err(|cause| format!("Could not size preview image: {cause}"))?;
    let mut format = GUID_WICPixelFormat24bppBGR;
    unsafe { frame.SetPixelFormat(&mut format) }
        .map_err(|cause| format!("Could not set preview image format: {cause}"))?;
    if format != GUID_WICPixelFormat24bppBGR {
        return Err("Windows does not support the preview image format".into());
    }
    unsafe { frame.WritePixels(height, width * 3, pixels) }
        .map_err(|cause| format!("Could not encode preview image: {cause}"))?;
    unsafe { frame.Commit() }
        .map_err(|cause| format!("Could not finish preview image: {cause}"))?;
    unsafe { encoder.Commit() }
        .map_err(|cause| format!("Could not finish preview image: {cause}"))?;
    let memory = unsafe { GetHGlobalFromStream(&backing) }
        .map_err(|cause| format!("Could not read preview image: {cause}"))?;
    let mut stat = windows::Win32::System::Com::STATSTG::default();
    unsafe { backing.Stat(&mut stat, windows::Win32::System::Com::STATFLAG_NONAME) }
        .map_err(|cause| format!("Could not read preview image size: {cause}"))?;
    let length = stat.cbSize as usize;
    let capacity = unsafe { GlobalSize(memory) };
    let pointer = unsafe { GlobalLock(memory) }.cast::<u8>();
    if pointer.is_null() || length == 0 || length > capacity {
        return Err("Preview image was empty".into());
    }
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) }.to_vec();
    unsafe { GlobalUnlock(memory) }.ok();
    Ok(bytes)
}
