#![no_main]
use libfuzzer_sys::fuzz_target;
use summoner_project::parse_project_toml;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_project_toml(s);
    }
});
