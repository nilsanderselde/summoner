use std::env;
use std::fs;
use std::process::Command;
use summoner_project::create_default_project;
use summoner_project::serialize_project_toml;

#[test]
fn test_export_clap_plugin() {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("summoner_clap_export_test");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();

    let project_path = temp_dir.join("test_session.toml");
    let project = create_default_project("Test Session");
    fs::write(&project_path, serialize_project_toml(&project).unwrap()).unwrap();

    let output_dir = temp_dir.join("plugins");

    // Execute the export-clap command via CLI using cargo run --bin summon
    let status = Command::new(env!("CARGO_BIN_EXE_summon"))
        .arg("export-clap")
        .arg(project_path.to_str().unwrap())
        .arg(output_dir.to_str().unwrap())
        .status()
        .expect("Failed to execute summon CLI");

    assert!(status.success(), "export-clap command failed");

    let plugin_dir = output_dir.join("test_session");
    assert!(plugin_dir.exists());
    assert!(plugin_dir.join("Cargo.toml").exists());
    assert!(plugin_dir.join("src").join("lib.rs").exists());
    assert!(plugin_dir.join("build.sh").exists());
    assert!(plugin_dir.join("build.bat").exists());

    // Verify that Cargo.toml parses and contains expected name
    let cargo_content = fs::read_to_string(plugin_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("test_session"));

    // Verify that the generated CLAP plugin passes cargo check
    let check_status = Command::new("cargo")
        .arg("check")
        .current_dir(&plugin_dir)
        .status()
        .expect("Failed to run cargo check on generated CLAP plugin");
    assert!(
        check_status.success(),
        "Generated CLAP plugin failed cargo check"
    );

    fs::remove_dir_all(&temp_dir).unwrap();
}
