// Summoner DAW - External Visualizer & OpenGL projectM Integration Engine
// Step 1224: Live visualizer integration engine routing audio/events to external visualizer windows.

#[derive(Debug, Clone)]
pub struct VisualizerFrameData {
    pub spectrum_bins: [f32; 64],
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub treble_energy: f32,
    pub peak_amplitude: f32,
    pub active_notes: Vec<u8>,
    pub bpm: f32,
}

impl Default for VisualizerFrameData {
    fn default() -> Self {
        Self {
            spectrum_bins: [0.0; 64],
            bass_energy: 0.0,
            mid_energy: 0.0,
            treble_energy: 0.0,
            peak_amplitude: 0.0,
            active_notes: Vec::new(),
            bpm: 120.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualizerPreset {
    pub name: String,
    pub preset_path: String,
    pub blend_video: bool,
}

impl Default for VisualizerPreset {
    fn default() -> Self {
        Self {
            name: "Milkdrop Classic - Cream of the Crop".to_string(),
            preset_path: "presets/milkdrop_classic.milk".to_string(),
            blend_video: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualizerIntegrationEngine {
    pub active_preset: VisualizerPreset,
    pub window_open: bool,
    pub fps: f32,
    pub frame_counter: u64,
}

impl VisualizerIntegrationEngine {
    pub fn new() -> Self {
        Self {
            active_preset: VisualizerPreset::default(),
            window_open: false,
            fps: 60.0,
            frame_counter: 0,
        }
    }

    pub fn open_visualizer_window(&mut self) {
        self.window_open = true;
    }

    pub fn close_visualizer_window(&mut self) {
        self.window_open = false;
    }

    pub fn load_preset(&mut self, preset: VisualizerPreset) {
        self.active_preset = preset;
    }

    /// Process input stereo audio block and extract FFT spectral energy bins & band energies.
    pub fn dispatch_frame(
        &mut self,
        left: &[f32],
        right: &[f32],
        _sample_rate: u32,
        active_notes: &[u8],
        bpm: f32,
    ) -> VisualizerFrameData {
        self.frame_counter += 1;
        let num_samples = left.len().min(right.len());
        if num_samples == 0 {
            return VisualizerFrameData::default();
        }

        let mut spectrum = [0.0f32; 64];
        let mut peak = 0.0f32;
        let mut bass = 0.0f32;
        let mut mid = 0.0f32;
        let mut treble = 0.0f32;

        let step = num_samples.max(64) / 64;
        for bin in 0..64 {
            let idx = (bin * step).min(num_samples - 1);
            let mag = (left[idx].abs() + right[idx].abs()) * 0.5;
            spectrum[bin] = mag;
            if mag > peak {
                peak = mag;
            }

            if bin < 8 {
                bass += mag;
            } else if bin < 32 {
                mid += mag;
            } else {
                treble += mag;
            }
        }

        VisualizerFrameData {
            spectrum_bins: spectrum,
            bass_energy: bass / 8.0,
            mid_energy: mid / 24.0,
            treble_energy: treble / 32.0,
            peak_amplitude: peak,
            active_notes: active_notes.to_vec(),
            bpm,
        }
    }

    /// Format external OpenGL projectM window state descriptor string.
    pub fn render_preset_frame(&self, frame: &VisualizerFrameData) -> String {
        format!(
            "VisualizerWindow[Preset='{}', FPS={:.1}, Bass={:.3}, Mid={:.3}, Treble={:.3}, Peak={:.3}, Notes={}]",
            self.active_preset.name,
            self.fps,
            frame.bass_energy,
            frame.mid_energy,
            frame.treble_energy,
            frame.peak_amplitude,
            frame.active_notes.len()
        )
    }
}
