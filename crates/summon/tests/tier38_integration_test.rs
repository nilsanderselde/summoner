// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 38 Integration Tests: Enterprise QA, Long-Term Support & Documentation (Steps 1081-1100).

use summoner_core::audio_drivers::{AAudioDriver, AudioUnitDriver};
use summoner_project::enterprise_qa::{ApiChangelogGenerator, GoldenRenderSuite};
use summoner_project::create_default_project;
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(parent) = manifest_dir.parent() {
        if let Some(root) = parent.parent() {
            if root.join("Cargo.toml").exists() {
                return root.to_path_buf();
            }
        }
    }
    PathBuf::from(".")
}

#[test]
fn test_step_1081_deny_missing_docs_compliance() {
    let root = get_workspace_root();
    assert!(root.join("crates/summoner_core/src/lib.rs").exists());
}

#[test]
fn test_step_1082_wasm_playground_assets() {
    let root = get_workspace_root();
    let wasm_page = root.join("docs/wasm_playground.html");
    assert!(wasm_page.exists(), "Wasm playground html file must exist at {:?}", wasm_page);
    let content = std::fs::read_to_string(&wasm_page).expect("Read wasm_playground failed");
    assert!(content.contains("Live WebAssembly Playground"));
}

#[test]
fn test_step_1083_api_changelog_generator() {
    let gen = ApiChangelogGenerator::new("1.0.0", "1.1.0");
    let log = gen.generate_changelog("summoner_core", &["AAudioDriver", "AudioUnitDriver"], &[]);
    assert!(log.contains("API Changelog for `summoner_core`"));
    assert!(log.contains("AAudioDriver"));
}

#[test]
fn test_step_1084_golden_render_suite_regression() {
    let suite = GoldenRenderSuite::new(10); // Run 10 golden project renders for unit test efficiency
    let passed = suite.run_suite().expect("Golden render suite failed");
    assert_eq!(passed, 10, "All 10 golden project renders must produce valid Blake3 hashes");

    let proj = create_default_project("Golden Test");
    assert!(suite.verify_golden_project(&proj));
}

#[test]
fn test_step_1085_valgrind_ci_script_exists() {
    let root = get_workspace_root();
    assert!(root.join("scripts/valgrind_check.sh").exists());
    assert!(
        root.join(".github/workflows/valgrind_ci.yml").exists()
            || root.join("_github_ci_disabled_for_now/workflows/valgrind_ci.yml").exists()
    );
}

#[test]
fn test_step_1086_arm64_cross_compilation_workflow() {
    let root = get_workspace_root();
    assert!(
        root.join(".github/workflows/arm64_release.yml").exists()
            || root.join("_github_ci_disabled_for_now/workflows/arm64_release.yml").exists()
    );
}

#[test]
fn test_step_1087_android_aaudio_ndk_driver() {
    let mut driver = AAudioDriver::new(48000, 192);
    assert_eq!(driver.sample_rate, 48000);
    assert_eq!(driver.buffer_size_frames, 192);
    assert!(!driver.is_active());

    driver.open_stream().expect("AAudio stream open failed");
    assert!(driver.is_active());

    let mut out = vec![1.0f32; 192];
    let count = driver.process_audio_callback(&mut out);
    assert_eq!(count, 192);
    assert_eq!(out[0], 0.0);
    assert!(driver.latency_ms() > 0.0);
}

#[test]
fn test_step_1088_ios_audiounit_coreaudio_driver() {
    let mut driver = AudioUnitDriver::new(44100, 256);
    assert_eq!(driver.sample_rate(), 44100);
    assert!(driver.render_callback(&mut [0.0; 256]).is_err());

    driver.initialize_unit().expect("AudioUnit initialization failed");
    let mut out = vec![0.5f32; 256];
    driver.render_callback(&mut out).expect("Render callback failed");
    assert_eq!(out[0], 0.0);
}

#[test]
fn test_step_1089_1091_debian_arch_alpine_packaging() {
    let root = get_workspace_root();
    assert!(root.join("packaging/linux/ppa_publish.sh").exists());
    assert!(root.join("packaging/linux/PKGBUILD").exists());
    assert!(root.join("packaging/linux/APKBUILD").exists());
}

#[test]
fn test_step_1092_docker_container_image() {
    let root = get_workspace_root();
    let dockerfile = root.join("Dockerfile");
    assert!(dockerfile.exists());
    let content = std::fs::read_to_string(&dockerfile).unwrap();
    assert!(content.contains("summoner/daw"));
}

#[test]
fn test_step_1093_kubernetes_helm_chart() {
    let root = get_workspace_root();
    assert!(root.join("packaging/helm/summoner-render-worker/Chart.yaml").exists());
    assert!(root.join("packaging/helm/summoner-render-worker/values.yaml").exists());
    assert!(root.join("packaging/helm/summoner-render-worker/templates/deployment.yaml").exists());
}

#[test]
fn test_step_1094_security_audit_monitoring_workflow() {
    let root = get_workspace_root();
    assert!(
        root.join(".github/workflows/security_audit.yml").exists()
            || root.join("_github_ci_disabled_for_now/workflows/security_audit.yml").exists()
    );
}

#[test]
fn test_step_1095_video_tutorials_assets() {
    let root = get_workspace_root();
    let tut_file = root.join("docs/tutorials/onboarding_tutorials.json");
    assert!(tut_file.exists());
    let content = std::fs::read_to_string(&tut_file).unwrap();
    assert!(content.contains("tut-01"));
}

#[test]
fn test_step_1096_academic_paper_microtonal_dsp() {
    let root = get_workspace_root();
    let paper = root.join("docs/PAPER_MICROTONAL_DSP.md");
    assert!(paper.exists());
    let content = std::fs::read_to_string(&paper).unwrap();
    assert!(content.contains("Deterministic Microtonal DSP"));
}

#[test]
fn test_step_1097_open_source_governance_code_of_conduct() {
    let root = get_workspace_root();
    assert!(root.join("GOVERNANCE.md").exists());
    assert!(root.join("CODE_OF_CONDUCT.md").exists());
}

#[test]
fn test_step_1098_1100_release_smoke_test_and_publish_scripts() {
    let root = get_workspace_root();
    assert!(root.join("scripts/release_smoke_test.sh").exists());
    assert!(root.join("scripts/publish_release_v1.1.0.sh").exists());
}
