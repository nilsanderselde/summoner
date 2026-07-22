use std::fs;
use std::path::Path;

pub fn generate_clap_plugin(project_toml_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_template = include_str!("clap_plugin_template/Cargo.toml.template");
    let lib_template = include_str!("clap_plugin_template/lib.rs.template");

    let plugin_name = project_toml_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("summon_plugin");
    
    let plugin_id = format!("com.summoner.{}", plugin_name.to_lowercase());
    let plugin_version = "0.1.0";

    let summon_dir = std::env::current_dir()?;
    let core_path = summon_dir.join("crates/summoner_core").to_string_lossy().replace("\\", "/");
    let dsp_path = summon_dir.join("crates/summoner_dsp").to_string_lossy().replace("\\", "/");
    let project_path_crate = summon_dir.join("crates/summoner_project").to_string_lossy().replace("\\", "/");

    let cargo_toml = cargo_template
        .replace("{{plugin_name}}", plugin_name)
        .replace("{{plugin_version}}", plugin_version)
        .replace("{{core_path}}", &core_path)
        .replace("{{dsp_path}}", &dsp_path)
        .replace("{{project_path}}", &project_path_crate);

    let lib_rs = lib_template
        .replace("{{plugin_id}}", &plugin_id)
        .replace("{{plugin_name}}", plugin_name)
        .replace("{{plugin_version}}", plugin_version)
        .replace("{{project_path_str}}", &project_toml_path.to_string_lossy().replace("\\", "/"));

    let plugin_dir = output_dir.join(plugin_name);
    let src_dir = plugin_dir.join("src");
    
    fs::create_dir_all(&src_dir)?;
    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(src_dir.join("lib.rs"), lib_rs)?;

    Ok(())
}
