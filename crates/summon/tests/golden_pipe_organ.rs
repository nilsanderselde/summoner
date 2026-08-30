// Summoner - Deterministic Golden Pipe Organ & Windchest Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::pipe_organ_bus::{
    PipeOrganArticulation, PipeOrganBus,
};
use summoner_core::transport::Transport;
use summoner_dsp::pipe_organ::{PipeOrgan, PipeOrganProfile};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::windchest::{air_velocity_from_pressure, Windchest};
#[cfg(feature = "gui")]
use summoner_gui::views::pipe_organ_view::PipeOrganView;
#[cfg(feature = "gui")]
use summoner_gui::views::rank_voicing_view::RankVoicingView;
use summoner_sequencer::pipe_organ_gesture::{
    PipeOrganGestureEngine, PipeOrganGesturePattern, PipeOrganInterpolationCurve, PipeOrganWaypoint,
};

const BLOCK_SIZE: usize = 64;

fn check_or_save_golden(hash_filename: &str, digest: &str) {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        let _ = fs::create_dir_all(golden_dir);
    }

    let hash_file = golden_dir.join(hash_filename);
    if hash_file.exists() {
        let expected = fs::read_to_string(&hash_file)
            .expect("Failed to read golden hash")
            .trim()
            .to_string();
        assert_eq!(
            digest, expected,
            "Golden pipe organ hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_pipe_organ_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Toccata Plenum (Grand Tutti: D minor chord D3=50, A3=57, D4=62, F4=65)
    let mut toccata = PipeOrgan::new(sample_rate);
    toccata.set_profile(PipeOrganProfile::ToccataPlenum);
    toccata.note_on(50, 0.95);
    toccata.note_on(57, 0.90);
    toccata.note_on(62, 0.90);
    toccata.note_on(65, 0.85);

    // 2. Warm Baroque Chorale (G major chord: G2=43, D3=50, B3=59, G4=67)
    let mut chorale = PipeOrgan::new(sample_rate);
    chorale.set_profile(PipeOrganProfile::BaroqueChorale);
    chorale.note_on(43, 0.80);
    chorale.note_on(50, 0.75);
    chorale.note_on(59, 0.75);
    chorale.note_on(67, 0.70);

    // 3. Gothic Cathedral (Full Tutti with all stops: C2=36, G2=43, C3=48, E3=52, G3=55, C4=60)
    let mut gothic = PipeOrgan::new(sample_rate);
    gothic.set_profile(PipeOrganProfile::GothicCathedral);
    gothic.note_on(36, 1.00);
    gothic.note_on(43, 0.95);
    gothic.note_on(48, 0.90);
    gothic.note_on(52, 0.85);
    gothic.note_on(55, 0.85);
    gothic.note_on(60, 0.80);

    // 4. French Romantic Reed (F minor reed fanfare: F3=53, C4=60, Ab4=68, C5=72)
    let mut french_reed = PipeOrgan::new(sample_rate);
    french_reed.set_profile(PipeOrganProfile::FrenchRomanticReed);
    french_reed.note_on(53, 0.90);
    french_reed.note_on(60, 0.85);
    french_reed.note_on(68, 0.85);
    french_reed.note_on(72, 0.80);

    // 5. Vocal Vox Humana with Tremulant (E minor: E3=52, B3=59, G4=67, E5=76)
    let mut vox = PipeOrgan::new(sample_rate);
    vox.set_profile(PipeOrganProfile::VocalVoxHumana);
    vox.note_on(52, 0.80);
    vox.note_on(59, 0.75);
    vox.note_on(67, 0.75);
    vox.note_on(76, 0.70);

    // 6. Silver Flutes Duo (A4=69, C#5=73, E5=76, A5=81)
    let mut flutes = PipeOrgan::new(sample_rate);
    flutes.set_profile(PipeOrganProfile::SilverFlutes);
    flutes.note_on(69, 0.75);
    flutes.note_on(73, 0.70);
    flutes.note_on(76, 0.70);
    flutes.note_on(81, 0.65);

    let mut out_tocc_l = [0.0f32; BLOCK_SIZE];
    let mut out_tocc_r = [0.0f32; BLOCK_SIZE];
    let mut out_chor_l = [0.0f32; BLOCK_SIZE];
    let mut out_chor_r = [0.0f32; BLOCK_SIZE];
    let mut out_goth_l = [0.0f32; BLOCK_SIZE];
    let mut out_goth_r = [0.0f32; BLOCK_SIZE];
    let mut out_fren_l = [0.0f32; BLOCK_SIZE];
    let mut out_fren_r = [0.0f32; BLOCK_SIZE];
    let mut out_voxh_l = [0.0f32; BLOCK_SIZE];
    let mut out_voxh_r = [0.0f32; BLOCK_SIZE];
    let mut out_flut_l = [0.0f32; BLOCK_SIZE];
    let mut out_flut_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            toccata.process_block(&[], &mut [&mut out_tocc_l[..], &mut out_tocc_r[..]], &ctx);
            chorale.process_block(&[], &mut [&mut out_chor_l[..], &mut out_chor_r[..]], &ctx);
            gothic.process_block(&[], &mut [&mut out_goth_l[..], &mut out_goth_r[..]], &ctx);
            french_reed.process_block(&[], &mut [&mut out_fren_l[..], &mut out_fren_r[..]], &ctx);
            vox.process_block(&[], &mut [&mut out_voxh_l[..], &mut out_voxh_r[..]], &ctx);
            flutes.process_block(&[], &mut [&mut out_flut_l[..], &mut out_flut_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note off and note transition halfway
            toccata.note_off(65);
            toccata.note_on(64, 0.85); // E4
            chorale.note_off(67);
            chorale.note_on(66, 0.70); // F#4
            gothic.note_off(60);
            gothic.note_on(59, 0.80);  // B3
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_tocc_l[i].is_finite() && out_tocc_r[i].is_finite());
            assert!(out_chor_l[i].is_finite() && out_chor_r[i].is_finite());
            assert!(out_goth_l[i].is_finite() && out_goth_r[i].is_finite());
            assert!(out_fren_l[i].is_finite() && out_fren_r[i].is_finite());
            assert!(out_voxh_l[i].is_finite() && out_voxh_r[i].is_finite());
            assert!(out_flut_l[i].is_finite() && out_flut_r[i].is_finite());

            assert!((-15.0..=15.0).contains(&out_tocc_l[i]) && (-15.0..=15.0).contains(&out_tocc_r[i]));
            assert!((-15.0..=15.0).contains(&out_chor_l[i]) && (-15.0..=15.0).contains(&out_chor_r[i]));
            assert!((-15.0..=15.0).contains(&out_goth_l[i]) && (-15.0..=15.0).contains(&out_goth_r[i]));
            assert!((-15.0..=15.0).contains(&out_fren_l[i]) && (-15.0..=15.0).contains(&out_fren_r[i]));
            assert!((-15.0..=15.0).contains(&out_voxh_l[i]) && (-15.0..=15.0).contains(&out_voxh_r[i]));
            assert!((-15.0..=15.0).contains(&out_flut_l[i]) && (-15.0..=15.0).contains(&out_flut_r[i]));

            hasher.update(&out_tocc_l[i].to_le_bytes());
            hasher.update(&out_tocc_r[i].to_le_bytes());
            hasher.update(&out_chor_l[i].to_le_bytes());
            hasher.update(&out_chor_r[i].to_le_bytes());
            hasher.update(&out_goth_l[i].to_le_bytes());
            hasher.update(&out_goth_r[i].to_le_bytes());
            hasher.update(&out_fren_l[i].to_le_bytes());
            hasher.update(&out_fren_r[i].to_le_bytes());
            hasher.update(&out_voxh_l[i].to_le_bytes());
            hasher.update(&out_voxh_r[i].to_le_bytes());
            hasher.update(&out_flut_l[i].to_le_bytes());
            hasher.update(&out_flut_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_pipe_organ.hash", &digest);
}

#[test]
fn test_golden_windchest_aerodynamics_deterministic() {
    let sample_rate = 48000;
    let mut windchest = Windchest::new(sample_rate);
    windchest.set_nominal_pressure(90.0);
    windchest.set_tremulant(true, 5.5, 0.15);

    // Verify air velocity formula: v = sqrt(2 * P_pascals / rho)
    // For P = 90 mmH2O: P_pascals = 90 * 9.80665 = 882.5985 Pa
    // v = sqrt(2 * 882.5985 / 1.204) = sqrt(1466.11) = 38.2898 m/s
    let v_expected = (2.0 * 90.0 * 9.80665 / 1.204f32).sqrt();
    let v_calc = air_velocity_from_pressure(90.0);
    assert!((v_calc - v_expected).abs() < 1e-3);

    let mut hasher = Hasher::new();

    // Simulate multi-rank chord burst demand cycles
    for step in 0..4096 {
        let t = step as f32 / 48000.0;
        let air_demand = if (0.02..0.06).contains(&t) {
            18.5 // Heavy tutti chord demand -> pressure sag
        } else if t > 0.06 {
            2.0  // Decay / recovery
        } else {
            0.0
        };

        let eff_p = {
            let _guard = AllocGuard::new();
            windchest.process_sample(air_demand)
        };

        let v = windchest.air_velocity();

        assert!(eff_p.is_finite());
        assert!(v.is_finite());
        assert!(eff_p > 10.0 && eff_p < 200.0);
        assert!(v > 10.0 && v < 80.0);

        hasher.update(&eff_p.to_le_bytes());
        hasher.update(&v.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_windchest.hash", &digest);
}

#[test]
fn test_golden_pipe_organ_gesture_and_bus_deterministic() {
    let mut engine = PipeOrganGestureEngine::new(PipeOrganGesturePattern::CustomSpline);
    engine.default_curve = PipeOrganInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        PipeOrganWaypoint {
            beat: 0.0,
            stops_mask: 119, // Principal 8', Bourdon 16', Octave 4', Super Octave 2', Mixture IV, Trompette 8'
            wind_pressure_mmh2o: 80.0,
            cutup_ratio: 0.25,
            chiff_duration_ms: 28.0,
            tracker_velocity: 0.85,
            tremulant_enabled: false,
            tremulant_rate_hz: 5.5,
            tremulant_depth: 0.10,
            master_gain: 0.85,
            articulation: PipeOrganArticulation::ToccataPlenum,
            curve: PipeOrganInterpolationCurve::CubicCatmullRom,
        },
        PipeOrganWaypoint {
            beat: 2.0,
            stops_mask: 255, // Full Gothic Tutti (All 8 ranks)
            wind_pressure_mmh2o: 115.0,
            cutup_ratio: 0.22,
            chiff_duration_ms: 22.0,
            tracker_velocity: 0.95,
            tremulant_enabled: false,
            tremulant_rate_hz: 6.0,
            tremulant_depth: 0.12,
            master_gain: 0.90,
            articulation: PipeOrganArticulation::GothicCathedral,
            curve: PipeOrganInterpolationCurve::CubicCatmullRom,
        },
        PipeOrganWaypoint {
            beat: 4.0,
            stops_mask: 138, // Bourdon 16', Flute 4', Vox Humana 8'
            wind_pressure_mmh2o: 70.0,
            cutup_ratio: 0.28,
            chiff_duration_ms: 40.0,
            tracker_velocity: 0.70,
            tremulant_enabled: true,
            tremulant_rate_hz: 5.2,
            tremulant_depth: 0.25,
            master_gain: 0.82,
            articulation: PipeOrganArticulation::VocalVoxHumana,
            curve: PipeOrganInterpolationCurve::CubicCatmullRom,
        },
        PipeOrganWaypoint {
            beat: 6.0,
            stops_mask: 11, // Principal 8', Bourdon 16', Flute 4'
            wind_pressure_mmh2o: 65.0,
            cutup_ratio: 0.30,
            chiff_duration_ms: 35.0,
            tracker_velocity: 0.65,
            tremulant_enabled: false,
            tremulant_rate_hz: 5.0,
            tremulant_depth: 0.08,
            master_gain: 0.80,
            articulation: PipeOrganArticulation::BaroqueChorale,
            curve: PipeOrganInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = PipeOrganBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(1120 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.apply_to_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(1120));
            bus.snapshot()
        };

        assert!(snap.wind_pressure_mmh2o.is_finite());
        assert!(snap.cutup_ratio.is_finite());
        assert!(snap.chiff_duration_ms.is_finite());
        assert!(snap.tracker_velocity.is_finite());
        assert!(snap.tremulant_rate_hz.is_finite());
        assert!(snap.tremulant_depth.is_finite());
        assert!(snap.master_gain.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.stops_mask.to_le_bytes());
        hasher.update(&snap.wind_pressure_mmh2o.to_le_bytes());
        hasher.update(&snap.cutup_ratio.to_le_bytes());
        hasher.update(&snap.chiff_duration_ms.to_le_bytes());
        hasher.update(&snap.tracker_velocity.to_le_bytes());
        hasher.update(&(if snap.tremulant_enabled { 1u32 } else { 0u32 }).to_le_bytes());
        hasher.update(&snap.tremulant_rate_hz.to_le_bytes());
        hasher.update(&snap.tremulant_depth.to_le_bytes());
        hasher.update(&snap.master_gain.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_pipe_organ_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_pipe_organ_views_ascii_and_snapshots() {
    let organ_view = PipeOrganView::new();
    let voicing_view = RankVoicingView::new();

    let organ_ascii = organ_view.render_ascii(80, 16);
    let voicing_ascii = voicing_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &organ_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &voicing_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_pipe_organ_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = organ_view.render_snapshot_png("scratch/renders/pipe_organ_view.png", 800, 520);
    let _ = organ_view.render_snapshot_png("../../scratch/renders/pipe_organ_view.png", 800, 520);
    let _ = organ_view.render_snapshot_png("crates/summon/scratch/renders/pipe_organ_view.png", 800, 520);
    let _ = organ_view.render_snapshot_png("crates/summoner_gui/scratch/renders/pipe_organ_view.png", 800, 520);

    let _ = voicing_view.render_snapshot_png("scratch/renders/rank_voicing_view.png", 800, 520);
    let _ = voicing_view.render_snapshot_png("../../scratch/renders/rank_voicing_view.png", 800, 520);
    let _ = voicing_view.render_snapshot_png("crates/summon/scratch/renders/rank_voicing_view.png", 800, 520);
    let _ = voicing_view.render_snapshot_png("crates/summoner_gui/scratch/renders/rank_voicing_view.png", 800, 520);
}

fn check_or_save(filename: &str, digest: &str) {
    check_or_save_golden(filename, digest);
}
