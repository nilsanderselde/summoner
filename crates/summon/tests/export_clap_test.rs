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

    // verify that it parses as a valid toml
    let cargo_content = fs::read_to_string(plugin_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("test_session"));
    
    fs::remove_dir_all(&temp_dir).unwrap();
}
