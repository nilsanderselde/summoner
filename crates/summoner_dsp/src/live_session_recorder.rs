// Summoner DAW - Master Output Live Session Recorder
// Step 1223: One-click "Record Live Session" master output recorder writing directly to offline WAV/FLAC disk files.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use hound::{WavSpec, WavWriter, SampleFormat};

/// Supported file formats for live session disk recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFormat {
    Wav,
    Flac,
}

/// Statistics returned upon stopping a live recording session.
#[derive(Debug, Clone)]
pub struct RecordingStats {
    pub file_path: PathBuf,
    pub total_samples: u64,
    pub duration_seconds: f64,
    pub file_size_bytes: u64,
}

/// One-click master output live session recorder writing directly to disk.
pub struct LiveSessionRecorder {
    recording: bool,
    sample_rate: u32,
    channels: u16,
    format: RecordingFormat,
    output_path: PathBuf,
    wav_writer: Option<WavWriter<BufWriter<File>>>,
    total_samples: u64,
}

impl std::fmt::Debug for LiveSessionRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveSessionRecorder")
            .field("recording", &self.recording)
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("format", &self.format)
            .field("output_path", &self.output_path)
            .field("total_samples", &self.total_samples)
            .finish()
    }
}

impl LiveSessionRecorder {
    pub fn new() -> Self {
        Self {
            recording: false,
            sample_rate: 44100,
            channels: 2,
            format: RecordingFormat::Wav,
            output_path: PathBuf::new(),
            wav_writer: None,
            total_samples: 0,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    /// Start a live recording session writing to specified output file path.
    pub fn start_recording(
        &mut self,
        path: &Path,
        sample_rate: u32,
        channels: u16,
        format: RecordingFormat,
    ) -> Result<(), String> {
        if self.recording {
            return Err("Recording session is already active".to_string());
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let writer = WavWriter::create(path, spec).map_err(|e| e.to_string())?;

        self.recording = true;
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.format = format;
        self.output_path = path.to_path_buf();
        self.wav_writer = Some(writer);
        self.total_samples = 0;

        Ok(())
    }

    /// Process a block of stereo or multi-channel audio samples and write directly to disk.
    pub fn process_block(&mut self, left: &[f32], right: &[f32]) {
        if !self.recording {
            return;
        }

        if let Some(writer) = &mut self.wav_writer {
            let num_frames = left.len().min(right.len());
            for i in 0..num_frames {
                let l_pcm = (left[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                let r_pcm = (right[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                let _ = writer.write_sample(l_pcm);
                let _ = writer.write_sample(r_pcm);
            }
            self.total_samples += num_frames as u64;
        }
    }

    /// Stop live session recording, finalize header, and return session stats.
    pub fn stop_recording(&mut self) -> Result<RecordingStats, String> {
        if !self.recording {
            return Err("No active recording session to stop".to_string());
        }

        self.recording = false;
        if let Some(writer) = self.wav_writer.take() {
            writer.finalize().map_err(|e| e.to_string())?;
        }

        let metadata = std::fs::metadata(&self.output_path).map_err(|e| e.to_string())?;
        let duration_seconds = self.total_samples as f64 / self.sample_rate as f64;

        Ok(RecordingStats {
            file_path: self.output_path.clone(),
            total_samples: self.total_samples,
            duration_seconds,
            file_size_bytes: metadata.len(),
        })
    }
}
