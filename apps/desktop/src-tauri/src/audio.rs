use serde::Serialize;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSource {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub process_id: Option<u32>,
    pub volume: f64,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputLevel {
    pub id: String,
    pub peak: Option<f64>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct AudioMixer {
    tracks: Mutex<Vec<AudioTrack>>,
}

#[derive(Default)]
pub struct AudioMeter {
    worker: Mutex<Option<AudioMeterWorker>>,
    levels: Arc<Mutex<Vec<AudioInputLevel>>>,
}

struct AudioMeterWorker {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for AudioMeterWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioMeter {
    fn start(&self, tracks: Vec<AudioTrack>) -> Result<(), String> {
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Microphone meter state is unavailable")?;
        worker.take();

        let mut levels = self
            .levels
            .lock()
            .map_err(|_| "Microphone meter state is unavailable")?;
        levels.clear();
        levels.extend(tracks.iter().map(|track| AudioInputLevel {
            id: track.id.clone(),
            peak: None,
            error: None,
        }));
        drop(levels);

        if tracks.is_empty() {
            return Ok(());
        }

        let stop = Arc::new(AtomicBool::new(false));
        let thread = spawn_meter_worker(tracks, Arc::clone(&self.levels), Arc::clone(&stop))?;
        *worker = Some(AudioMeterWorker {
            stop,
            thread: Some(thread),
        });
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        self.worker
            .lock()
            .map_err(|_| "Microphone meter state is unavailable")?
            .take();
        self.levels
            .lock()
            .map_err(|_| "Microphone meter state is unavailable")?
            .clear();
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<AudioInputLevel>, String> {
        self.levels
            .lock()
            .map(|levels| levels.clone())
            .map_err(|_| "Microphone meter state is unavailable".into())
    }
}

impl AudioMixer {
    fn snapshot(&self) -> Result<Vec<AudioTrack>, String> {
        self.tracks
            .lock()
            .map(|tracks| tracks.clone())
            .map_err(|_| "Audio mixer state is unavailable".into())
    }
}

#[tauri::command]
pub fn get_audio_sources(kind: String) -> Result<Vec<AudioSource>, String> {
    list_sources(&kind)
}

#[tauri::command]
pub fn get_audio_tracks(mixer: State<'_, AudioMixer>) -> Result<Vec<AudioTrack>, String> {
    mixer.snapshot()
}

#[tauri::command]
pub fn add_audio_track(
    mixer: State<'_, AudioMixer>,
    kind: String,
    id: String,
) -> Result<Vec<AudioTrack>, String> {
    if mixer
        .tracks
        .lock()
        .map_err(|_| "Audio mixer state is unavailable")?
        .iter()
        .any(|track| track.id == id)
    {
        return Err("Audio source is already in the mixer".into());
    }

    let source = list_sources(&kind)?
        .into_iter()
        .find(|source| source.id == id)
        .ok_or_else(|| {
            "Audio source is no longer available. Refresh and choose it again.".to_owned()
        })?;

    let mut tracks = mixer
        .tracks
        .lock()
        .map_err(|_| "Audio mixer state is unavailable")?;
    if tracks.iter().any(|track| track.id == source.id) {
        return Err("Audio source is already in the mixer".into());
    }
    tracks.push(AudioTrack {
        id: source.id,
        label: source.label,
        kind: source.kind,
        process_id: source.process_id,
        volume: if kind == "desktop" { 0.7 } else { 0.8 },
        muted: false,
    });
    Ok(tracks.clone())
}

#[tauri::command]
pub fn set_audio_track_settings(
    mixer: State<'_, AudioMixer>,
    id: String,
    volume: f64,
    muted: bool,
) -> Result<Vec<AudioTrack>, String> {
    if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
        return Err("Audio volume must be between 0 and 1".into());
    }
    let mut tracks = mixer
        .tracks
        .lock()
        .map_err(|_| "Audio mixer state is unavailable")?;
    let track = tracks
        .iter_mut()
        .find(|track| track.id == id)
        .ok_or_else(|| "Audio source is no longer in the mixer".to_owned())?;
    track.volume = volume;
    track.muted = muted;
    Ok(tracks.clone())
}

#[tauri::command]
pub fn remove_audio_track(
    mixer: State<'_, AudioMixer>,
    id: String,
) -> Result<Vec<AudioTrack>, String> {
    let mut tracks = mixer
        .tracks
        .lock()
        .map_err(|_| "Audio mixer state is unavailable")?;
    let original_len = tracks.len();
    tracks.retain(|track| track.id != id);
    if tracks.len() == original_len {
        return Err("Audio source is no longer in the mixer".into());
    }
    Ok(tracks.clone())
}

#[tauri::command]
pub fn start_audio_meter(
    mixer: State<'_, AudioMixer>,
    meter: State<'_, AudioMeter>,
) -> Result<(), String> {
    let ids = mixer
        .snapshot()?
        .into_iter()
        .filter(|track| track.kind == "mic" || track.kind == "application")
        .collect();
    meter.start(ids)
}

#[tauri::command]
pub fn stop_audio_meter(meter: State<'_, AudioMeter>) -> Result<(), String> {
    meter.stop()
}

#[tauri::command]
pub fn get_audio_input_levels(meter: State<'_, AudioMeter>) -> Result<Vec<AudioInputLevel>, String> {
    meter.snapshot()
}

#[cfg(windows)]
fn spawn_meter_worker(
    tracks: Vec<AudioTrack>,
    levels: Arc<Mutex<Vec<AudioInputLevel>>>,
    stop: Arc<AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, String> {
    std::thread::Builder::new()
        .name("arca-mic-meter".into())
        .spawn(move || windows_audio::meter_sources(tracks, levels, stop))
        .map_err(|error| format!("Could not start microphone meter: {error}"))
}

#[cfg(not(windows))]
fn spawn_meter_worker(
    _tracks: Vec<AudioTrack>,
    _levels: Arc<Mutex<Vec<AudioInputLevel>>>,
    _stop: Arc<AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, String> {
    Err("Microphone metering is only available on Windows".into())
}

#[cfg(windows)]
mod windows_audio {
    use super::{AudioInputLevel, AudioSource, AudioTrack};
    use std::collections::HashSet;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;
    use std::time::{Duration, Instant};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        ActivateAudioInterfaceAsync, AudioSessionStateActive, DEVICE_STATE_ACTIVE,
        IActivateAudioInterfaceAsyncOperation, IActivateAudioInterfaceCompletionHandler,
        IActivateAudioInterfaceCompletionHandler_Impl,
        IAudioSessionControl2, IAudioSessionManager2, AUDCLNT_BUFFERFLAGS_SILENT,
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
        AUDIOCLIENT_ACTIVATION_PARAMS, AUDIOCLIENT_ACTIVATION_PARAMS_0,
        AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK, AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS,
        IMMDevice, IMMDeviceEnumerator, IAudioCaptureClient, IAudioClient, MMDeviceEnumerator,
        PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE, VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
        WAVEFORMATEX, WAVEFORMATEXTENSIBLE, eCapture, eConsole, eRender,
    };
    use windows::Win32::System::Com::StructuredStorage::{
        PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0, PropVariantClear,
        PropVariantToStringAlloc,
    };
    use windows::Win32::System::Com::{
        BLOB, CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
        CoUninitialize, IAgileObject, IAgileObject_Impl, STGM_READ,
    };
    use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use std::sync::Condvar;
    use std::mem::ManuallyDrop;
    use windows::Win32::System::Variant::VT_BLOB;
    use windows::core::{GUID, HRESULT, Interface, PCWSTR};
    use windows_core::implement;

    struct ComApartment;

    const PROCESS_LOOPBACK_MIN_BUILD: u32 = 20348;

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(version: *mut OSVERSIONINFOW) -> i32;
    }

    fn process_loopback_supported() -> bool {
        let mut version = OSVERSIONINFOW::default();
        version.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        unsafe {
            RtlGetVersion(&mut version) == 0 && version.dwBuildNumber >= PROCESS_LOOPBACK_MIN_BUILD
        }
    }

    impl ComApartment {
        fn initialize() -> Result<Self, String> {
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
                .ok()
                .map_err(|error| format!("Windows audio initialization failed: {error}"))?;
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    fn enumerator() -> Result<IMMDeviceEnumerator, String> {
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
            .map_err(|error| format!("Windows audio device discovery failed: {error}"))
    }

    struct MixFormat(*mut WAVEFORMATEX);

    impl Drop for MixFormat {
        fn drop(&mut self) {
            unsafe { CoTaskMemFree(Some(self.0.cast())) };
        }
    }

    #[derive(Clone, Copy)]
    enum SampleFormat {
        Float32,
        Pcm8,
        Pcm16,
        Pcm24,
        Pcm32,
    }

    struct MicCapture {
        client: IAudioClient,
        capture: IAudioCaptureClient,
        format: SampleFormat,
        block_align: usize,
    }

    struct ActivationResult {
        client: Mutex<Option<Result<IAudioClient, String>>>,
        ready: Condvar,
    }

    #[implement(IActivateAudioInterfaceCompletionHandler, IAgileObject)]
    struct ProcessLoopbackActivation {
        result: Arc<ActivationResult>,
    }

    impl IActivateAudioInterfaceCompletionHandler_Impl for ProcessLoopbackActivation_Impl {
        fn ActivateCompleted(
            &self,
            operation: windows::core::Ref<'_, IActivateAudioInterfaceAsyncOperation>,
        ) -> windows::core::Result<()> {
            let result = (|| {
                let operation = operation.as_ref()
                    .ok_or_else(|| "Windows returned no activation result".to_owned())?;
                let mut activation_status = HRESULT(0);
                let mut activated = None;
                unsafe { operation.GetActivateResult(&mut activation_status, &mut activated) }
                    .map_err(|error| format!("Could not activate application audio: {error}"))?;
                activation_status
                    .ok()
                    .map_err(|error| format!("Could not activate application audio: {error}"))?;
                activated
                    .ok_or_else(|| "Windows returned no application audio interface".to_owned())?
                    .cast::<IAudioClient>()
                    .map_err(|error| format!("Could not open application audio: {error}"))
            })();

            *self.result.client.lock().unwrap_or_else(|error| error.into_inner()) = Some(result);
            self.result.ready.notify_one();
            Ok(())
        }
    }

    impl IAgileObject_Impl for ProcessLoopbackActivation_Impl {}

    struct MeterEntry {
        id: String,
        capture: Option<MicCapture>,
        peak: f32,
        error: Option<String>,
    }

    const PCM_SUBFORMAT: GUID =
        GUID::from_values(1, 0, 0x0010, [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71]);
    const IEEE_FLOAT_SUBFORMAT: GUID =
        GUID::from_values(3, 0, 0x0010, [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71]);

    fn microphone_capture(
        enumerator: &IMMDeviceEnumerator,
        id: &str,
    ) -> Result<MicCapture, String> {
        let id_wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        let device = unsafe { enumerator.GetDevice(PCWSTR(id_wide.as_ptr())) }
            .map_err(|error| format!("Could not open microphone: {error}"))?;
        let client = unsafe { device.Activate::<IAudioClient>(CLSCTX_ALL, None) }
            .map_err(|error| format!("Could not access microphone: {error}"))?;
        let mix_format = MixFormat(
            unsafe { client.GetMixFormat() }
                .map_err(|error| format!("Could not read microphone format: {error}"))?,
        );
        if mix_format.0.is_null() {
            return Err("Windows returned an empty microphone format".into());
        }

        let wave = unsafe { mix_format.0.read_unaligned() };
        let sub_format = if wave.wFormatTag == 0xfffe {
            let extensible = unsafe { mix_format.0.cast::<WAVEFORMATEXTENSIBLE>().read_unaligned() };
            extensible.SubFormat
        } else {
            GUID::zeroed()
        };
        let sample_format = match (wave.wFormatTag, wave.wBitsPerSample, sub_format) {
            (3, 32, _) | (0xfffe, 32, IEEE_FLOAT_SUBFORMAT) => SampleFormat::Float32,
            (1, 8, _) | (0xfffe, 8, PCM_SUBFORMAT) => SampleFormat::Pcm8,
            (1, 16, _) | (0xfffe, 16, PCM_SUBFORMAT) => SampleFormat::Pcm16,
            (1, 24, _) | (0xfffe, 24, PCM_SUBFORMAT) => SampleFormat::Pcm24,
            (1, 32, _) | (0xfffe, 32, PCM_SUBFORMAT) => SampleFormat::Pcm32,
            _ => return Err("This microphone uses an unsupported sample format".into()),
        };
        let channels = wave.nChannels as usize;
        let block_align = wave.nBlockAlign as usize;
        let sample_bytes = wave.wBitsPerSample as usize / 8;
        if channels == 0 || block_align == 0 || channels.saturating_mul(sample_bytes) > block_align {
            return Err("Windows returned an invalid microphone format".into());
        }

        unsafe {
            client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    0,
                    1_000_000,
                    0,
                    mix_format.0,
                    None,
                )
                .map_err(|error| format!("Could not start microphone metering: {error}"))?;
        }
        let capture = unsafe { client.GetService::<IAudioCaptureClient>() }
            .map_err(|error| format!("Could not read microphone input: {error}"))?;
        unsafe { client.Start() }
            .map_err(|error| format!("Could not start microphone input: {error}"))?;

        Ok(MicCapture {
            client,
            capture,
            format: sample_format,
            block_align,
        })
    }

    fn application_capture(process_id: u32) -> Result<MicCapture, String> {
        let state = Arc::new(ActivationResult {
            client: Mutex::new(None),
            ready: Condvar::new(),
        });
        let handler: IActivateAudioInterfaceCompletionHandler = ProcessLoopbackActivation {
            result: Arc::clone(&state),
        }
        .into();
        let mut activation = AUDIOCLIENT_ACTIVATION_PARAMS {
            ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
            Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
                ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                    TargetProcessId: process_id,
                    ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE,
                },
            },
        };
        let mut params = PROPVARIANT::default();
        params.Anonymous = PROPVARIANT_0 {
            Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
                vt: VT_BLOB,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: PROPVARIANT_0_0_0 {
                    blob: BLOB {
                        cbSize: std::mem::size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
                        pBlobData: (&mut activation as *mut AUDIOCLIENT_ACTIVATION_PARAMS).cast(),
                    },
                },
            }),
        };
        let _operation = unsafe {
            ActivateAudioInterfaceAsync(
                VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
                &IAudioClient::IID,
                Some(&params as *const PROPVARIANT),
                &handler,
            )
        }
        .map_err(|error| format!("Could not start application audio capture: {error}"))?;

        let result = state.client.lock().map_err(|_| "Application audio activation failed")?;
        let (mut result, timeout) = state
            .ready
            .wait_timeout_while(result, Duration::from_secs(5), |result| result.is_none())
            .map_err(|_| "Application audio activation failed")?;
        if timeout.timed_out() && result.is_none() {
            return Err("Timed out while opening application audio".into());
        }
        let client = result
            .take()
            .ok_or_else(|| "Windows did not finish opening application audio".to_owned())??;
        let mix_format = MixFormat(
            unsafe { client.GetMixFormat() }
                .map_err(|error| format!("Could not read application audio format: {error}"))?,
        );
        if mix_format.0.is_null() {
            return Err("Windows returned an empty application audio format".into());
        }
        let wave = unsafe { mix_format.0.read_unaligned() };
        let sub_format = if wave.wFormatTag == 0xfffe {
            unsafe { mix_format.0.cast::<WAVEFORMATEXTENSIBLE>().read_unaligned() }.SubFormat
        } else {
            GUID::zeroed()
        };
        let sample_format = match (wave.wFormatTag, wave.wBitsPerSample, sub_format) {
            (3, 32, _) | (0xfffe, 32, IEEE_FLOAT_SUBFORMAT) => SampleFormat::Float32,
            (1, 8, _) | (0xfffe, 8, PCM_SUBFORMAT) => SampleFormat::Pcm8,
            (1, 16, _) | (0xfffe, 16, PCM_SUBFORMAT) => SampleFormat::Pcm16,
            (1, 24, _) | (0xfffe, 24, PCM_SUBFORMAT) => SampleFormat::Pcm24,
            (1, 32, _) | (0xfffe, 32, PCM_SUBFORMAT) => SampleFormat::Pcm32,
            _ => return Err("This application uses an unsupported audio format".into()),
        };
        let block_align = wave.nBlockAlign as usize;
        if wave.nChannels == 0 || block_align == 0 {
            return Err("Windows returned an invalid application audio format".into());
        }
        unsafe {
            client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_LOOPBACK,
                    1_000_000,
                    0,
                    mix_format.0,
                    None,
                )
                .map_err(|error| format!("Could not initialize application audio: {error}"))?;
        }
        let capture = unsafe { client.GetService::<IAudioCaptureClient>() }
            .map_err(|error| format!("Could not read application audio: {error}"))?;
        unsafe { client.Start() }
            .map_err(|error| format!("Could not start application audio: {error}"))?;
        Ok(MicCapture {
            client,
            capture,
            format: sample_format,
            block_align,
        })
    }

    fn read_peak(capture: &IAudioCaptureClient, format: SampleFormat, block_align: usize) -> Result<Option<f32>, String> {
        let mut peak = 0.0f32;
        loop {
            let frames = unsafe { capture.GetNextPacketSize() }
                .map_err(|error| format!("Could not read microphone input: {error}"))?;
            if frames == 0 {
                return Ok(Some(peak));
            }

            let mut data = std::ptr::null_mut();
            let mut frame_count = 0;
            let mut flags = 0;
            unsafe {
                capture
                    .GetBuffer(&mut data, &mut frame_count, &mut flags, None, None)
                    .map_err(|error| format!("Could not read microphone input: {error}"))?;
            }

            if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 == 0 && !data.is_null() {
                let bytes = frame_count as usize * block_align;
                let packet_peak = unsafe { peak_from_samples(data, bytes, format) };
                peak = peak.max(packet_peak);
            }

            unsafe { capture.ReleaseBuffer(frame_count) }
                .map_err(|error| format!("Could not release microphone buffer: {error}"))?;
        }
    }

    unsafe fn peak_from_samples(data: *const u8, length: usize, format: SampleFormat) -> f32 {
        let sample_bytes = match format {
            SampleFormat::Float32 | SampleFormat::Pcm32 => 4,
            SampleFormat::Pcm24 => 3,
            SampleFormat::Pcm16 => 2,
            SampleFormat::Pcm8 => 1,
        };
        let sample_count = length / sample_bytes;
        let mut peak = 0.0f32;
        for index in 0..sample_count {
            let sample = unsafe { data.add(index * sample_bytes) };
            let value = match format {
                SampleFormat::Float32 => {
                    let value = unsafe { std::ptr::read_unaligned(sample.cast::<f32>()) };
                    if value.is_finite() { value.abs().min(1.0) } else { 0.0 }
                }
                SampleFormat::Pcm8 => (unsafe { *sample }) as f32 / 128.0 - 1.0,
                SampleFormat::Pcm16 => {
                    unsafe { std::ptr::read_unaligned(sample.cast::<i16>()) as f32 / 32768.0 }
                }
                SampleFormat::Pcm24 => {
                    let sign = if unsafe { *sample.add(2) } & 0x80 != 0 { 0xff } else { 0 };
                    let value = i32::from_le_bytes([
                        unsafe { *sample },
                        unsafe { *sample.add(1) },
                        unsafe { *sample.add(2) },
                        sign,
                    ]);
                    value as f32 / 8_388_608.0
                }
                SampleFormat::Pcm32 => {
                    unsafe { std::ptr::read_unaligned(sample.cast::<i32>()) as f32 / 2_147_483_648.0 }
                }
            };
            peak = peak.max(value.abs());
        }
        peak.min(1.0)
    }

    fn publish_levels(levels: &Mutex<Vec<AudioInputLevel>>, entries: &[MeterEntry]) {
        if let Ok(mut current) = levels.lock() {
            *current = entries
                .iter()
                .map(|entry| AudioInputLevel {
                    id: entry.id.clone(),
                    peak: entry.error.is_none().then_some(entry.peak as f64),
                    error: entry.error.clone(),
                })
                .collect();
        }
    }

    pub fn meter_sources(tracks: Vec<AudioTrack>, levels: Arc<Mutex<Vec<AudioInputLevel>>>, stop: Arc<AtomicBool>) {
        let ids = tracks.iter().map(|track| track.id.clone()).collect::<Vec<_>>();
        let _apartment = match ComApartment::initialize() {
            Ok(apartment) => apartment,
            Err(error) => {
                set_meter_errors(&levels, &ids, &error);
                return;
            }
        };
        let enumerator = match enumerator() {
            Ok(enumerator) => enumerator,
            Err(error) => {
                set_meter_errors(&levels, &ids, &error);
                return;
            }
        };

        let mut entries = tracks
            .into_iter()
            .map(|track| {
                let capture = match track.kind.as_str() {
                    "mic" => microphone_capture(&enumerator, &track.id),
                    "application" => track.process_id
                        .ok_or_else(|| "Application process is unavailable".to_owned())
                        .and_then(application_capture),
                    _ => Err("This audio source does not have an input meter".into()),
                };
                match capture {
                    Ok(capture) => MeterEntry { id: track.id, capture: Some(capture), peak: 0.0, error: None },
                    Err(error) => MeterEntry { id: track.id, capture: None, peak: 0.0, error: Some(error) },
                }
            })
            .collect::<Vec<_>>();
        publish_levels(&levels, &entries);

        let mut last_publish = Instant::now();
        while !stop.load(Ordering::Relaxed) {
            for entry in &mut entries {
                if entry.error.is_some() {
                    continue;
                }
                let result = entry.capture.as_ref().map(|capture| {
                    read_peak(&capture.capture, capture.format, capture.block_align)
                });
                match result {
                    Some(Ok(Some(peak))) => entry.peak = entry.peak.max(peak),
                    Some(Ok(None)) => {}
                    Some(Err(error)) => {
                        entry.error = Some(error);
                        if let Some(capture) = entry.capture.take() {
                            let _ = unsafe { capture.client.Stop() };
                        }
                    }
                    None => entry.error = Some("Microphone input is unavailable".into()),
                }
            }

            if last_publish.elapsed() >= Duration::from_millis(50) {
                publish_levels(&levels, &entries);
                for entry in &mut entries {
                    if entry.error.is_none() {
                        entry.peak = 0.0;
                    }
                }
                last_publish = Instant::now();
            }
            thread::sleep(Duration::from_millis(5));
        }

        for entry in entries {
            if let Some(capture) = entry.capture {
                let _ = unsafe { capture.client.Stop() };
            }
        }
    }

    fn set_meter_errors(levels: &Mutex<Vec<AudioInputLevel>>, ids: &[String], error: &str) {
        if let Ok(mut current) = levels.lock() {
            *current = ids
                .iter()
                .map(|id| AudioInputLevel {
                    id: id.clone(),
                    peak: None,
                    error: Some(error.to_owned()),
                })
                .collect();
        }
    }

    fn take_wide_string(value: windows::core::PWSTR) -> String {
        if value.is_null() {
            return String::new();
        }
        let string = unsafe { value.to_string() }.unwrap_or_default();
        unsafe { CoTaskMemFree(Some(value.0.cast())) };
        string
    }

    fn device_id(device: &IMMDevice) -> Result<String, String> {
        unsafe { device.GetId() }
            .map(take_wide_string)
            .map_err(|error| format!("Could not read an audio device identifier: {error}"))
    }

    fn device_label(device: &IMMDevice) -> Result<String, String> {
        let store: IPropertyStore = unsafe { device.OpenPropertyStore(STGM_READ) }
            .map_err(|error| format!("Could not read an audio device name: {error}"))?;
        let mut value: PROPVARIANT = unsafe { store.GetValue(&PKEY_Device_FriendlyName) }
            .map_err(|error| format!("Could not read an audio device name: {error}"))?;
        let result = unsafe { PropVariantToStringAlloc(&value) }
            .map(take_wide_string)
            .map_err(|error| format!("Could not read an audio device name: {error}"));
        let _ = unsafe { PropVariantClear(&mut value) };
        result
    }

    fn process_label(process_id: u32) -> Option<String> {
        let process =
            unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }.ok()?;
        let mut path = vec![0u16; 1024];
        let mut length = path.len() as u32;
        let result = unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(path.as_mut_ptr()),
                &mut length,
            )
        };
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(process) };
        result.ok()?;
        let path = String::from_utf16(&path[..length as usize]).ok()?;
        std::path::Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
    }

    fn active_microphones() -> Result<Vec<AudioSource>, String> {
        let enumerator = enumerator()?;
        let devices = unsafe { enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE) }
            .map_err(|error| format!("Could not list microphones: {error}"))?;
        let count = unsafe { devices.GetCount() }
            .map_err(|error| format!("Could not count microphones: {error}"))?;
        let mut sources = Vec::with_capacity(count as usize);

        for index in 0..count {
            let device = unsafe { devices.Item(index) }
                .map_err(|error| format!("Could not read microphone {index}: {error}"))?;
            let id = device_id(&device)?;
            let label = device_label(&device)?;
            sources.push(AudioSource {
                id,
                label,
                kind: "mic".into(),
                process_id: None,
            });
        }
        Ok(sources)
    }

    fn default_desktop_audio() -> Result<Vec<AudioSource>, String> {
        let device = unsafe { enumerator()?.GetDefaultAudioEndpoint(eRender, eConsole) }
            .map_err(|error| format!("No default desktop audio output is available: {error}"))?;
        Ok(vec![AudioSource {
            id: device_id(&device)?,
            label: "Desktop audio".into(),
            kind: "desktop".into(),
            process_id: None,
        }])
    }

    fn active_applications() -> Result<Vec<AudioSource>, String> {
        let enumerator = enumerator()?;
        let devices = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) }
            .map_err(|error| format!("Could not inspect active audio outputs: {error}"))?;
        let device_count = unsafe { devices.GetCount() }
            .map_err(|error| format!("Could not count active audio outputs: {error}"))?;
        let mut sources = Vec::new();
        let mut seen = HashSet::new();

        for device_index in 0..device_count {
            let Ok(device) = (unsafe { devices.Item(device_index) }) else {
                continue;
            };
            let Ok(manager) =
                (unsafe { device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) })
            else {
                continue;
            };
            let Ok(sessions) = (unsafe { manager.GetSessionEnumerator() }) else {
                continue;
            };
            let Ok(count) = (unsafe { sessions.GetCount() }) else {
                continue;
            };

            for session_index in 0..count {
                let Ok(session) = (unsafe { sessions.GetSession(session_index) }) else {
                    continue;
                };
                if unsafe { session.GetState() }.ok() != Some(AudioSessionStateActive) {
                    continue;
                }
                let Ok(session) = session.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let Ok(process_id) = (unsafe { session.GetProcessId() }) else {
                    continue;
                };
                if process_id == 0 || !seen.insert(process_id) {
                    continue;
                }
                let session_control: windows::Win32::Media::Audio::IAudioSessionControl =
                    match session.cast() {
                        Ok(control) => control,
                        Err(_) => continue,
                    };
                let label = unsafe { session_control.GetDisplayName() }
                    .map(take_wide_string)
                    .unwrap_or_default();
                let label = if label.trim().is_empty() {
                    process_label(process_id)
                        .unwrap_or_else(|| format!("Application ({process_id})"))
                } else {
                    label
                };
                sources.push(AudioSource {
                    id: format!("{process_id}"),
                    label,
                    kind: "application".into(),
                    process_id: Some(process_id),
                });
            }
        }

        sources.sort_by(|left, right| left.label.to_lowercase().cmp(&right.label.to_lowercase()));
        sources.dedup_by(|left, right| left.process_id == right.process_id);
        Ok(sources)
    }

    pub fn list_sources(kind: &str) -> Result<Vec<AudioSource>, String> {
        if !process_loopback_supported() {
            return Err(format!(
                "Audio capture requires Windows build {PROCESS_LOOPBACK_MIN_BUILD} or newer."
            ));
        }
        let _apartment = ComApartment::initialize()?;
        match kind {
            "mic" => active_microphones(),
            "desktop" => default_desktop_audio(),
            "application" => active_applications(),
            _ => Err("Unknown audio source kind".into()),
        }
    }
}

#[cfg(windows)]
pub fn list_sources(kind: &str) -> Result<Vec<AudioSource>, String> {
    let kind = kind.to_owned();
    std::thread::Builder::new()
        .name("arca-audio-discovery".into())
        .spawn(move || windows_audio::list_sources(&kind))
        .map_err(|error| format!("Could not start audio discovery: {error}"))?
        .join()
        .map_err(|_| "Audio discovery stopped unexpectedly".to_owned())?
}

#[cfg(not(windows))]
pub fn list_sources(_kind: &str) -> Result<Vec<AudioSource>, String> {
    Err("Audio source discovery is only available on Windows".into())
}
