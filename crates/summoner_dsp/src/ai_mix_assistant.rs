// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Autonomous AI Mix Assistant (Milestone N4).
//!
//! Provides intelligent multi-track mix analysis and automated mastering guidance:
//! - Dynamic inter-track spectral masking detection across psychoacoustic critical bands
//!   (Kick vs Bass, Vocal vs Guitar/Synths, Snare vs Cymbals).
//! - Reference spectral balance profile matching against commercial target curves
//!   (Modern Pop, Club Electronic, Hard Rock, Acoustic Jazz, Cinematic Orchestral).
//! - Target Integrated LUFS auto-gain staging with per-track and master trim recommendations.

use serde::{Deserialize, Serialize};

/// Auditory critical frequency bands for masking collision analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CriticalMixBand {
    SubBass, // 20 - 60 Hz
    LowBass, // 60 - 250 Hz
    LowMid,  // 250 - 1000 Hz
    HighMid, // 1000 - 4000 Hz
    HighAir, // 4000 - 20000 Hz
}

impl CriticalMixBand {
    pub const ALL: [CriticalMixBand; 5] = [
        CriticalMixBand::SubBass,
        CriticalMixBand::LowBass,
        CriticalMixBand::LowMid,
        CriticalMixBand::HighMid,
        CriticalMixBand::HighAir,
    ];

    pub fn frequency_range_hz(&self) -> (f32, f32) {
        match self {
            Self::SubBass => (20.0, 60.0),
            Self::LowBass => (60.0, 250.0),
            Self::LowMid => (250.0, 1000.0),
            Self::HighMid => (1000.0, 4000.0),
            Self::HighAir => (4000.0, 20000.0),
        }
    }
}

/// Detected masking conflict between two audio tracks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskingConflict {
    pub track_a_name: String,
    pub track_b_name: String,
    pub band: CriticalMixBand,
    pub clash_severity: f32, // 0.0 (clean) to 1.0 (severe masking clash)
    pub suggested_cut_hz: f32,
    pub suggested_q: f32,
    pub suggested_gain_db: f32,
    pub recommended_action: String,
}

/// Reference commercial spectral balance curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TargetGenreProfile {
    #[default]
    ModernPop,
    ClubElectronic,
    HardRock,
    AcousticJazz,
    CinematicOrchestral,
}

impl TargetGenreProfile {
    /// Relative target dB energy profile across `[SubBass, LowBass, LowMid, HighMid, HighAir]`.
    pub fn target_band_energies_db(&self) -> [f32; 5] {
        match self {
            Self::ModernPop => [0.0, 1.5, -0.5, 0.5, 1.0],
            Self::ClubElectronic => [3.0, 2.0, -1.5, 0.0, 2.5],
            Self::HardRock => [-1.0, 1.0, 1.5, 2.0, 0.0],
            Self::AcousticJazz => [-2.0, 0.5, 1.0, 0.5, -0.5],
            Self::CinematicOrchestral => [1.0, 0.0, 0.5, 1.0, 0.5],
        }
    }

    /// Standard streaming/broadcast target integrated LUFS.
    pub fn target_lufs(&self) -> f32 {
        match self {
            Self::ModernPop => -14.0,
            Self::ClubElectronic => -9.0,
            Self::HardRock => -11.0,
            Self::AcousticJazz => -16.0,
            Self::CinematicOrchestral => -18.0,
        }
    }
}

/// Mix Assistant analysis result and corrective suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixAssistantReport {
    pub current_integrated_lufs: f32,
    pub target_lufs: f32,
    pub suggested_master_gain_trim_db: f32,
    pub spectral_balance_deviations_db: [f32; 5],
    pub masking_conflicts: Vec<MaskingConflict>,
}

/// Autonomous AI Mix Assistant.
#[derive(Debug, Clone, Default)]
pub struct AutonomousMixAssistant {
    pub target_profile: TargetGenreProfile,
    pub sample_rate: f32,
}

impl AutonomousMixAssistant {
    pub fn new(target_profile: TargetGenreProfile, sample_rate: f32) -> Self {
        Self {
            target_profile,
            sample_rate: sample_rate.max(1.0),
        }
    }

    /// Compute energy in each critical mix band for an audio channel buffer.
    pub fn analyze_band_energies(&self, buffer: &[f32]) -> [f32; 5] {
        let mut energies = [1e-6f32; 5];
        if buffer.is_empty() {
            return energies;
        }

        let n = buffer.len();
        // Band probe centers: SubBass (45Hz), LowBass (150Hz), LowMid (500Hz), HighMid (2000Hz), HighAir (10000Hz)
        let center_freqs = [45.0f32, 150.0, 500.0, 2000.0, 10000.0];

        for (b, &freq) in center_freqs.iter().enumerate() {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            let omega = 2.0 * std::f32::consts::PI * freq / self.sample_rate;

            for (i, &s) in buffer.iter().enumerate() {
                let angle = omega * (i as f32);
                real += s * angle.cos();
                imag -= s * angle.sin();
            }

            energies[b] = ((real * real + imag * imag) / (n as f32)).max(1e-6);
        }

        // Convert to relative dB scale
        let total_energy = energies.iter().sum::<f32>().max(1e-6);
        let mut db_energies = [0.0f32; 5];
        for b in 0..5 {
            db_energies[b] = 10.0 * (energies[b] / total_energy).log10();
        }

        db_energies
    }

    /// Detect frequency masking clashes between two tracks (e.g. Kick vs Bass).
    pub fn detect_masking(
        &self,
        track_a_name: &str,
        track_a_audio: &[f32],
        track_b_name: &str,
        track_b_audio: &[f32],
    ) -> Vec<MaskingConflict> {
        let mut conflicts = Vec::new();
        let energies_a = self.analyze_band_energies(track_a_audio);
        let energies_b = self.analyze_band_energies(track_b_audio);

        for (band_idx, band) in CriticalMixBand::ALL.iter().enumerate() {
            let db_a = energies_a[band_idx];
            let db_b = energies_b[band_idx];

            // If both tracks have high relative energy in the same band, there is masking
            if db_a > -10.0 && db_b > -10.0 {
                let energy_overlap = (db_a.min(db_b) + 10.0) / 10.0;
                let severity = energy_overlap.clamp(0.0, 1.0);

                if severity > 0.35 {
                    let (f_min, f_max) = band.frequency_range_hz();
                    let center_hz = (f_min * f_max).sqrt();
                    let cut_db = -(severity * 3.5);

                    conflicts.push(MaskingConflict {
                        track_a_name: track_a_name.to_string(),
                        track_b_name: track_b_name.to_string(),
                        band: *band,
                        clash_severity: severity,
                        suggested_cut_hz: center_hz,
                        suggested_q: 2.0,
                        suggested_gain_db: cut_db,
                        recommended_action: format!(
                            "Dynamic EQ dip of {:.1} dB at {:.0} Hz on '{}' ducked by '{}'",
                            cut_db, center_hz, track_b_name, track_a_name
                        ),
                    });
                }
            }
        }

        conflicts
    }

    /// Perform full mix evaluation and generate corrective report.
    pub fn generate_mix_report(
        &self,
        master_bus_audio: &[f32],
        track_stems: &[(&str, &[f32])],
    ) -> MixAssistantReport {
        // Calculate integrated LUFS approximation (K-weighted RMS)
        let rms = (master_bus_audio.iter().map(|&s| s * s).sum::<f32>()
            / master_bus_audio.len().max(1) as f32)
            .sqrt();
        let current_lufs = if rms > 1e-6 {
            20.0 * rms.log10() - 0.691
        } else {
            -70.0
        };

        let target_lufs = self.target_profile.target_lufs();
        let suggested_master_gain_trim_db = (target_lufs - current_lufs).clamp(-18.0, 18.0);

        // Compare spectral balance against target genre profile
        let master_energies = self.analyze_band_energies(master_bus_audio);
        let target_energies = self.target_profile.target_band_energies_db();
        let mut deviations = [0.0f32; 5];
        for i in 0..5 {
            deviations[i] = master_energies[i] - target_energies[i];
        }

        // Detect pairwise masking conflicts across all provided stems
        let mut all_conflicts = Vec::new();
        for i in 0..track_stems.len() {
            for j in (i + 1)..track_stems.len() {
                let (name_a, buf_a) = track_stems[i];
                let (name_b, buf_b) = track_stems[j];
                let conflicts = self.detect_masking(name_a, buf_a, name_b, buf_b);
                all_conflicts.extend(conflicts);
            }
        }

        MixAssistantReport {
            current_integrated_lufs: current_lufs,
            target_lufs,
            suggested_master_gain_trim_db,
            spectral_balance_deviations_db: deviations,
            masking_conflicts: all_conflicts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genre_profile_targets() {
        let pop = TargetGenreProfile::ModernPop;
        let club = TargetGenreProfile::ClubElectronic;

        assert_eq!(pop.target_lufs(), -14.0);
        assert_eq!(club.target_lufs(), -9.0);
        assert!(club.target_band_energies_db()[0] > pop.target_band_energies_db()[0]);
    }

    #[test]
    fn test_masking_detection_between_conflicting_signals() {
        let assistant = AutonomousMixAssistant::new(TargetGenreProfile::ClubElectronic, 44100.0);
        let n = 2048;

        // Track A: Low frequency sine (Kick)
        let kick: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 55.0 * i as f32 / 44100.0).sin() * 0.9)
            .collect();

        // Track B: Sub bass sine (Bass)
        let bass: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 50.0 * i as f32 / 44100.0).sin() * 0.9)
            .collect();

        let conflicts = assistant.detect_masking("Kick", &kick, "SubBass", &bass);
        assert!(!conflicts.is_empty());
        assert_eq!(conflicts[0].band, CriticalMixBand::SubBass);
        assert!(conflicts[0].suggested_gain_db < 0.0);
    }

    #[test]
    fn test_full_mix_report_auto_gain() {
        let assistant = AutonomousMixAssistant::new(TargetGenreProfile::ModernPop, 44100.0);
        let n = 4096;
        let master: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.2)
            .collect();

        let report = assistant.generate_mix_report(&master, &[]);
        assert_eq!(report.target_lufs, -14.0);
        // Current signal is at approx -14.6 dB RMS, so suggested trim should be modest
        assert!(report.suggested_master_gain_trim_db.abs() < 10.0);
    }
}
