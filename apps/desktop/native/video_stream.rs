use arca_capture::windows::CaptureFrames;
use arca_compose::windows::GpuFrameConverter;
use arca_encode::{H264Settings, HardwareAacEncoder, HardwareH264Encoder};
use arca_publish::{
    RtmpPublisher, aac_frame, aac_sequence_header, avc_frame_from_annex_b,
    avc_sequence_header_from_annex_b,
};
use bytes::Bytes;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

enum VideoMessage {
    VideoTag(Bytes, u32),
    AudioTag(Bytes, u32),
}

pub struct VideoStreamSession {
    stop: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    frames: Arc<CaptureFrames>,
    _encoder: JoinHandle<()>,
    _audio_encoder: Option<JoinHandle<()>>,
    _network: JoinHandle<()>,
}

impl VideoStreamSession {
    pub fn start(
        frames: Arc<CaptureFrames>,
        ingest_url: String,
        stream_key: String,
        settings: H264Settings,
        audio_sources: Vec<arca_audio::AudioSource>,
    ) -> Result<Self, String> {
        if stream_key.trim().is_empty() {
            return Err("Add a stream key in Settings before going live.".into());
        }
        let stop = Arc::new(AtomicBool::new(false));
        let failed = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new("Connecting to ingest…".into()));
        let (sender, receiver) = mpsc::sync_channel(3);
        let audio_capture = if audio_sources.is_empty() {
            None
        } else {
            Some(
                arca_audio::wasapi::start_capture_stream(&audio_sources)
                    .map_err(|error| error.to_string())?,
            )
        };

        let network_status = Arc::clone(&status);
        let network_failed = Arc::clone(&failed);
        let network_stop = Arc::clone(&stop);
        let audio_enabled = audio_capture.is_some();
        let network = thread::Builder::new()
            .name("arca-rtmps-publish".into())
            .spawn(move || {
                publish(
                    ingest_url,
                    stream_key,
                    receiver,
                    network_status,
                    network_failed,
                    network_stop,
                    audio_enabled,
                )
            })
            .map_err(|error| format!("Could not start the RTMPS worker: {error}"))?;

        let audio_encoder = if let Some(capture) = audio_capture {
            let audio_sender = sender.clone();
            let audio_stop = Arc::clone(&stop);
            let audio_failed = Arc::clone(&failed);
            let audio_status = Arc::clone(&status);
            match thread::Builder::new()
                .name("arca-audio-encode".into())
                .spawn(move || {
                    encode_audio(
                        capture,
                        audio_sender,
                        audio_stop,
                        audio_status,
                        audio_failed,
                    )
                }) {
                Ok(worker) => Some(worker),
                Err(error) => {
                    stop.store(true, Ordering::Release);
                    return Err(format!("Could not start the audio encoder worker: {error}"));
                }
            }
        } else {
            None
        };

        let encoder_stop = Arc::clone(&stop);
        let encoder_failed = Arc::clone(&failed);
        let encoder_status = Arc::clone(&status);
        let encoder_frames = Arc::clone(&frames);
        frames.set_active(true);
        let encoder = match thread::Builder::new()
            .name("arca-video-encode".into())
            .spawn(move || {
                encode(
                    encoder_frames,
                    sender,
                    encoder_stop,
                    encoder_status,
                    encoder_failed,
                    settings,
                )
            }) {
            Ok(encoder) => encoder,
            Err(error) => {
                frames.set_active(false);
                stop.store(true, Ordering::Release);
                return Err(format!("Could not start the video encoder worker: {error}"));
            }
        };

        Ok(Self {
            stop,
            failed,
            status,
            frames,
            _encoder: encoder,
            _audio_encoder: audio_encoder,
            _network: network,
        })
    }

    pub fn status(&self) -> String {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| "Stream status unavailable".into())
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
        self.frames.set_active(false);
    }

    pub fn is_failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}

impl Drop for VideoStreamSession {
    fn drop(&mut self) {
        self.stop();
    }
}

fn publish(
    ingest_url: String,
    stream_key: String,
    receiver: Receiver<VideoMessage>,
    status: Arc<Mutex<String>>,
    failed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    audio_enabled: bool,
) {
    let mut publisher = match RtmpPublisher::connect(&ingest_url, &stream_key) {
        Ok(publisher) => publisher,
        Err(error) => {
            failed.store(true, Ordering::Release);
            set_status(&status, format!("Could not connect: {error}"));
            return;
        }
    };
    if failed.load(Ordering::Acquire) {
        let _ = publisher.stop();
        return;
    }
    if stop.load(Ordering::Acquire) {
        let _ = publisher.stop();
        set_status(&status, "Offline".into());
        return;
    }
    set_status(
        &status,
        if audio_enabled {
            "Live · audio and video".into()
        } else {
            "Live · video only".into()
        },
    );

    for message in receiver {
        if stop.load(Ordering::Acquire) {
            break;
        }
        let result = match message {
            VideoMessage::VideoTag(payload, timestamp_ms) => {
                publisher.publish_video_tag(payload, timestamp_ms)
            }
            VideoMessage::AudioTag(payload, timestamp_ms) => {
                publisher.publish_audio_tag(payload, timestamp_ms)
            }
        };
        if let Err(error) = result {
            failed.store(true, Ordering::Release);
            set_status(&status, format!("Publishing stopped: {error}"));
            return;
        }
    }

    if let Err(error) = publisher.stop() {
        failed.store(true, Ordering::Release);
        set_status(&status, format!("Stream stopped with an error: {error}"));
    } else if !failed.load(Ordering::Acquire) {
        set_status(&status, "Offline".into());
    }
}

fn encode(
    frames: Arc<CaptureFrames>,
    sender: SyncSender<VideoMessage>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    failed: Arc<AtomicBool>,
    settings: H264Settings,
) {
    struct ActiveFrames(Arc<CaptureFrames>);
    impl Drop for ActiveFrames {
        fn drop(&mut self) {
            self.0.set_active(false);
        }
    }
    let _active = ActiveFrames(Arc::clone(&frames));
    let mut converter: Option<GpuFrameConverter> = None;
    let mut encoder: Option<HardwareH264Encoder> = None;
    let mut first_sample_time = None;
    let mut waiting_for_keyframe = true;

    while !stop.load(Ordering::Acquire) {
        let Some(frame) = frames.next(Duration::from_millis(50)) else {
            continue;
        };

        if converter.is_none() {
            let mut encoder_converter = match GpuFrameConverter::new(
                &frame.texture,
                settings.width,
                settings.height,
                settings.frame_rate,
            ) {
                Ok(converter) => converter,
                Err(error) => {
                    set_failure(&status, &failed, format!("GPU video setup failed: {error}"));
                    break;
                }
            };
            let nv12 = match encoder_converter.convert(&frame.texture) {
                Ok(texture) => texture,
                Err(error) => {
                    set_failure(
                        &status,
                        &failed,
                        format!("GPU frame conversion failed: {error}"),
                    );
                    break;
                }
            };
            let device = match unsafe { nv12.GetDevice() } {
                Ok(device) => device,
                Err(error) => {
                    set_failure(
                        &status,
                        &failed,
                        format!("Could not access the capture GPU: {error}"),
                    );
                    break;
                }
            };
            let hardware_encoder = match HardwareH264Encoder::new(&device, settings) {
                Ok(encoder) => encoder,
                Err(error) => {
                    set_failure(
                        &status,
                        &failed,
                        format!("Hardware encoder setup failed: {error}"),
                    );
                    break;
                }
            };
            let header = match avc_sequence_header_from_annex_b(hardware_encoder.sequence_header())
            {
                Ok(header) => header,
                Err(error) => {
                    set_failure(
                        &status,
                        &failed,
                        format!("H.264 stream setup failed: {error}"),
                    );
                    break;
                }
            };
            if sender.send(VideoMessage::VideoTag(header, 0)).is_err() {
                if !failed.load(Ordering::Acquire) {
                    set_failure(&status, &failed, "The RTMPS worker stopped.".into());
                }
                break;
            }
            converter = Some(encoder_converter);
            encoder = Some(hardware_encoder);
        }

        let converted = match converter.as_mut().unwrap().convert(&frame.texture) {
            Ok(texture) => texture,
            Err(error) => {
                set_failure(
                    &status,
                    &failed,
                    format!("GPU frame conversion failed: {error}"),
                );
                break;
            }
        };
        let samples = match encoder.as_mut().unwrap().encode(
            &converted,
            frame.sequence as i64 * 10_000_000 / i64::from(settings.frame_rate),
        ) {
            Ok(samples) => samples,
            Err(error) => {
                set_failure(
                    &status,
                    &failed,
                    format!("Hardware encoding failed: {error}"),
                );
                break;
            }
        };

        for sample in samples {
            if waiting_for_keyframe && !sample.keyframe {
                continue;
            }
            waiting_for_keyframe = false;
            let origin = *first_sample_time.get_or_insert(sample.timestamp_100ns);
            let timestamp_ms = sample
                .timestamp_100ns
                .saturating_sub(origin)
                .max(0)
                .saturating_div(10_000)
                .min(i64::from(u32::MAX)) as u32;
            let payload = match avc_frame_from_annex_b(&sample.bytes, sample.keyframe) {
                Ok(payload) => payload,
                Err(error) => {
                    set_failure(
                        &status,
                        &failed,
                        format!("Could not mux an encoded frame: {error}"),
                    );
                    return;
                }
            };
            if sender
                .send(VideoMessage::VideoTag(payload, timestamp_ms))
                .is_err()
            {
                if !failed.load(Ordering::Acquire) {
                    set_failure(&status, &failed, "The RTMPS worker stopped.".into());
                }
                return;
            }
        }
    }
}

fn encode_audio(
    capture: arca_audio::wasapi::AudioCaptureStream,
    sender: SyncSender<VideoMessage>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    failed: Arc<AtomicBool>,
) {
    let mut encoder = match HardwareAacEncoder::new() {
        Ok(encoder) => encoder,
        Err(error) => {
            set_failure(
                &status,
                &failed,
                format!("AAC encoder setup failed: {error}"),
            );
            stop.store(true, Ordering::Release);
            return;
        }
    };
    match aac_sequence_header(encoder.sequence_header()) {
        Ok(header) => {
            if sender.send(VideoMessage::AudioTag(header, 0)).is_err() {
                set_failure(&status, &failed, "The RTMPS worker stopped.".into());
                stop.store(true, Ordering::Release);
                return;
            }
        }
        Err(error) => {
            set_failure(
                &status,
                &failed,
                format!("AAC stream setup failed: {error}"),
            );
            stop.store(true, Ordering::Release);
            return;
        }
    }
    while !stop.load(Ordering::Acquire) {
        let Some(block) = capture.pop_timeout(Duration::from_millis(50)) else {
            if let Some(error) = capture.error() {
                set_failure(&status, &failed, format!("Audio capture stopped: {error}"));
                stop.store(true, Ordering::Release);
                return;
            }
            continue;
        };
        let samples = match encoder.encode_at(&block.samples, block.timestamp_100ns) {
            Ok(samples) => samples,
            Err(error) => {
                set_failure(&status, &failed, format!("AAC encoding failed: {error}"));
                stop.store(true, Ordering::Release);
                return;
            }
        };
        for sample in samples {
            let payload = match aac_frame(&sample.bytes) {
                Ok(payload) => payload,
                Err(error) => {
                    set_failure(&status, &failed, format!("Could not mux audio: {error}"));
                    stop.store(true, Ordering::Release);
                    return;
                }
            };
            let timestamp_ms = sample
                .timestamp_100ns
                .max(0)
                .saturating_div(10_000)
                .min(i64::from(u32::MAX)) as u32;
            if sender
                .send(VideoMessage::AudioTag(payload, timestamp_ms))
                .is_err()
            {
                if !failed.load(Ordering::Acquire) {
                    set_failure(&status, &failed, "The RTMPS worker stopped.".into());
                }
                stop.store(true, Ordering::Release);
                return;
            }
        }
    }
}

fn set_status(status: &Mutex<String>, value: String) {
    if let Ok(mut current) = status.lock() {
        *current = value;
    }
}

fn set_failure(status: &Mutex<String>, failed: &AtomicBool, value: String) {
    failed.store(true, Ordering::Release);
    set_status(status, value);
}
