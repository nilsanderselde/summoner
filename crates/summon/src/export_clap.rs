use std::fs;
use std::path::{Path, PathBuf};
use summoner_project::parse_project_toml;

fn find_crate_path(crate_name: &str) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let candidates = [
        cwd.join("crates").join(crate_name),
        cwd.join("..").join(crate_name),
        cwd.join("..").join("crates").join(crate_name),
    ];
    for cand in &candidates {
        if cand.join("Cargo.toml").exists() {
            if let Ok(abs) = cand.canonicalize() {
                let path_str = abs.to_string_lossy().replace("\\", "/");
                let clean_str = if let Some(stripped) = path_str.strip_prefix("//?/") {
                    stripped
                } else if let Some(stripped) = path_str.strip_prefix("\\\\?\\") {
                    stripped
                } else {
                    &path_str
                };
                return clean_str.to_string();
            }
        }
    }
    format!("../{}", crate_name)
}

pub fn generate_clap_plugin(project_toml_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_template = include_str!("clap_plugin_template/Cargo.toml.template");
    let lib_template = include_str!("clap_plugin_template/lib.rs.template");

    let raw_toml = fs::read_to_string(project_toml_path)?;
    let parsed_project = parse_project_toml(&raw_toml).ok();

    let plugin_name = project_toml_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("summon_plugin");

    let instrument_name = parsed_project
        .as_ref()
        .and_then(|p| p.tracks.first())
        .map(|t| t.name.as_str())
        .unwrap_or(plugin_name);

    let plugin_id = format!("com.summoner.{}", plugin_name.to_lowercase());
    let plugin_version = "0.1.0";

    let core_path = find_crate_path("summoner_core");
    let dsp_path = find_crate_path("summoner_dsp");
    let project_path_crate = find_crate_path("summoner_project");

    let cargo_toml = cargo_template
        .replace("{{plugin_name}}", plugin_name)
        .replace("{{plugin_version}}", plugin_version)
        .replace("{{core_path}}", &core_path)
        .replace("{{dsp_path}}", &dsp_path)
        .replace("{{project_path}}", &project_path_crate);

    let lib_rs = lib_template
        .replace("{{plugin_id}}", &plugin_id)
        .replace("{{plugin_name}}", plugin_name)
        .replace("{{instrument_name}}", instrument_name)
        .replace("{{preset_toml}}", &raw_toml.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n"))
        .replace("{{plugin_version}}", plugin_version)
        .replace("{{project_path_str}}", &project_toml_path.to_string_lossy().replace("\\", "/"));

    let plugin_dir = output_dir.join(plugin_name);
    let src_dir = plugin_dir.join("src");

    fs::create_dir_all(&src_dir)?;
    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    fs::write(plugin_dir.join("build.sh"), "#!/bin/sh\ncargo build --release\n")?;
    fs::write(plugin_dir.join("build.bat"), "@echo off\r\ncargo build --release\r\n")?;

    println!("Successfully generated CLAP plugin template at {}", plugin_dir.display());

    Ok(())
}
