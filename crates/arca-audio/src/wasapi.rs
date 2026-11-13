use crate::{AudioSource as DeviceSource, AudioSourceKind};
use arca_core::pipeline::BoundedQueue;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

const OUTPUT_SAMPLE_RATE: u32 = 48_000;
const AUDIO_BLOCK_FRAMES: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioInputLevel {
    pub id: String,
    pub peak: Option<f64>,
    pub error: Option<String>,
}

struct AudioTrack {
    id: String,
    kind: String,
    process_id: Option<u32>,
}
struct AudioSource {
    id: String,
    label: String,
    kind: String,
    process_id: Option<u32>,
}

pub struct AudioMeter {
    stop: Arc<AtomicBool>,
    levels: Arc<Mutex<Vec<AudioInputLevel>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub struct AudioCaptureStream {
    stop: Arc<AtomicBool>,
    queue: Arc<BoundedQueue<crate::AudioPcmBlock>>,
    error: Arc<Mutex<Option<String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioCaptureStream {
    pub fn pop_timeout(&self, timeout: Duration) -> Option<crate::AudioPcmBlock> {
        self.queue.pop_timeout(timeout)
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|error| error.clone())
    }
}

impl Drop for AudioCaptureStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.queue.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioMeter {
    pub fn snapshot(&self) -> Vec<AudioInputLevel> {
        self.levels
            .lock()
            .map(|levels| levels.clone())
            .unwrap_or_default()
    }
}

impl Drop for AudioMeter {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
#[cfg(windows)]
mod windows_audio {
    use super::{
        AUDIO_BLOCK_FRAMES, AudioInputLevel, AudioSource, AudioTrack, BoundedQueue,
        OUTPUT_SAMPLE_RATE,
    };
    use std::collections::HashSet;
    use std::mem::ManuallyDrop;
    use std::sync::Condvar;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;
    use std::time::{Duration, Instant};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
        AUDIOCLIENT_ACTIVATION_PARAMS, AUDIOCLIENT_ACTIVATION_PARAMS_0,
        AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK, AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS,
        ActivateAudioInterfaceAsync, AudioSessionStateActive, DEVICE_STATE_ACTIVE,
        IActivateAudioInterfaceAsyncOperation, IActivateAudioInterfaceCompletionHandler,
        IActivateAudioInterfaceCompletionHandler_Impl, IAudioCaptureClient, IAudioClient,
        IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator,
        MMDeviceEnumerator, PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE,
        VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, WAVEFORMATEX, WAVEFORMATEXTENSIBLE, eCapture,
        eConsole, eRender,
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
    use windows::Win32::System::Variant::VT_BLOB;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
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
        channels: usize,
        sample_rate: u32,
    }

    struct MixerEntry {
        capture: MicCapture,
        resample_phase: u64,
        pending: std::collections::VecDeque<[f32; 2]>,
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
                let operation = operation
                    .as_ref()
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

            *self
                .result
                .client
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = Some(result);
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

    const PCM_SUBFORMAT: GUID = GUID::from_values(
        1,
        0,
        0x0010,
        [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
    );
    const IEEE_FLOAT_SUBFORMAT: GUID = GUID::from_values(
        3,
        0,
        0x0010,
        [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
    );

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
            let extensible =
                unsafe { mix_format.0.cast::<WAVEFORMATEXTENSIBLE>().read_unaligned() };
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
        if channels == 0 || block_align == 0 || channels.saturating_mul(sample_bytes) > block_align
        {
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
            channels,
            sample_rate: wave.nSamplesPerSec,
        })
    }

    fn desktop_capture(enumerator: &IMMDeviceEnumerator) -> Result<MicCapture, String> {
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }
            .map_err(|error| format!("Could not open desktop audio: {error}"))?;
        let client = unsafe { device.Activate::<IAudioClient>(CLSCTX_ALL, None) }
            .map_err(|error| format!("Could not access desktop audio: {error}"))?;
        let mix_format = MixFormat(
            unsafe { client.GetMixFormat() }
                .map_err(|error| format!("Could not read desktop audio format: {error}"))?,
        );
        if mix_format.0.is_null() {
            return Err("Windows returned an empty desktop audio format".into());
        }

        let wave = unsafe { mix_format.0.read_unaligned() };
        let sub_format = if wave.wFormatTag == 0xfffe {
            unsafe { mix_format.0.cast::<WAVEFORMATEXTENSIBLE>().read_unaligned() }.SubFormat
        } else {
            GUID::zeroed()
        };
        let format = match (wave.wFormatTag, wave.wBitsPerSample, sub_format) {
            (3, 32, _) | (0xfffe, 32, IEEE_FLOAT_SUBFORMAT) => SampleFormat::Float32,
            (1, 8, _) | (0xfffe, 8, PCM_SUBFORMAT) => SampleFormat::Pcm8,
            (1, 16, _) | (0xfffe, 16, PCM_SUBFORMAT) => SampleFormat::Pcm16,
            (1, 24, _) | (0xfffe, 24, PCM_SUBFORMAT) => SampleFormat::Pcm24,
            (1, 32, _) | (0xfffe, 32, PCM_SUBFORMAT) => SampleFormat::Pcm32,
            _ => return Err("Desktop audio uses an unsupported sample format".into()),
        };
        if wave.nChannels == 0 || wave.nBlockAlign == 0 {
            return Err("Windows returned an invalid desktop audio format".into());
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
                .map_err(|error| format!("Could not initialize desktop audio: {error}"))?;
        }
        let capture = unsafe { client.GetService::<IAudioCaptureClient>() }
            .map_err(|error| format!("Could not read desktop audio: {error}"))?;
        unsafe { client.Start() }
            .map_err(|error| format!("Could not start desktop audio: {error}"))?;
        Ok(MicCapture {
            client,
            capture,
            format,
            block_align: wave.nBlockAlign as usize,
            channels: wave.nChannels as usize,
            sample_rate: wave.nSamplesPerSec,
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

        let result = state
            .client
            .lock()
            .map_err(|_| "Application audio activation failed")?;
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
            channels: wave.nChannels as usize,
            sample_rate: wave.nSamplesPerSec,
        })
    }

    fn open_capture(
        enumerator: &IMMDeviceEnumerator,
        track: &AudioTrack,
    ) -> Result<MicCapture, String> {
        match track.kind.as_str() {
            "mic" => microphone_capture(enumerator, &track.id),
            "desktop" => desktop_capture(enumerator),
            "application" => track
                .process_id
                .ok_or_else(|| "Application process is unavailable".to_owned())
                .and_then(application_capture),
            _ => Err("Unknown audio source type".into()),
        }
    }

    pub fn capture_mix(
        tracks: Vec<AudioTrack>,
        queue: Arc<BoundedQueue<crate::AudioPcmBlock>>,
        capture_error_state: Arc<Mutex<Option<String>>>,
        stop: Arc<AtomicBool>,
        started: std::sync::mpsc::SyncSender<Result<(), String>>,
    ) {
        let _apartment = match ComApartment::initialize() {
            Ok(apartment) => apartment,
            Err(error) => {
                let message = error.clone();
                let _ = started.send(Err(message.clone()));
                set_capture_error(&capture_error_state, &message);
                queue.close();
                return;
            }
        };
        let enumerator = match enumerator() {
            Ok(enumerator) => enumerator,
            Err(message) => {
                let _ = started.send(Err(message.clone()));
                set_capture_error(&capture_error_state, &message);
                queue.close();
                return;
            }
        };
        let mut entries = Vec::with_capacity(tracks.len());
        for track in &tracks {
            match open_capture(&enumerator, track) {
                Ok(capture) => entries.push(MixerEntry {
                    capture,
                    resample_phase: 0,
                    pending: std::collections::VecDeque::new(),
                }),
                Err(message) => {
                    let _ = started.send(Err(message.clone()));
                    set_capture_error(&capture_error_state, &message);
                    queue.close();
                    for entry in entries {
                        let _ = unsafe { entry.capture.client.Stop() };
                    }
                    return;
                }
            }
        }
        let _ = started.send(Ok(()));
        let mut block_index = 0i64;
        while !stop.load(Ordering::Acquire) {
            for entry in &mut entries {
                let captured = match read_stereo_frames(&entry.capture) {
                    Ok(frames) => frames,
                    Err(message) => {
                        set_capture_error(&capture_error_state, &message);
                        stop.store(true, Ordering::Release);
                        break;
                    }
                };
                entry.pending.extend(resample_packet(
                    &captured,
                    entry.capture.sample_rate,
                    &mut entry.resample_phase,
                ));
                while entry.pending.len() > OUTPUT_SAMPLE_RATE as usize / 2 {
                    entry.pending.pop_front();
                }
            }
            if stop.load(Ordering::Acquire) {
                break;
            }
            if entries
                .iter()
                .all(|entry| entry.pending.len() >= AUDIO_BLOCK_FRAMES)
            {
                let mut samples = Vec::with_capacity(AUDIO_BLOCK_FRAMES * 2);
                for _ in 0..AUDIO_BLOCK_FRAMES {
                    let (mut left, mut right) = (0.0f32, 0.0f32);
                    for entry in &mut entries {
                        if let Some([l, r]) = entry.pending.pop_front() {
                            left += l;
                            right += r;
                        }
                    }
                    samples.push((left.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
                    samples.push((right.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
                }
                let timestamp_100ns = block_index * AUDIO_BLOCK_FRAMES as i64 * 10_000_000
                    / i64::from(OUTPUT_SAMPLE_RATE);
                block_index += 1;
                let _ = queue.push_latest(crate::AudioPcmBlock {
                    samples,
                    timestamp_100ns,
                });
            } else {
                thread::sleep(Duration::from_millis(2));
            }
        }
        for entry in entries {
            let _ = unsafe { entry.capture.client.Stop() };
        }
        queue.close();
    }

    fn set_capture_error(error: &Mutex<Option<String>>, message: &str) {
        if let Ok(mut current) = error.lock() {
            *current = Some(message.to_owned());
        }
    }

    fn read_stereo_frames(capture: &MicCapture) -> Result<Vec<[f32; 2]>, String> {
        let mut output = Vec::new();
        loop {
            let frames = unsafe { capture.capture.GetNextPacketSize() }
                .map_err(|error| format!("Could not read audio input: {error}"))?;
            if frames == 0 {
                return Ok(output);
            }
            let mut data = std::ptr::null_mut();
            let mut frame_count = 0;
            let mut flags = 0;
            unsafe {
                capture
                    .capture
                    .GetBuffer(&mut data, &mut frame_count, &mut flags, None, None)
                    .map_err(|error| format!("Could not read audio input: {error}"))?;
            }
            if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null() {
                output.resize(output.len() + frame_count as usize, [0.0; 2]);
            } else {
                let sample_bytes = sample_bytes(capture.format);
                for frame in 0..frame_count as usize {
                    let mut left = 0.0;
                    let mut right = 0.0;
                    let mut left_count = 0;
                    let mut right_count = 0;
                    for channel in 0..capture.channels {
                        let sample = unsafe {
                            data.add(frame * capture.block_align + channel * sample_bytes)
                        };
                        let value = unsafe { decode_sample(sample, capture.format) };
                        if channel % 2 == 0 {
                            left += value;
                            left_count += 1;
                        } else {
                            right += value;
                            right_count += 1;
                        }
                    }
                    if capture.channels == 1 {
                        right = left;
                        right_count = left_count;
                    }
                    output.push([
                        left / left_count.max(1) as f32,
                        right / right_count.max(1) as f32,
                    ]);
                }
            }
            unsafe { capture.capture.ReleaseBuffer(frame_count) }
                .map_err(|error| format!("Could not release audio buffer: {error}"))?;
        }
    }

    fn sample_bytes(format: SampleFormat) -> usize {
        match format {
            SampleFormat::Float32 | SampleFormat::Pcm32 => 4,
            SampleFormat::Pcm24 => 3,
            SampleFormat::Pcm16 => 2,
            SampleFormat::Pcm8 => 1,
        }
    }

    unsafe fn decode_sample(sample: *const u8, format: SampleFormat) -> f32 {
        let value = match format {
            SampleFormat::Float32 => unsafe { std::ptr::read_unaligned(sample.cast::<f32>()) },
            SampleFormat::Pcm8 => (unsafe { *sample }) as f32 / 128.0 - 1.0,
            SampleFormat::Pcm16 => {
                (unsafe { std::ptr::read_unaligned(sample.cast::<i16>()) }) as f32 / 32768.0
            }
            SampleFormat::Pcm24 => {
                let sign = if unsafe { *sample.add(2) } & 0x80 != 0 {
                    0xff
                } else {
                    0
                };
                i32::from_le_bytes([
                    unsafe { *sample },
                    unsafe { *sample.add(1) },
                    unsafe { *sample.add(2) },
                    sign,
                ]) as f32
                    / 8_388_608.0
            }
            SampleFormat::Pcm32 => {
                (unsafe { std::ptr::read_unaligned(sample.cast::<i32>()) }) as f32 / 2_147_483_648.0
            }
        };
        if value.is_finite() {
            value.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    fn resample_packet(input: &[[f32; 2]], input_rate: u32, phase: &mut u64) -> Vec<[f32; 2]> {
        if input.is_empty() || input_rate == 0 {
            return Vec::new();
        }
        let packet_end = input.len() as u64 * u64::from(OUTPUT_SAMPLE_RATE);
        let mut output = Vec::new();
        while *phase < packet_end {
            let left = (*phase / u64::from(OUTPUT_SAMPLE_RATE)) as usize;
            let right = (left + 1).min(input.len() - 1);
            let fraction =
                (*phase % u64::from(OUTPUT_SAMPLE_RATE)) as f32 / OUTPUT_SAMPLE_RATE as f32;
            output.push([
                input[left][0] * (1.0 - fraction) + input[right][0] * fraction,
                input[left][1] * (1.0 - fraction) + input[right][1] * fraction,
            ]);
            *phase += u64::from(input_rate);
        }
        *phase -= packet_end;
        output
    }

    fn read_peak(
        capture: &IAudioCaptureClient,
        format: SampleFormat,
        block_align: usize,
    ) -> Result<Option<f32>, String> {
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
                    if value.is_finite() {
                        value.abs().min(1.0)
                    } else {
                        0.0
                    }
                }
                SampleFormat::Pcm8 => (unsafe { *sample }) as f32 / 128.0 - 1.0,
                SampleFormat::Pcm16 => unsafe {
                    std::ptr::read_unaligned(sample.cast::<i16>()) as f32 / 32768.0
                },
                SampleFormat::Pcm24 => {
                    let sign = if unsafe { *sample.add(2) } & 0x80 != 0 {
                        0xff
                    } else {
                        0
                    };
                    let value = i32::from_le_bytes([
                        unsafe { *sample },
                        unsafe { *sample.add(1) },
                        unsafe { *sample.add(2) },
                        sign,
                    ]);
                    value as f32 / 8_388_608.0
                }
                SampleFormat::Pcm32 => unsafe {
                    std::ptr::read_unaligned(sample.cast::<i32>()) as f32 / 2_147_483_648.0
                },
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

    pub fn meter_sources(
        tracks: Vec<AudioTrack>,
        levels: Arc<Mutex<Vec<AudioInputLevel>>>,
        stop: Arc<AtomicBool>,
    ) {
        let ids = tracks
            .iter()
            .map(|track| track.id.clone())
            .collect::<Vec<_>>();
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
                    "desktop" => desktop_capture(&enumerator),
                    "application" => track
                        .process_id
                        .ok_or_else(|| "Application process is unavailable".to_owned())
                        .and_then(application_capture),
                    _ => Err("This audio source does not have an input meter".into()),
                };
                match capture {
                    Ok(capture) => MeterEntry {
                        id: track.id,
                        capture: Some(capture),
                        peak: 0.0,
                        error: None,
                    },
                    Err(error) => MeterEntry {
                        id: track.id,
                        capture: None,
                        peak: 0.0,
                        error: Some(error),
                    },
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

    #[cfg(test)]
    mod tests {
        use super::resample_packet;

        #[test]
        fn resampling_keeps_rate_across_packet_boundaries() {
            let input = vec![[0.25, -0.25]; 441];
            let mut phase = 0;
            let first = resample_packet(&input[..100], 44_100, &mut phase);
            let second = resample_packet(&input[100..], 44_100, &mut phase);

            assert_eq!(first.len() + second.len(), 480);
            assert!(
                first
                    .iter()
                    .chain(&second)
                    .all(|frame| *frame == [0.25, -0.25])
            );
        }
    }
}

pub fn start_meter(sources: &[DeviceSource]) -> Result<AudioMeter, crate::AudioError> {
    #[cfg(windows)]
    {
        let tracks = sources
            .iter()
            .map(|source| AudioTrack {
                id: source.id.clone(),
                kind: match source.kind {
                    AudioSourceKind::Microphone => "mic",
                    AudioSourceKind::Desktop => "desktop",
                    AudioSourceKind::Application => "application",
                }
                .into(),
                process_id: source.process_id,
            })
            .collect::<Vec<_>>();
        let stop = Arc::new(AtomicBool::new(false));
        let levels = Arc::new(Mutex::new(
            tracks
                .iter()
                .map(|track| AudioInputLevel {
                    id: track.id.clone(),
                    peak: None,
                    error: None,
                })
                .collect(),
        ));
        let worker_stop = Arc::clone(&stop);
        let worker_levels = Arc::clone(&levels);
        let thread = std::thread::Builder::new()
            .name("arca-audio-meter".into())
            .spawn(move || windows_audio::meter_sources(tracks, worker_levels, worker_stop))
            .map_err(|error| {
                crate::AudioError::Backend(format!("Could not start audio meter: {error}"))
            })?;
        Ok(AudioMeter {
            stop,
            levels,
            thread: Some(thread),
        })
    }
    #[cfg(not(windows))]
    {
        let _ = sources;
        Err(crate::AudioError::UnsupportedPlatform)
    }
}

pub fn start_capture_stream(
    sources: &[DeviceSource],
) -> Result<AudioCaptureStream, crate::AudioError> {
    #[cfg(windows)]
    {
        if sources.is_empty() {
            return Err(crate::AudioError::Backend(
                "Select at least one audio source before starting capture.".into(),
            ));
        }
        let tracks = sources
            .iter()
            .map(|source| AudioTrack {
                id: source.id.clone(),
                kind: match source.kind {
                    AudioSourceKind::Microphone => "mic",
                    AudioSourceKind::Desktop => "desktop",
                    AudioSourceKind::Application => "application",
                }
                .into(),
                process_id: source.process_id,
            })
            .collect::<Vec<_>>();
        let stop = Arc::new(AtomicBool::new(false));
        let queue = Arc::new(
            BoundedQueue::new(8).map_err(|error| crate::AudioError::Backend(error.into()))?,
        );
        let error = Arc::new(Mutex::new(None));
        let (started, ready) = std::sync::mpsc::sync_channel(1);
        let worker_stop = Arc::clone(&stop);
        let worker_queue = Arc::clone(&queue);
        let worker_error = Arc::clone(&error);
        let thread = std::thread::Builder::new()
            .name("arca-audio-capture".into())
            .spawn(move || {
                windows_audio::capture_mix(tracks, worker_queue, worker_error, worker_stop, started)
            })
            .map_err(|error| {
                crate::AudioError::Backend(format!("Could not start audio capture: {error}"))
            })?;
        match ready.recv_timeout(Duration::from_secs(7)) {
            Ok(Ok(())) => Ok(AudioCaptureStream {
                stop,
                queue,
                error,
                thread: Some(thread),
            }),
            Ok(Err(message)) => {
                stop.store(true, Ordering::Release);
                let _ = thread.join();
                Err(crate::AudioError::Backend(message))
            }
            Err(error) => {
                stop.store(true, Ordering::Release);
                queue.close();
                let _ = thread.join();
                Err(crate::AudioError::Backend(format!(
                    "Audio capture did not initialize: {error}"
                )))
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = sources;
        Err(crate::AudioError::UnsupportedPlatform)
    }
}

pub fn enumerate_microphones() -> Result<Vec<DeviceSource>, crate::AudioError> {
    discover("mic")
}
pub fn enumerate_desktop_audio() -> Result<Vec<DeviceSource>, crate::AudioError> {
    discover("desktop")
}
pub fn enumerate_active_applications() -> Result<Vec<DeviceSource>, crate::AudioError> {
    discover("application")
}
fn discover(kind: &str) -> Result<Vec<DeviceSource>, crate::AudioError> {
    #[cfg(windows)]
    {
        let kind = kind.to_owned();
        let sources = std::thread::Builder::new()
            .name("arca-audio-discovery".into())
            .spawn(move || windows_audio::list_sources(&kind))
            .map_err(|error| {
                crate::AudioError::Backend(format!("Could not start audio discovery: {error}"))
            })?
            .join()
            .map_err(|_| crate::AudioError::Backend("Audio discovery stopped unexpectedly".into()))?
            .map_err(crate::AudioError::Backend)?;
        Ok(sources
            .into_iter()
            .map(|source| DeviceSource {
                id: source.id,
                label: source.label,
                kind: match source.kind.as_str() {
                    "mic" => AudioSourceKind::Microphone,
                    "desktop" => AudioSourceKind::Desktop,
                    _ => AudioSourceKind::Application,
                },
                process_id: source.process_id,
            })
            .collect())
    }
    #[cfg(not(windows))]
    {
        let _ = kind;
        Err(crate::AudioError::UnsupportedPlatform)
    }
}
