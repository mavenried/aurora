use rodio::{Sink, Source};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioBuffer {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    fully_loaded: bool,
    duration: Duration,
}

impl AudioBuffer {
    fn new(sample_rate: u32, channels: u16, duration: Duration) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            duration,
            fully_loaded: false,
            channels: channels,
        }
    }

    fn push_samples(&mut self, new: &[f32]) {
        self.samples.extend_from_slice(new);
    }

    fn sample_index_from_time(&self, pos: Duration) -> usize {
        let frame = (pos.as_secs_f64() * self.sample_rate as f64) as usize;
        frame * self.channels as usize
    }
}

pub struct SeekableAudio {
    sink: Arc<Sink>,
    buffer: Arc<Mutex<AudioBuffer>>,
    source: Option<BufferSource>,
}

impl SeekableAudio {
    pub fn new(path: &Path, sink: Arc<Sink>) -> Result<Self, Box<dyn std::error::Error>> {
        let (sample_rate, channels, duration) = Self::read_metadata(path)?;
        let buffer = Arc::new(Mutex::new(AudioBuffer::new(
            sample_rate,
            channels,
            duration,
        )));

        {
            let path = path.to_owned();
            let buffer_clone = buffer.clone();
            std::thread::spawn(move || {
                let _ = Self::decode_streaming(path, buffer_clone);
            });
        }
        let buf = BufferSource::new(buffer.clone(), 0);
        sink.append(buf.clone());
        sink.pause();

        Ok(Self {
            sink,
            buffer,
            source: Some(buf),
        })
    }

    fn read_metadata(path: &Path) -> Result<(u32, u16, Duration), Box<dyn std::error::Error>> {
        let file = Box::new(File::open(path)?);
        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = std::path::Path::new(path).extension() {
            hint.with_extension(ext.to_str().unwrap_or(""));
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let reader = probed.format;

        let track = reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No track")?;

        let rate = track
            .codec_params
            .sample_rate
            .ok_or("Missing sample rate")?;
        let channels = track
            .codec_params
            .channels
            .ok_or("Missing channels")?
            .count() as u16;

        let mut duration = Duration::ZERO;
        if let Some(tb) = track.codec_params.time_base {
            if let Some(frames) = track.codec_params.n_frames {
                duration = tb.calc_time(frames).into();
            }
        }

        Ok((rate, channels, duration))
    }

    fn decode_streaming(
        path: PathBuf,
        buffer: Arc<Mutex<AudioBuffer>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = Box::new(File::open(&path)?);
        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension() {
            hint.with_extension(ext.to_str().unwrap_or(""));
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let mut reader = probed.format;

        let track = reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No audio track")?;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;

        let track_id = track.id;

        loop {
            let packet = match reader.next_packet() {
                Ok(p) => p,
                Err(Error::IoError(_)) => break,
                Err(e) => return Err(e.into()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                    buf.copy_interleaved_ref(decoded);

                    buffer.lock().unwrap().push_samples(buf.samples());
                }

                Err(Error::DecodeError(_)) => continue,

                Err(Error::IoError(_)) => break,
                Err(e) => return Err(e.into()),
            }
        }

        buffer.lock().unwrap().fully_loaded = true;
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
        let frame = idx / buf.channels as usize;
        Duration::from_secs_f64(frame as f64 / buf.sample_rate as f64)
    }
}

impl Iterator for BufferSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let buf = self.buffer.lock().unwrap();

            if self.position.load(Ordering::Relaxed) < buf.samples.len() {
                let s = buf.samples[self.position.load(Ordering::Relaxed)];
                self.position.fetch_add(1, Ordering::Relaxed);
                return Some(s);
            }

            if buf.fully_loaded {
                return None;
            }

            drop(buf);
            std::thread::sleep(Duration::from_millis(3));
        }
    }
}

impl Source for BufferSource {
    fn channels(&self) -> u16 {
        self.buffer.lock().unwrap().channels
    }
    fn sample_rate(&self) -> u32 {
        self.buffer.lock().unwrap().sample_rate
    }
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(self.buffer.lock().unwrap().duration)
    }
}
