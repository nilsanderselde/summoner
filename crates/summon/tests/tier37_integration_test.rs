// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 37 Integration Tests: Immersive Audio, 3D Spatial Panning & Multichannel Engine (Steps 1061-1080).

use summoner_core::audio::{ChannelLayout, MultichannelAudioBuffer};
use summoner_dsp::spatial_audio::*;
use summoner_project::create_default_project;
use summoner_project::export::export_adm_bwf;

#[test]
fn test_step_1061_multichannel_layout_pipeline() {
    let mono = ChannelLayout::Mono;
    let stereo = ChannelLayout::Stereo;
    let sur51 = ChannelLayout::Surround5_1;
    let sur714 = ChannelLayout::Surround7_1_4;

    assert_eq!(mono.channels(), 1);
    assert_eq!(stereo.channels(), 2);
    assert_eq!(sur51.channels(), 6);
    assert_eq!(sur714.channels(), 12);

    let mut buf = MultichannelAudioBuffer::new(sur714);
    assert_eq!(buf.num_channels(), 12);
    buf.set_active_frames(512);
    buf.clear();
    assert_eq!(buf.num_frames(), 512);
}

#[test]
fn test_step_1062_3d_binaural_spatial_panner_node() {
    let mut panner = BinauralSpatialPannerNode::new(44100);
    panner.set_position(Position3D::new(-1.5, 2.0, 0.5));

    let in_mono = vec![0.8f32; 256];
    let mut out_l = vec![0.0f32; 256];
    let mut out_r = vec![0.0f32; 256];

    panner.process_block(&in_mono, &mut out_l, &mut out_r);

    let rms_l: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();
    let rms_r: f32 = (out_r.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();

    assert!(rms_l > 0.0);
    assert!(rms_r > 0.0);
    assert!(
        rms_l > rms_r,
        "Left ear should receive higher gain for left sound source"
    );
}

#[test]
fn test_step_1063_higher_order_ambisonics_3rd_order_encoder_decoder() {
    let enc = AmbisonicsEncoder3D {
        position: Position3D::new(0.5, 1.0, 0.2),
    };
    let weights = enc.encoding_weights();
    assert_eq!(weights.len(), 16);

    let mut b_format = vec![vec![0.0f32; 128]; 16];
    let input = vec![0.6f32; 128];
    enc.encode(&input, &mut b_format);

    let dec = AmbisonicsDecoder3D::new(ChannelLayout::Surround7_1_4);
    let mut output = MultichannelAudioBuffer::new(ChannelLayout::Surround7_1_4);
    output.set_active_frames(128);

    dec.decode(&b_format, &mut output);
    assert_eq!(output.num_channels(), 12);

    let mut st_l = vec![0.0f32; 128];
    let mut st_r = vec![0.0f32; 128];
    output.downmix_to_stereo(&mut st_l, &mut st_r);

    let rms_l: f32 = (st_l.iter().map(|s| s * s).sum::<f32>() / 128.0).sqrt();
    let rms_r: f32 = (st_r.iter().map(|s| s * s).sum::<f32>() / 128.0).sqrt();
    assert!(rms_l > 0.0);
    assert!(rms_r > 0.0);
}

#[test]
fn test_step_1075_surround_stem_splitter_bed_and_objects() {
    let splitter = SurroundStemSplitterBedObject::new();
    assert_eq!(splitter.bed_layout, ChannelLayout::Surround7_1_4);
    assert_eq!(splitter.num_objects, 4);
}

#[test]
fn test_step_1066_dolby_atmos_adm_bwf_exporter() {
    let proj = create_default_project("Atmos Session");
    let bytes = export_adm_bwf(&proj).expect("ADM BWF export failed");
    assert!(bytes.starts_with(b"RIFF"));
    let str_content = String::from_utf8_lossy(&bytes);
    assert!(str_content.contains("axml"));
    assert!(str_content.contains("7.1.4 Surround Bed"));
}

#[test]
fn test_step_1067_distance_attenuation_air_absorption_doppler() {
    let mut node = DistanceDopplerNode::new(44100);
    node.position = Position3D::new(0.0, 10.0, 0.0); // 10 meters away

    let input = vec![0.9f32; 128];
    let mut output = vec![0.0f32; 128];

    node.process_block(&input, &mut output);

    let rms_out: f32 = (output.iter().map(|s| s * s).sum::<f32>() / 128.0).sqrt();
    assert!(
        rms_out < 0.2,
        "Distance attenuation should reduce signal level at 10 meters"
    );
}

#[test]
fn test_step_1068_1076_head_tracking_osc_and_apple_spatial_audio() {
    let mut tracker = HeadTrackerReceiver::new();

    // OSC test
    let ok = tracker.parse_osc("/spatial/head/orientation", &[45.0, -10.0, 5.0]);
    assert!(ok);
    assert_eq!(tracker.yaw_deg, 45.0);

    // Apple Spatial Audio AirPods motion test
    tracker.parse_apple_quaternion(1.0, 0.0, 0.0, 0.0);
    assert_eq!(tracker.yaw_deg, 0.0);
}

#[test]
fn test_step_1069_surround_limiter_and_bs1770_loudness() {
    let mut limiter = SurroundLimiterAndLoudness::new();
    let mut buf = MultichannelAudioBuffer::new(ChannelLayout::Surround5_1);
    buf.set_active_frames(256);

    for ch in 0..6 {
        for s in buf.channel_mut(ch).iter_mut() {
            *s = 2.0; // Exceeds 0 dB limit
        }
    }

    limiter.process(&mut buf);

    for ch in 0..6 {
        for &s in buf.channel(ch) {
            assert!(
                s.abs() <= 1.0,
                "Surround limiter must restrict true peak <= 1.0"
            );
        }
    }
}

#[test]
fn test_step_1071_speaker_layout_calibration_matrix() {
    let mut matrix = SpeakerCalibrationMatrix::new(12);
    matrix.channel_gains_db[0] = -3.0; // Trim left channel by -3dB

    let mut buf = MultichannelAudioBuffer::new(ChannelLayout::Surround7_1_4);
    buf.set_active_frames(64);
    buf.channel_mut(0).fill(1.0);

    matrix.apply(&mut buf);

    let val = buf.channel(0)[0];
    assert!((val - 0.7079).abs() < 1e-3);
}

#[test]
fn test_step_1072_brir_convolution() {
    let brir = BrirConvolutionNode::new();
    let input = vec![1.0, 0.0, 0.0, 0.0];
    let mut out_l = vec![0.0; 64];
    let mut out_r = vec![0.0; 64];

    brir.process(&input, &mut out_l, &mut out_r);
    assert!(out_l.iter().any(|&s| s != 0.0));
    assert!(out_r.iter().any(|&s| s != 0.0));
}

#[test]
fn test_step_1073_3d_spatial_object_automation_lanes() {
    let mut auto = SpatialObjectAutomation::new();
    auto.add_keyframe(0, Position3D::new(0.0, 0.0, 0.0));
    auto.add_keyframe(100, Position3D::new(10.0, 5.0, 2.0));

    let pos = auto.evaluate_at_frame(50);
    assert_eq!(pos.x, 5.0);
    assert_eq!(pos.y, 2.5);
    assert_eq!(pos.z, 1.0);
}
