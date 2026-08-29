// Summoner - Deterministic Golden Granular Cloud & 3D Spatial Room Acoustic Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::spatial_bus::SpatialBus;
use summoner_core::transport::Transport;
use summoner_dsp::granular_cloud::{GrainWindowType, GranularCloudNode};
use summoner_dsp::sampler::SampleBuffer;
use summoner_dsp::spatial_audio::Position3D;
use summoner_dsp::spatial_room::{SpatialRoomNode, WallMaterial};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::spatial_trajectory_view::SpatialTrajectoryView;
use summoner_sequencer::spatial_trajectory::{
    SpatialAutomationEngine, SpatialInterpolationCurve, SpatialTrajectoryTrack, SpatialWaypoint,
    TrajectoryPathType,
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
            "Golden granular cloud & spatial room acoustic hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_granular_cloud_window_reconstruction_and_spray_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut cloud_hann = GranularCloudNode::new(sample_rate);
    cloud_hann.window_type = GrainWindowType::Hann;
    cloud_hann.grain_duration_ms = 45.0;
    cloud_hann.density = 60.0;
    cloud_hann.spray = 0.15;
    cloud_hann.pitch_jitter = 2.0;

    let mut cloud_tukey = GranularCloudNode::new(sample_rate);
    cloud_tukey.window_type = GrainWindowType::Tukey;
    cloud_tukey.tukey_alpha = 0.4;
    cloud_tukey.grain_duration_ms = 80.0;
    cloud_tukey.density = 40.0;
    cloud_tukey.spray = 0.25;

    let mut cloud_blackman = GranularCloudNode::new(sample_rate);
    cloud_blackman.window_type = GrainWindowType::Blackman;
    cloud_blackman.grain_duration_ms = 30.0;
    cloud_blackman.density = 75.0;
    cloud_blackman.stereo_spread = 1.0;

    // Create a deterministic sine wave sample buffer
    let mut sample_data = Vec::with_capacity(48000);
    for n in 0..48000 {
        let t = n as f32 / 48000.0;
        let s = (2.0 * PI * 220.0 * t).sin() * 0.8;
        sample_data.push(s);
    }
    let sample_buffer = Arc::new(SampleBuffer {
        data: sample_data,
        channels: 1,
        sample_rate: 48000,
    });
    cloud_hann.load_sample_buffer(Arc::clone(&sample_buffer));
    cloud_blackman.load_sample_buffer(sample_buffer);

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_hann = [0.0f32; BLOCK_SIZE];
    let mut out_r_hann = [0.0f32; BLOCK_SIZE];
    let mut out_l_tukey = [0.0f32; BLOCK_SIZE];
    let mut out_r_tukey = [0.0f32; BLOCK_SIZE];
    let mut out_l_black = [0.0f32; BLOCK_SIZE];
    let mut out_r_black = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio across all granular cloud configurations (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            let sig = (2.0 * PI * 440.0 * n / 48000.0).sin() * 0.6;
            in_l[i] = sig;
            in_r[i] = sig * 0.9;
        }

        {
            let _guard = AllocGuard::new();
            cloud_hann.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_hann[..], &mut out_r_hann[..]],
                &ctx,
            );
            cloud_tukey.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tukey[..], &mut out_r_tukey[..]],
                &ctx,
            );
            cloud_blackman.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_black[..], &mut out_r_black[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_hann[i].is_finite());
            assert!(out_r_hann[i].is_finite());
            assert!(out_l_tukey[i].is_finite());
            assert!(out_r_tukey[i].is_finite());
            assert!(out_l_black[i].is_finite());
            assert!(out_r_black[i].is_finite());

            hasher.update(&out_l_hann[i].to_le_bytes());
            hasher.update(&out_r_hann[i].to_le_bytes());
            hasher.update(&out_l_tukey[i].to_le_bytes());
            hasher.update(&out_r_tukey[i].to_le_bytes());
            hasher.update(&out_l_black[i].to_le_bytes());
            hasher.update(&out_r_black[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_granular_cloud.hash", &digest);
}

#[test]
fn test_golden_spatial_room_early_reflections_and_fdn_diffuse_reverb_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut room_studio = SpatialRoomNode::new(sample_rate);
    room_studio.set_room_dimensions(8.0, 12.0, 3.5);
    room_studio.set_positions(Position3D::new(2.5, 3.0, 1.5), Position3D::new(4.0, 7.0, 1.7));
    room_studio.material = WallMaterial::StudioAcoustic;
    room_studio.set_rt60(1.6);
    room_studio.dry_wet = 0.65;

    let mut room_cathedral = SpatialRoomNode::new(sample_rate);
    room_cathedral.set_room_dimensions(20.0, 45.0, 18.0);
    room_cathedral.set_positions(Position3D::new(5.0, 8.0, 2.0), Position3D::new(10.0, 25.0, 1.8));
    room_cathedral.material = WallMaterial::CathedralStone;
    room_cathedral.set_rt60(4.5);
    room_cathedral.late_gain = 0.85;
    room_cathedral.dry_wet = 0.80;

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_studio = [0.0f32; BLOCK_SIZE];
    let mut out_r_studio = [0.0f32; BLOCK_SIZE];
    let mut out_l_cath = [0.0f32; BLOCK_SIZE];
    let mut out_r_cath = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples) starting with an impulse
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        for i in 0..BLOCK_SIZE {
            if block_idx == 0 && i == 0 {
                in_l[i] = 1.0; // Impulse
                in_r[i] = 1.0;
            } else {
                in_l[i] = 0.0;
                in_r[i] = 0.0;
            }
        }

        {
            let _guard = AllocGuard::new();
            room_studio.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_studio[..], &mut out_r_studio[..]],
                &ctx,
            );
            room_cathedral.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_cath[..], &mut out_r_cath[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_studio[i].is_finite());
            assert!(out_r_studio[i].is_finite());
            assert!(out_l_cath[i].is_finite());
            assert!(out_r_cath[i].is_finite());

            hasher.update(&out_l_studio[i].to_le_bytes());
            hasher.update(&out_r_studio[i].to_le_bytes());
            hasher.update(&out_l_cath[i].to_le_bytes());
            hasher.update(&out_r_cath[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spatial_room.hash", &digest);
}

#[test]
fn test_golden_spatial_trajectory_spline_and_automation_dispatch() {
    let mut engine = SpatialAutomationEngine::new();

    // Track 1: Orbit
    let mut track1 = SpatialTrajectoryTrack::new(0, 0, "Lead Synth Orbit");
    track1.path_type = TrajectoryPathType::Orbit {
        radius_m: 5.0,
        speed_cycles_per_beat: 0.125,
        tilt_rad: 0.25,
        eccentricity: 0.3,
        center_x: 0.0,
        center_y: 0.0,
        center_z: 0.0,
    };
    track1.cycle_beats = 8.0;
    track1.param_azimuth = Some(ParamId(70));
    track1.param_elevation = Some(ParamId(71));
    track1.param_distance = Some(ParamId(72));

    // Track 2: Lissajous3D
    let mut track2 = SpatialTrajectoryTrack::new(1, 1, "Percussion Knot");
    track2.path_type = TrajectoryPathType::Lissajous3D {
        freq_x: 0.25,
        freq_y: 0.50,
        freq_z: 0.75,
        phase_x: 0.0,
        phase_y: PI / 4.0,
        phase_z: PI / 2.0,
        scale_x: 4.0,
        scale_y: 6.0,
        scale_z: 2.5,
    };
    track2.param_azimuth = Some(ParamId(73));
    track2.param_elevation = Some(ParamId(74));
    track2.param_distance = Some(ParamId(75));

    // Track 3: Catmull-Rom Waypoint Spline
    let mut track3 = SpatialTrajectoryTrack::new(2, 2, "Vocal Spline");
    track3.path_type = TrajectoryPathType::WaypointSpline;
    track3.waypoints = vec![
        SpatialWaypoint {
            beat: 0.0,
            azimuth_rad: -PI / 2.0,
            elevation_rad: 0.0,
            distance_m: 3.0,
            spread: 0.1,
            curve: SpatialInterpolationCurve::CubicCatmullRom,
        },
        SpatialWaypoint {
            beat: 4.0,
            azimuth_rad: 0.0,
            elevation_rad: PI / 4.0,
            distance_m: 2.0,
            spread: 0.3,
            curve: SpatialInterpolationCurve::CubicCatmullRom,
        },
        SpatialWaypoint {
            beat: 8.0,
            azimuth_rad: PI / 2.0,
            elevation_rad: -PI / 6.0,
            distance_m: 4.0,
            spread: 0.5,
            curve: SpatialInterpolationCurve::CubicCatmullRom,
        },
        SpatialWaypoint {
            beat: 12.0,
            azimuth_rad: PI,
            elevation_rad: 0.0,
            distance_m: 6.0,
            spread: 0.2,
            curve: SpatialInterpolationCurve::CubicCatmullRom,
        },
    ];
    track3.cycle_beats = 16.0;

    engine.add_track(track1);
    engine.add_track(track2);
    engine.add_track(track3);

    // Verify TOML round-trip
    let toml_str = engine.to_toml_string().expect("TOML serialization failed");
    let restored = SpatialAutomationEngine::from_toml_str(&toml_str).expect("TOML deserialization failed");
    assert_eq!(engine, restored);

    let spatial_bus = SpatialBus::new();
    let mut param_bus = ParamBus::new();
    for id in 70..=75 {
        let _ = param_bus.register(ParamId(id), 0.0);
    }

    let mut hasher = Hasher::new();

    // Evaluate trajectory positions across 32 song beats in 0.25 beat increments
    for step in 0..128 {
        let beat = step as f64 * 0.25;
        {
            let _guard = AllocGuard::new();
            engine.evaluate_and_dispatch(beat, &spatial_bus, &param_bus);
        }

        for obj_id in 0..3 {
            let vec = spatial_bus.get_spherical(obj_id);
            let (cx, cy, cz) = spatial_bus.get_cartesian(obj_id);

            assert!(vec.azimuth_rad.is_finite());
            assert!(vec.elevation_rad.is_finite());
            assert!(vec.distance_m.is_finite());
            assert!(cx.is_finite());
            assert!(cy.is_finite());
            assert!(cz.is_finite());

            hasher.update(&vec.azimuth_rad.to_le_bytes());
            hasher.update(&vec.elevation_rad.to_le_bytes());
            hasher.update(&vec.distance_m.to_le_bytes());
            hasher.update(&cx.to_le_bytes());
            hasher.update(&cy.to_le_bytes());
            hasher.update(&cz.to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spatial_trajectory.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_spatial_trajectory_view_snapshot_and_hud_telemetry() {
    let mut view = SpatialTrajectoryView::new();
    view.source_azimuth_rad = PI / 3.0;
    view.source_elevation_rad = PI / 6.0;
    view.source_distance_m = 4.5;
    view.grain_density = 48.0;
    view.grain_duration_ms = 75.0;
    view.grain_spray = 0.25;
    view.refresh_particles();

    let ascii = view.render_ascii(80, 24);
    assert!(!ascii.is_empty());
    assert!(ascii[0].contains("SPATIAL TRAJECTORY"));
    assert!(ascii[0].contains("Az:"));

    let render_path = "scratch/renders/spatial_trajectory_view.png";
    let res = view.render_snapshot_png(render_path, 800, 520);
    let _ = view.render_snapshot_png("../../scratch/renders/spatial_trajectory_view.png", 800, 520);
    assert!(res.is_ok(), "Spatial trajectory snapshot render must succeed: {:?}", res);

    let mut hasher = Hasher::new();
    for line in &ascii {
        hasher.update(line.as_bytes());
    }
    hasher.update(&view.source_azimuth_rad.to_le_bytes());
    hasher.update(&view.source_elevation_rad.to_le_bytes());
    hasher.update(&view.source_distance_m.to_le_bytes());
    hasher.update(&view.grain_density.to_le_bytes());
    hasher.update(&view.grain_duration_ms.to_le_bytes());
    hasher.update(&view.grain_spray.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spatial_trajectory_view.hash", &digest);
}
