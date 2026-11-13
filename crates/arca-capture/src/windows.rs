use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D11::{ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D},
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGISwapChain1,
            },
        },
    },
    core::Interface,
};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        GraphicsCaptureItemType, MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};

pub struct CapturedFrame {
    pub texture: ID3D11Texture2D,
    pub width: u32,
    pub height: u32,
    pub sequence: u64,
    pub captured_at: Instant,
}

pub struct CaptureFrames {
    queue: arca_core::pipeline::BoundedQueue<CapturedFrame>,
    active: AtomicBool,
    sequence: AtomicU64,
    seen: AtomicU64,
}

impl CaptureFrames {
    pub fn new(capacity: usize) -> Result<Arc<Self>, &'static str> {
        Ok(Arc::new(Self {
            queue: arca_core::pipeline::BoundedQueue::new(capacity)?,
            active: AtomicBool::new(false),
            sequence: AtomicU64::new(0),
            seen: AtomicU64::new(0),
        }))
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Release);
        if !active {
            while self.queue.pop_timeout(Duration::ZERO).is_some() {}
        }
    }

    pub fn next(&self, timeout: Duration) -> Option<CapturedFrame> {
        self.queue.pop_timeout(timeout)
    }

    pub fn dropped(&self) -> u64 {
        self.queue.dropped()
    }

    pub fn captured(&self) -> u64 {
        self.seen.load(Ordering::Relaxed)
    }

    fn submit(&self, texture: &ID3D11Texture2D, width: u32, height: u32) {
        let seen = self.seen.fetch_add(1, Ordering::Relaxed) + 1;
        if !self.active.load(Ordering::Acquire) {
            if seen.is_multiple_of(300) {
                tracing::trace!(captured_frames = seen, "capture preview frames received");
            }
            return;
        }
        let frame = CapturedFrame {
            texture: texture.clone(),
            width,
            height,
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            captured_at: Instant::now(),
        };
        let _ = self.queue.push_latest(frame);
        if seen.is_multiple_of(300) {
            tracing::trace!(
                captured_frames = seen,
                dropped_frames = self.queue.dropped(),
                "capture pipeline frames received"
            );
        }
    }
}

#[derive(Clone)]
pub struct CaptureTarget {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub height: u32,
    item: TargetItem,
}

#[derive(Clone, Copy)]
enum TargetItem {
    Window(Window),
    Monitor(Monitor),
}

pub struct CaptureSession {
    control: Option<
        windows_capture::capture::CaptureControl<
            CaptureHandler,
            Box<dyn std::error::Error + Send + Sync>,
        >,
    >,
}

impl CaptureSession {
    pub fn is_finished(&self) -> bool {
        self.control
            .as_ref()
            .is_none_or(|control| control.is_finished())
    }

    pub fn stop(mut self) -> Result<(), String> {
        self.control
            .take()
            .ok_or_else(|| "Capture session already stopped".to_owned())?
            .stop()
            .map_err(|error| error.to_string())
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.stop();
        }
    }
}

pub fn enumerate_targets() -> Result<Vec<CaptureTarget>, String> {
    let mut targets = Vec::new();
    for window in Window::enumerate().map_err(|error| error.to_string())? {
        let Ok(title) = window.title() else { continue };
        if title.trim().is_empty()
            || window.width().unwrap_or_default() <= 0
            || window.height().unwrap_or_default() <= 0
        {
            continue;
        }
        targets.push(CaptureTarget {
            id: format!("window:{:p}", window.as_raw_hwnd()),
            label: title,
            width: window.width().unwrap_or_default() as u32,
            height: window.height().unwrap_or_default() as u32,
            item: TargetItem::Window(window),
        });
    }
    for (index, monitor) in Monitor::enumerate()
        .map_err(|error| error.to_string())?
        .into_iter()
        .enumerate()
    {
        if monitor.width().unwrap_or_default() == 0 || monitor.height().unwrap_or_default() == 0 {
            continue;
        }
        let label = monitor
            .device_string()
            .unwrap_or_else(|_| format!("Display {}", index + 1));
        targets.push(CaptureTarget {
            id: format!("display:{index}"),
            label,
            width: monitor.width().unwrap_or_default(),
            height: monitor.height().unwrap_or_default(),
            item: TargetItem::Monitor(monitor),
        });
    }
    Ok(targets)
}

pub struct CaptureHandler {
    frames: Arc<CaptureFrames>,
    presenter: D3D11Presenter,
}

pub struct CaptureFlags {
    preview_hwnd: isize,
    width: u32,
    height: u32,
    update_interval: Duration,
    frames: Arc<CaptureFrames>,
}

struct D3D11Presenter {
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    width: u32,
    height: u32,
}

impl D3D11Presenter {
    fn new(
        device: &ID3D11Device,
        context: ID3D11DeviceContext,
        hwnd: isize,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let dxgi_device = device
            .cast::<windows::Win32::Graphics::Dxgi::IDXGIDevice>()
            .map_err(|e| e.to_string())?;
        let adapter = unsafe { dxgi_device.GetAdapter() }.map_err(|e| e.to_string())?;
        let factory =
            unsafe { adapter.GetParent::<windows::Win32::Graphics::Dxgi::IDXGIFactory2>() }
                .map_err(|e| e.to_string())?;
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width.max(1),
            Height: height.max(1),
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            Flags: 0,
        };
        let swap_chain = unsafe {
            factory.CreateSwapChainForHwnd(device, HWND(hwnd as *mut _), &desc, None, None)
        }
        .map_err(|e| format!("Could not create the native preview surface: {e}"))?;
        Ok(Self {
            context,
            swap_chain,
            width: width.max(1),
            height: height.max(1),
        })
    }

    fn present(&mut self, frame: &ID3D11Texture2D, width: u32, height: u32) -> Result<(), String> {
        if self.width != width || self.height != height {
            unsafe {
                self.swap_chain.ResizeBuffers(
                    0,
                    width,
                    height,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_SWAP_CHAIN_FLAG(0),
                )
            }
            .map_err(|e| format!("Could not resize the native preview surface: {e}"))?;
            self.width = width;
            self.height = height;
        }
        let back_buffer = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }
            .map_err(|e| e.to_string())?;
        unsafe {
            self.context.CopyResource(&back_buffer, frame);
        }
        unsafe {
            self.swap_chain
                .Present(0, windows::Win32::Graphics::Dxgi::DXGI_PRESENT(0))
        }
        .ok()
        .map_err(|e| format!("Could not present the captured frame: {e}"))
    }
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = CaptureFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let CaptureFlags {
            preview_hwnd,
            width,
            height,
            frames,
            ..
        } = ctx.flags;
        Ok(Self {
            frames,
            presenter: D3D11Presenter::new(
                &ctx.device,
                ctx.device_context,
                preview_hwnd,
                width,
                height,
            )?,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        self.presenter
            .present(frame.as_raw_texture(), frame.width(), frame.height())
            .map_err(std::io::Error::other)?;
        self.frames
            .submit(frame.as_raw_texture(), frame.width(), frame.height());
        Ok(())
    }
}

pub fn start_capture(
    target: &CaptureTarget,
    preview_hwnd: isize,
    frames: Arc<CaptureFrames>,
    frame_rate: u32,
) -> Result<CaptureSession, String> {
    if !(1..=60).contains(&frame_rate) {
        return Err("Capture frame rate must be between 1 and 60 fps".into());
    }
    let flags = CaptureFlags {
        preview_hwnd,
        width: target.width,
        height: target.height,
        update_interval: Duration::from_secs_f64(1.0 / f64::from(frame_rate)),
        frames,
    };
    match target.item {
        TargetItem::Window(window) => start_item(window, flags),
        TargetItem::Monitor(monitor) => start_item(monitor, flags),
    }
}

fn start_item<T>(item: T, flags: CaptureFlags) -> Result<CaptureSession, String>
where
    T: TryInto<GraphicsCaptureItemType> + Send + 'static,
{
    let settings = Settings::new(
        item,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Custom(flags.update_interval),
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        flags,
    );
    CaptureHandler::start_free_threaded(settings)
        .map(|control| CaptureSession {
            control: Some(control),
        })
        .map_err(|error| error.to_string())
}
