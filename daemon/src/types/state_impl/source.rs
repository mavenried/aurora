use rodio::{Player, Source};
use std::fs::File;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

// Player is connected to Rodio's mixer once, so its advertised sample rate
// must not change when the queued track changes.
const PLAYBACK_SAMPLE_RATE: u32 = 48_000;
const PLAYBACK_CHANNELS: usize = 2;

pub struct AudioBuffer {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    fully_loaded: bool,
    format_ready: bool,
    duration: Duration,
}

impl AudioBuffer {
    fn new(sample_rate: u32, channels: u16, duration: Duration) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            duration,
            fully_loaded: false,
            format_ready: false,
            channels,
        }
    }

    fn push_samples(&mut self, new: &[f32]) {
        self.samples.extend_from_slice(new);
    }

    fn sample_index_from_time(&self, pos: Duration) -> usize {
        let frame = (pos.as_secs_f64() * PLAYBACK_SAMPLE_RATE as f64) as usize;
        frame * PLAYBACK_CHANNELS
    }
}

pub struct SeekableAudio {
    sink: Arc<Player>,
    buffer: Arc<Mutex<AudioBuffer>>,
    source: Option<BufferSource>,
    cancelled: Arc<AtomicBool>,
}

impl SeekableAudio {
    pub fn new(path: &Path, sink: Arc<Player>) -> Result<Self, Box<dyn std::error::Error>> {
        let (sample_rate, channels, duration) = Self::read_metadata(path)?;
        let buffer = Arc::new(Mutex::new(AudioBuffer::new(
            sample_rate,
            channels,
            duration,
        )));
        let cancelled = Arc::new(AtomicBool::new(false));
        let format_ready = Arc::new(Condvar::new());

        {
            let path = path.to_owned();
            let buffer_clone = buffer.clone();
            let cancelled_clone = cancelled.clone();
            let format_ready_clone = format_ready.clone();
            std::thread::spawn(move || {
                decode_worker(path, buffer_clone, cancelled_clone, format_ready_clone)
            });
        }

        // Rodio derives its playback conversion from the Source format when it
        // starts consuming it. Wait until Symphonia has decoded a packet and
        // established the actual output format before appending the source.
        let mut state = buffer.lock().unwrap();
        while !state.format_ready && !state.fully_loaded {
            state = format_ready.wait(state).unwrap();
        }
        drop(state);

        let buf = BufferSource::new(buffer.clone(), 0);
        sink.append(buf.clone());
        sink.pause();

        Ok(Self {
            sink,
            buffer,
            source: Some(buf),
            cancelled,
        })
    }

    fn read_metadata(path: &Path) -> Result<(u32, u16, Duration), Box<dyn std::error::Error>> {
        let file = Box::new(File::open(path)?);
        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = std::path::Path::new(path).extension() {
            hint.with_extension(ext.to_str().unwrap_or(""));
        }

        let reader = symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let track = reader
            .tracks()
            .iter()
            .find(|track| {
                track
                    .codec_params
                    .as_ref()
                    .and_then(|params| params.audio())
                    .is_some_and(|params| {
                        params.codec != symphonia::core::codecs::audio::CODEC_ID_NULL_AUDIO
                    })
            })
            .ok_or("No track")?;

        let params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .ok_or("Not an audio track")?;
        let rate = params.sample_rate.ok_or("Missing sample rate")?;
        let channels = params.channels.as_ref().ok_or("Missing channels")?.count() as u16;

        let mut duration = Duration::ZERO;
        if let Some(tb) = track.time_base
            && let Some(frames) = track.num_frames
        {
            if let Some(time) = tb.calc_time(symphonia::core::units::Timestamp::new(frames as i64))
            {
                duration = Duration::from_secs_f64(time.as_secs_f64());
            }
        }

        Ok((rate, channels, duration))
    }

    fn decode_streaming(
        path: PathBuf,
        buffer: Arc<Mutex<AudioBuffer>>,
        cancelled: Arc<AtomicBool>,
        format_ready: Arc<Condvar>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = Box::new(File::open(&path)?);
        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension() {
            hint.with_extension(ext.to_str().unwrap_or(""));
        }

        let mut reader = symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let track = reader
            .tracks()
            .iter()
            .find(|track| {
                track
                    .codec_params
                    .as_ref()
                    .and_then(|params| params.audio())
                    .is_some_and(|params| {
                        params.codec != symphonia::core::codecs::audio::CODEC_ID_NULL_AUDIO
                    })
            })
            .ok_or("No audio track")?;

        let codec_params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .ok_or("Not an audio track")?;
        let mut decoder = symphonia::default::get_codecs().make_audio_decoder(
            codec_params,
            &symphonia::core::codecs::audio::AudioDecoderOptions::default(),
        )?;

        let track_id = track.id;

        loop {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            let packet = match reader.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => break,
                Err(Error::IoError(_)) => break,
                Err(e) => return Err(e.into()),
            };

            if packet.track_id != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Codec parameters describe the stream, but the decoder's
                    // output spec is authoritative (some formats can update
                    // their parameters after probing). The Source rate is what
                    // Rodio uses to determine playback speed, so stale probe
                    // metadata here changes both pitch and duration.
                    let spec = decoded.spec();
                    let decoded_rate = spec.rate();
                    let decoded_channels = spec.channels().count() as u16;
                    {
                        let mut buffer = buffer.lock().unwrap();
                        if !buffer.format_ready {
                            buffer.sample_rate = decoded_rate;
                            buffer.channels = decoded_channels;
                            buffer.format_ready = true;
                        } else if buffer.sample_rate != decoded_rate
                            || buffer.channels != decoded_channels
                        {
                            return Err(format!(
                                "Audio format changed while decoding ({} Hz/{} channels to {} Hz/{} channels)",
                                buffer.sample_rate, buffer.channels, decoded_rate, decoded_channels
                            ).into());
                        }
                    }
                    format_ready.notify_all();
                    let mut samples = Vec::with_capacity(decoded.samples_interleaved());
                    decoded.copy_to_vec_interleaved(&mut samples);
                    buffer.lock().unwrap().push_samples(&samples);
                }

                Err(Error::DecodeError(_)) => continue,

                Err(Error::IoError(_)) => break,
                Err(e) => return Err(e.into()),
            }
        }

        buffer.lock().unwrap().fully_loaded = true;
        format_ready.notify_all();
        Ok(())
    }

    pub fn get_position(&self) -> Duration {
        if let Some(s) = &self.source {
            s.current_position()
        } else {
            Duration::ZERO
        }
    }

    pub fn seek(&mut self, pos: Duration) {
        let idx = {
            let buf = self.buffer.lock().unwrap();
            buf.sample_index_from_time(pos)
        };

        let was_paused = self.sink.is_paused();

        self.sink.stop();
        let source = BufferSource::new(self.buffer.clone(), idx);
        self.source = Some(source.clone());
        self.sink.append(source);

        if !was_paused {
            self.sink.play();
        }
    }
}

impl Drop for SeekableAudio {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

fn decode_worker(
    path: PathBuf,
    buffer: Arc<Mutex<AudioBuffer>>,
    cancelled: Arc<AtomicBool>,
    format_ready: Arc<Condvar>,
) {
    if let Err(err) = SeekableAudio::decode_streaming(
        path.clone(),
        buffer.clone(),
        cancelled,
        format_ready.clone(),
    ) {
        tracing::error!("Failed to decode audio file {path:?}: {err}");
        if let Ok(mut buffer) = buffer.lock() {
            buffer.fully_loaded = true;
        }
        format_ready.notify_all();
    }
}

#[derive(Clone)]
struct BufferSource {
    buffer: Arc<Mutex<AudioBuffer>>,
    position: Arc<AtomicUsize>,
}

impl BufferSource {
    fn new(buffer: Arc<Mutex<AudioBuffer>>, start_pos: usize) -> Self {
        Self {
            buffer,
            position: Arc::new(AtomicUsize::from(start_pos)),
        }
    }
    pub fn current_position(&self) -> Duration {
        let idx = self.position.load(Ordering::Relaxed);
        let buf = self.buffer.lock().unwrap();
        let frame = idx / PLAYBACK_CHANNELS;
        Duration::from_secs_f64(frame as f64 / PLAYBACK_SAMPLE_RATE as f64)
    }
}

impl Iterator for BufferSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let buf = self.buffer.lock().unwrap();

            let output_index = self.position.load(Ordering::Relaxed);
            let output_frame = output_index / PLAYBACK_CHANNELS;
            let output_channel = output_index % PLAYBACK_CHANNELS;
            let input_channels = buf.channels as usize;
            let channel = output_channel.min(input_channels.saturating_sub(1));
            let input_position =
                output_frame as f64 * buf.sample_rate as f64 / PLAYBACK_SAMPLE_RATE as f64;
            let input_frame = input_position.floor() as usize;
            let fraction = input_position - input_frame as f64;
            let input_index = input_frame * input_channels + channel;
            let next_index = (input_frame + 1) * input_channels + channel;

            if input_index < buf.samples.len()
                && (fraction == 0.0 || next_index < buf.samples.len() || buf.fully_loaded)
            {
                let first = buf.samples[input_index];
                let sample = if fraction == 0.0 || next_index >= buf.samples.len() {
                    first
                } else {
                    let second = buf.samples[next_index];
                    first + (second - first) * fraction as f32
                };
                self.position.fetch_add(1, Ordering::Relaxed);
                return Some(sample);
            }

            if buf.fully_loaded && input_index >= buf.samples.len() {
                return None;
            }

            drop(buf);
            std::thread::sleep(Duration::from_millis(3));
        }
    }
}

impl Source for BufferSource {
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(PLAYBACK_CHANNELS as u16).unwrap_or(NonZero::<u16>::MIN)
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(PLAYBACK_SAMPLE_RATE).unwrap_or(NonZero::<u32>::MIN)
    }
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(self.buffer.lock().unwrap().duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_to_sample_index_accounts_for_rate_and_channels() {
        let buffer = AudioBuffer::new(48_000, 2, Duration::from_secs(10));
        assert_eq!(
            buffer.sample_index_from_time(Duration::from_secs(1)),
            96_000
        );
        assert_eq!(
            buffer.sample_index_from_time(Duration::from_millis(500)),
            48_000
        );
    }

    #[test]
    fn time_to_sample_index_starts_at_zero() {
        let buffer = AudioBuffer::new(44_100, 1, Duration::from_secs(1));
        assert_eq!(buffer.sample_index_from_time(Duration::ZERO), 0);
    }

    #[test]
    fn decoder_failure_marks_audio_buffer_complete() {
        let buffer = Arc::new(Mutex::new(AudioBuffer::new(
            44_100,
            2,
            Duration::from_secs(1),
        )));
        let path = std::env::temp_dir().join(format!("missing-audio-{}", uuid::Uuid::new_v4()));

        decode_worker(
            path,
            buffer.clone(),
            Arc::new(AtomicBool::new(false)),
            Arc::new(Condvar::new()),
        );

        assert!(buffer.lock().unwrap().fully_loaded);
    }
}
