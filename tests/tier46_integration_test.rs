// Summoner DAW - Tier 46 End-to-End Integration Tests
// Step 1252: Integration tests for spatial impulse response generator and spectral morphing engine

use summoner_core::node::{AudioNode, ProcessContext};
use summoner_dsp::spatial_audio::{
    AcousticMaterial, Position3D, ProceduralSpatialIrGenerator, RoomAcousticModel, SpatialIrConfig,
};
use summoner_dsp::spectrogram_art::{
    ColorMappingMode, FrequencyMapping, SpectralMorphConfig, SpectralMorphMode,
    SpectrogramArtConfig, SpectrogramArtMorphNode, SpectrogramArtMorpher, SpectrogramImage,
};

#[test]
fn test_tier46_spatial_impulse_response_generator_integration() {
    // 1. Rectangular room acoustic model
    let rect_config = SpatialIrConfig {
        model: RoomAcousticModel::Rectangular {
            width: 10.0,
            length: 15.0,
            height: 4.0,
        },
        source_pos: Position3D::new(1.5, 3.0, 1.2),
        listener_pos: Position3D::new(-1.5, 8.0, 1.2),
        material: AcousticMaterial::WoodPaneling,
        air_damping: 0.002,
        sample_rate: 44100,
        duration_sec: 0.5,
    };

    let rect_gen = ProceduralSpatialIrGenerator::new(rect_config);
    let rt60_rect = rect_gen.calculate_sabine_rt60();
    assert!(
        rt60_rect > 0.05 && rt60_rect < 5.0,
        "RT60 out of expected range: {}",
        rt60_rect
    );

    let ir_rect = rect_gen.generate();
    assert!(
        !ir_rect.is_empty(),
        "IR left/right channels should not be empty"
    );
    assert_eq!(ir_rect.sample_rate, 44100);
    assert!(ir_rect.direct_delay_ms > 0.0);
    assert!(ir_rect.left.iter().any(|&s| s.abs() > 0.01));
    assert!(ir_rect.right.iter().any(|&s| s.abs() > 0.01));

    let mc_buf = ir_rect.to_multichannel_buffer();
    assert_eq!(mc_buf.num_channels(), 2);
    assert_eq!(mc_buf.num_frames(), ir_rect.left.len());

    // 2. Spherical room acoustic model
    let sphere_config = SpatialIrConfig {
        model: RoomAcousticModel::Spherical { radius: 8.0 },
        source_pos: Position3D::new(0.5, 0.5, 0.0),
        listener_pos: Position3D::new(0.0, 4.0, 0.0),
        material: AcousticMaterial::Concrete,
        air_damping: 0.001,
        sample_rate: 44100,
        duration_sec: 0.4,
    };

    let sphere_gen = ProceduralSpatialIrGenerator::new(sphere_config);
    let rt60_sphere = sphere_gen.calculate_sabine_rt60();
    assert!(rt60_sphere > 0.05);

    let ir_sphere = sphere_gen.generate();
    assert!(!ir_sphere.is_empty());
    assert!(ir_sphere.left.iter().any(|&s| s != 0.0));
    assert!(ir_sphere.right.iter().any(|&s| s != 0.0));

    // 3. Custom mesh room acoustic model
    let custom_config = SpatialIrConfig {
        model: RoomAcousticModel::CustomMesh {
            volume_m3: 350.0,
            surface_area_m2: 300.0,
            avg_absorption: 0.25,
            scattering: 0.6,
        },
        source_pos: Position3D::new(0.0, 1.5, 0.0),
        listener_pos: Position3D::new(0.0, 5.0, 0.0),
        material: AcousticMaterial::AcousticFoam,
        air_damping: 0.003,
        sample_rate: 48000,
        duration_sec: 0.3,
    };

    let custom_gen = ProceduralSpatialIrGenerator::new(custom_config);
    let rt60_custom = custom_gen.calculate_sabine_rt60();
    assert!(rt60_custom > 0.02);

    let ir_custom = custom_gen.generate();
    assert!(!ir_custom.is_empty());
    assert_eq!(ir_custom.sample_rate, 48000);
}

#[test]
fn test_tier46_spectral_morphing_engine_integration() {
    let art_config = SpectrogramArtConfig {
        min_freq_hz: 80.0,
        max_freq_hz: 8000.0,
        mapping: FrequencyMapping::Logarithmic,
        color_mode: ColorMappingMode::RGBColorNote,
        num_oscillators: 32,
    };

    let modes = [
        SpectralMorphMode::LinearCrossfade,
        SpectralMorphMode::SpectralWarp,
        SpectralMorphMode::ThresholdBlend,
        SpectralMorphMode::ColorHueMorph,
    ];

    let mut img_a = SpectrogramImage::new(16, 16);
    let mut img_b = SpectrogramImage::new(16, 16);

    for i in 0..16 {
        img_a.set_pixel(i, i, 255, 0, 0); // Red diagonal
        img_b.set_pixel(i, 15 - i, 0, 255, 255); // Cyan anti-diagonal
    }

    for mode in modes {
        let morph_config = SpectralMorphConfig {
            morph_mode: mode,
            morph_factor: 0.5,
            art_config: art_config.clone(),
        };

        let morpher = SpectrogramArtMorpher::new(morph_config);

        // Morph image rasters
        let morphed_img = morpher.morph_images(&img_a, &img_b, 0.5);
        assert_eq!(morphed_img.width, 16);
        assert_eq!(morphed_img.height, 16);

        // Verify image pixels blend content from both source soundscapes
        let mut non_zero_pixels = 0;
        for y in 0..16 {
            for x in 0..16 {
                let (r, g, b) = morphed_img.get_pixel(x, y);
                if r > 0 || g > 0 || b > 0 {
                    non_zero_pixels += 1;
                }
            }
        }
        assert!(
            non_zero_pixels > 0,
            "Morphed image should contain blended content for mode {:?}",
            mode
        );

        // Synthesize morphed offline PCM audio buffer
        let pcm_audio = morpher.generate_morphed_audio_buffer(&img_a, &img_b, 0.5, 44100, 0.1);
        assert!(
            !pcm_audio.is_empty(),
            "Generated audio buffer should not be empty"
        );
        assert!(
            pcm_audio.iter().any(|&s| s.abs() > 0.01),
            "Audio signal should be synthesized"
        );
    }
}

#[test]
fn test_tier46_spectrogram_art_morph_node_realtime_integration() {
    let art_config = SpectrogramArtConfig {
        min_freq_hz: 100.0,
        max_freq_hz: 4000.0,
        mapping: FrequencyMapping::Linear,
        color_mode: ColorMappingMode::Grayscale,
        num_oscillators: 16,
    };

    let morph_config = SpectralMorphConfig {
        morph_mode: SpectralMorphMode::LinearCrossfade,
        morph_factor: 0.3,
        art_config,
    };

    let mut img_a = SpectrogramImage::new(32, 16);
    let mut img_b = SpectrogramImage::new(32, 16);

    for x in 0..32 {
        img_a.set_pixel(x, 4, 200, 200, 200);
        img_b.set_pixel(x, 12, 180, 180, 180);
    }

    let morpher = SpectrogramArtMorpher::new(morph_config);
    let mut node = SpectrogramArtMorphNode {
        morpher,
        image_a: img_a,
        image_b: img_b,
        morph_factor: 0.5,
        phase_accumulator: vec![0.0; 16],
    };

    assert_eq!(node.name(), "SpectrogramArtMorphNode");

    let inputs: &[&[f32]] = &[];
    let mut out_l = vec![0.0f32; 512];
    let mut out_r = vec![0.0f32; 512];
    let outputs: &mut [&mut [f32]] = &mut [&mut out_l[..], &mut out_r[..]];

    let ctx = ProcessContext::new(44100, 120.0, 0);

    node.process(inputs, outputs, &ctx);

    // Audio node outputs non-silent audio block
    let rms_l: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / 512.0).sqrt();
    let rms_r: f32 = (out_r.iter().map(|s| s * s).sum::<f32>() / 512.0).sqrt();
    assert!(rms_l > 0.001, "Left channel RMS should be non-zero");
    assert!(rms_r > 0.001, "Right channel RMS should be non-zero");
}

#[test]
fn test_tier46_spatial_morphing_audio_pipeline_integration() {
    // Pipeline test: Generate spatial IR, morph soundscapes, and verify end-to-end signal processing compatibility
    let rect_config = SpatialIrConfig {
        model: RoomAcousticModel::Rectangular {
            width: 8.0,
            length: 12.0,
            height: 3.5,
        },
        source_pos: Position3D::new(0.0, 2.0, 1.0),
        listener_pos: Position3D::new(0.0, 6.0, 1.0),
        material: AcousticMaterial::Concrete,
        air_damping: 0.001,
        sample_rate: 44100,
        duration_sec: 0.25,
    };
    let ir_gen = ProceduralSpatialIrGenerator::new(rect_config);
    let spatial_ir = ir_gen.generate();

    let art_config = SpectrogramArtConfig {
        min_freq_hz: 100.0,
        max_freq_hz: 5000.0,
        mapping: FrequencyMapping::Logarithmic,
        color_mode: ColorMappingMode::RGBColorNote,
        num_oscillators: 16,
    };
    let morph_config = SpectralMorphConfig {
        morph_mode: SpectralMorphMode::LinearCrossfade,
        morph_factor: 0.5,
        art_config,
    };
    let morpher = SpectrogramArtMorpher::new(morph_config);

    let mut img1 = SpectrogramImage::new(16, 16);
    let mut img2 = SpectrogramImage::new(16, 16);
    for x in 0..16 {
        img1.set_pixel(x, 2, 255, 100, 50);
        img2.set_pixel(x, 10, 50, 100, 255);
    }

    let raw_morphed_audio = morpher.generate_morphed_audio_buffer(&img1, &img2, 0.5, 44100, 0.25);
    assert!(!raw_morphed_audio.is_empty());
    assert_eq!(spatial_ir.sample_rate, 44100);

    // Simple time-domain convolution with spatial IR left/right channels
    let ir_l = &spatial_ir.left;
    let ir_r = &spatial_ir.right;
    let conv_len = raw_morphed_audio.len() + ir_l.len() - 1;
    let mut wet_l = vec![0.0f32; conv_len];
    let mut wet_r = vec![0.0f32; conv_len];

    for i in 0..raw_morphed_audio.len() {
        for j in 0..ir_l.len() {
            wet_l[i + j] += raw_morphed_audio[i] * ir_l[j];
            wet_r[i + j] += raw_morphed_audio[i] * ir_r[j];
        }
    }

    let rms_wet_l: f32 = (wet_l.iter().map(|s| s * s).sum::<f32>() / conv_len as f32).sqrt();
    let rms_wet_r: f32 = (wet_r.iter().map(|s| s * s).sum::<f32>() / conv_len as f32).sqrt();

    assert!(
        rms_wet_l > 0.0001,
        "Processed left channel should have non-zero energy"
    );
    assert!(
        rms_wet_r > 0.0001,
        "Processed right channel should have non-zero energy"
    );
}
