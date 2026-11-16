//! First-sound acceptance test (FND-6): the shipped `summon` binary and GUI app must be able to
//! create a project, render it and produce non-silent, deterministic audio on every OS.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

fn summon() -> Command {
    Command::new(env!("CARGO_BIN_EXE_summon"))
}

fn scratch_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "summoner_first_sound_{}_{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn read_wav(path: &Path) -> (hound::WavSpec, Vec<f32>) {
    let mut r = hound::WavReader::open(path).expect("rendered wav opens");
    let spec = r.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().map(|s| s.unwrap()).collect(),
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.unwrap() as f32 / max)
                .collect()
        }
    };
    (spec, samples)
}

fn init_and_render(dir: &Path, out_name: &str, pass_directory: bool) -> PathBuf {
    let project_dir = dir.join("proj");
    let status = summon()
        .args(["init", project_dir.to_str().unwrap()])
        .output()
        .expect("spawn summon init");
    assert!(
        status.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let project_toml = project_dir.join("summoner_session.toml");
    assert!(
        project_toml.exists(),
        "init must create summoner_session.toml"
    );

    let render_target = if pass_directory {
        project_dir.to_str().unwrap()
    } else {
        project_toml.to_str().unwrap()
    };

    let out = dir.join(out_name);
    let run = summon()
        .args([
            "render-wav",
            render_target,
            out.to_str().unwrap(),
            "--frames",
            "88200",
        ])
        .output()
        .expect("spawn summon render-wav");
    assert!(
        run.status.success(),
        "render failed: {}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    out
}

#[test]
fn default_project_renders_audible_audio() {
    let dir = scratch_dir("audible");
    let wav = init_and_render(&dir, "out.wav", false);
    let (spec, samples) = read_wav(&wav);
    assert!(spec.channels >= 1);
    assert!(
        samples.len() >= 88200,
        "expected >= 2 s of audio, got {}",
        samples.len()
    );
    assert!(samples.iter().all(|s| s.is_finite()), "NaN/Inf in output");
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    assert!(peak > 0.01, "output is silent (peak {peak})");
    assert!(rms > 0.001, "output is near-silent (rms {rms})");
    assert!(peak <= 1.0 + 1e-3, "output clips: peak {peak}");

    // Also verify passing the project directory directly to render-wav works
    let wav_dir = dir.join("out_from_dir.wav");
    let run = summon()
        .args([
            "render-wav",
            dir.join("proj").to_str().unwrap(),
            wav_dir.to_str().unwrap(),
            "--frames",
            "4410",
        ])
        .output()
        .expect("spawn summon render-wav with directory");
    assert!(run.status.success());
    let (_, dir_samples) = read_wav(&wav_dir);
    assert!(dir_samples.iter().any(|s| s.abs() > 0.01));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rendering_is_bit_deterministic() {
    let dir = scratch_dir("determinism");
    let a = init_and_render(&dir, "a.wav", false);
    let b = {
        let project_toml = dir.join("proj").join("summoner_session.toml");
        let out = dir.join("b.wav");
        let run = summon()
            .args([
                "render-wav",
                project_toml.to_str().unwrap(),
                out.to_str().unwrap(),
                "--frames",
                "88200",
            ])
            .output()
            .unwrap();
        assert!(run.status.success());
        out
    };
    let ha = blake3::hash(&std::fs::read(&a).unwrap());
    let hb = blake3::hash(&std::fs::read(&b).unwrap());
    assert_eq!(ha, hb, "two renders of one project differ");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn gui_and_live_engine_first_sound_smoke() {
    let dir = scratch_dir("gui_smoke");
    let wav_path = dir.join("gui_first_sound.wav");
    let project = summoner_project::create_default_project("GUI First Sound");
    let param_bus = Arc::new(summoner_core::param_bus::ParamBus::new());
    let mut app = summoner_gui::app::SummonerApp::new(project, param_bus);

    app.start_live_audio();
    let frames = app
        .export_wav_to_path(&wav_path)
        .expect("GUI export_wav_to_path must succeed");
    assert!(frames > 0);

    let (spec, samples) = read_wav(&wav_path);
    assert!(spec.channels >= 1);
    assert!(!samples.is_empty());
    assert!(
        samples.iter().all(|s| s.is_finite()),
        "NaN/Inf in GUI WAV output"
    );
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    assert!(peak > 0.01, "GUI output is silent (peak {peak})");
    assert!(rms > 0.001, "GUI output is near-silent (rms {rms})");

    let _ = std::fs::remove_dir_all(&dir);
}
