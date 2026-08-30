// Summoner - Deterministic Golden Sitar & Jawari Bridge Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::sitar_bus::{
    register_sitar_params, SitarBus, SITAR_BASE_PARAM_ID,
};
use summoner_core::transport::Transport;
use summoner_dsp::jawari_bridge::{
    JawariBridge, JawariProfile, RagaScale, SitarGourdBody, TarabResonatorBank,
};
use summoner_dsp::sitar::{Sitar, SitarProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::jawari_bridge_view::JawariBridgeView;
#[cfg(feature = "gui")]
use summoner_gui::views::sitar_view::SitarView;
use summoner_sequencer::sitar_gesture::{
    SitarGestureEngine, SitarGesturePattern,
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
            "Golden sitar hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_sitar_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Ravi Shankar Kharaj Pancham (Raga Yaman: C#3=49, E#3/F3=53, G#3=56, B#3/C4=60)
    let mut ravi = Sitar::new(sample_rate);
    ravi.set_profile(SitarProfile::RaviShankarKharaj);
    ravi.set_raga_scale(RagaScale::Yaman);
    ravi.note_on(49, 0.88);
    ravi.note_on(53, 0.82);
    ravi.note_on(56, 0.80);
    ravi.trigger_chikari_strum(0.75);

    // 2. Vilayat Khan Gayaki (Raga Bhairav vocal style: D3=50, D#3=51, F#3=54, A3=57)
    let mut vilayat = Sitar::new(sample_rate);
    vilayat.set_profile(SitarProfile::VilayatKhanGayaki);
    vilayat.set_raga_scale(RagaScale::Bhairav);
    vilayat.set_meend_pull(2.0); // 2 semitones meend
    vilayat.note_on(50, 0.90);
    vilayat.note_on(54, 0.85);

    // 3. Surbahar Deep Bass (Raga Darbari: C#2=37, D#2=39, E2=40, G#2=44)
    let mut surbahar = Sitar::new(sample_rate);
    surbahar.set_profile(SitarProfile::SurbaharBass);
    surbahar.set_raga_scale(RagaScale::DarbariKanhada);
    surbahar.set_meend_pull(3.5);
    surbahar.note_on(37, 0.92);
    surbahar.note_on(44, 0.85);

    // 4. Electric Sitar Jhajhar (Raga Bilawal: C3=48, E3=52, G3=55, C4=60)
    let mut electric = Sitar::new(sample_rate);
    electric.set_profile(SitarProfile::ElectricSitar);
    electric.set_raga_scale(RagaScale::Bilawal);
    electric.note_on(48, 0.95);
    electric.note_on(60, 0.88);
    electric.trigger_chikari_strum(0.85);

    // 5. Antique Tanjore Gourd Sitar (Raga Todi: C#3=49, D3=50, E3=52, G3=55)
    let mut antique = Sitar::new(sample_rate);
    antique.set_profile(SitarProfile::AntiqueGourd);
    antique.set_raga_scale(RagaScale::Todi);
    antique.set_meend_pull(1.5);
    antique.note_on(49, 0.78);
    antique.note_on(52, 0.75);

    let mut out_ravi_l = [0.0f32; BLOCK_SIZE];
    let mut out_ravi_r = [0.0f32; BLOCK_SIZE];
    let mut out_vila_l = [0.0f32; BLOCK_SIZE];
    let mut out_vila_r = [0.0f32; BLOCK_SIZE];
    let mut out_surb_l = [0.0f32; BLOCK_SIZE];
    let mut out_surb_r = [0.0f32; BLOCK_SIZE];
    let mut out_elec_l = [0.0f32; BLOCK_SIZE];
    let mut out_elec_r = [0.0f32; BLOCK_SIZE];
    let mut out_anti_l = [0.0f32; BLOCK_SIZE];
    let mut out_anti_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            ravi.process_block(&[], &mut [&mut out_ravi_l[..], &mut out_ravi_r[..]], &ctx);
            vilayat.process_block(&[], &mut [&mut out_vila_l[..], &mut out_vila_r[..]], &ctx);
            surbahar.process_block(&[], &mut [&mut out_surb_l[..], &mut out_surb_r[..]], &ctx);
            electric.process_block(&[], &mut [&mut out_elec_l[..], &mut out_elec_r[..]], &ctx);
            antique.process_block(&[], &mut [&mut out_anti_l[..], &mut out_anti_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note transition & meend sweep halfway
            ravi.note_off(49);
            ravi.note_on(61, 0.90); // C#4
            ravi.set_meend_pull(3.0);
            vilayat.set_meend_pull(4.5); // Deep gayaki bend
            surbahar.trigger_chikari_strum(0.80);
            electric.note_off(48);
            electric.note_on(55, 0.92);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_ravi_l[i].is_finite() && out_ravi_r[i].is_finite());
            assert!(out_vila_l[i].is_finite() && out_vila_r[i].is_finite());
            assert!(out_surb_l[i].is_finite() && out_surb_r[i].is_finite());
            assert!(out_elec_l[i].is_finite() && out_elec_r[i].is_finite());
            assert!(out_anti_l[i].is_finite() && out_anti_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_ravi_l[i]) && (-1.0..=1.0).contains(&out_ravi_r[i]));
            assert!((-1.0..=1.0).contains(&out_vila_l[i]) && (-1.0..=1.0).contains(&out_vila_r[i]));
            assert!((-1.0..=1.0).contains(&out_surb_l[i]) && (-1.0..=1.0).contains(&out_surb_r[i]));
            assert!((-1.0..=1.0).contains(&out_elec_l[i]) && (-1.0..=1.0).contains(&out_elec_r[i]));
            assert!((-1.0..=1.0).contains(&out_anti_l[i]) && (-1.0..=1.0).contains(&out_anti_r[i]));

            hasher.update(&out_ravi_l[i].to_le_bytes());
            hasher.update(&out_ravi_r[i].to_le_bytes());
            hasher.update(&out_vila_l[i].to_le_bytes());
            hasher.update(&out_vila_r[i].to_le_bytes());
            hasher.update(&out_surb_l[i].to_le_bytes());
            hasher.update(&out_surb_r[i].to_le_bytes());
            hasher.update(&out_elec_l[i].to_le_bytes());
            hasher.update(&out_elec_r[i].to_le_bytes());
            hasher.update(&out_anti_l[i].to_le_bytes());
            hasher.update(&out_anti_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_sitar.hash", &digest);
}

#[test]
fn test_golden_sitar_jawari_and_tarab_deterministic() {
    let sample_rate = 48000;

    let mut jawari = JawariBridge::new(JawariProfile::DeerHornCurved);
    let mut tarab = TarabResonatorBank::new(sample_rate, RagaScale::Yaman, 138.59);
    let mut body = SitarGourdBody::new(sample_rate);

    let mut hasher = Hasher::new();

    for step in 0..2048 {
        let t = step as f32 / 48000.0;
        let synthetic_disp = (t * 138.59 * 2.0 * std::f32::consts::PI).sin() * 0.0015;

        let (f_jawari, shorten, tarab_out, b_l, b_r) = {
            let _guard = AllocGuard::new();
            let (fj, sh) = jawari.step_contact(synthetic_disp, sample_rate);
            let to = tarab.process_sympathetic(synthetic_disp * 100.0 + fj);
            let (l, r) = body.process_body(to * 0.5 + fj * 0.01);
            (fj, sh, to, l, r)
        };

        assert!(f_jawari.is_finite());
        assert!(shorten.is_finite());
        assert!(tarab_out.is_finite());
        assert!(b_l.is_finite() && b_r.is_finite());

        hasher.update(&f_jawari.to_le_bytes());
        hasher.update(&shorten.to_le_bytes());
        hasher.update(&tarab_out.to_le_bytes());
        hasher.update(&b_l.to_le_bytes());
        hasher.update(&b_r.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_sitar_jawari.hash", &digest);
}

#[test]
fn test_golden_sitar_gesture_and_bus_deterministic() {
    let mut engine = SitarGestureEngine::with_pattern(
        SitarGesturePattern::RagaYamanAlap {
            phrase_length_beats: 8.0,
            max_meend_pull: 4.0,
        },
    );
    engine.is_looping = true;

    let bus = SitarBus::new();
    let mut param_bus = ParamBus::new();
    register_sitar_params(&mut param_bus, SITAR_BASE_PARAM_ID);

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), SITAR_BASE_PARAM_ID);
            bus.snapshot()
        };

        assert!(snap.mizrab_velocity.is_finite());
        assert!(snap.meend_pull_semitones.is_finite());
        assert!(snap.jawari_gap_mm.is_finite());
        assert!(snap.jiva_thread_pos.is_finite());
        assert!(snap.tarab_bleed.is_finite());
        assert!(snap.chikari_trigger.is_finite());
        assert!(snap.gourd_decay.is_finite());
        assert!(snap.body_gain.is_finite());
        assert!(snap.master_gain.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.mizrab_velocity.to_le_bytes());
        hasher.update(&snap.meend_pull_semitones.to_le_bytes());
        hasher.update(&snap.jawari_gap_mm.to_le_bytes());
        hasher.update(&snap.jiva_thread_pos.to_le_bytes());
        hasher.update(&snap.tarab_bleed.to_le_bytes());
        hasher.update(&snap.chikari_trigger.to_le_bytes());
        hasher.update(&snap.raga_scale_id.to_le_bytes());
        hasher.update(&snap.gourd_decay.to_le_bytes());
        hasher.update(&snap.body_gain.to_le_bytes());
        hasher.update(&snap.master_gain.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_sitar_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_sitar_views_ascii_and_snapshots() {
    let sitar_view = SitarView::new();
    let jawari_view = JawariBridgeView::new();

    let sitar_ascii = sitar_view.render_ascii(80, 16);
    let jawari_ascii = jawari_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &sitar_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &jawari_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_sitar_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = sitar_view.render_snapshot_png("scratch/renders/sitar_view.png", 800, 520);
    let _ = sitar_view.render_snapshot_png("../../scratch/renders/sitar_view.png", 800, 520);
    let _ = sitar_view.render_snapshot_png("crates/summon/scratch/renders/sitar_view.png", 800, 520);
    let _ = sitar_view.render_snapshot_png("crates/summoner_gui/scratch/renders/sitar_view.png", 800, 520);

    let _ = jawari_view.render_snapshot_png("scratch/renders/jawari_bridge_view.png", 800, 520);
    let _ = jawari_view.render_snapshot_png("../../scratch/renders/jawari_bridge_view.png", 800, 520);
    let _ = jawari_view.render_snapshot_png("crates/summon/scratch/renders/jawari_bridge_view.png", 800, 520);
    let _ = jawari_view.render_snapshot_png("crates/summoner_gui/scratch/renders/jawari_bridge_view.png", 800, 520);
}
