//! OS-conventional application directories (audit item PLT-5).
//!
//! | Kind   | Windows                       | macOS                              | Linux (XDG)                          |
//! |--------|-------------------------------|------------------------------------|--------------------------------------|
//! | config | `%APPDATA%\Summoner`          | `~/Library/Application Support/Summoner` | `$XDG_CONFIG_HOME/summoner`    |
//! | data   | `%LOCALAPPDATA%\Summoner`     | `~/Library/Application Support/Summoner` | `$XDG_DATA_HOME/summoner`      |
//! | cache  | `%LOCALAPPDATA%\Summoner\Cache` | `~/Library/Caches/Summoner`      | `$XDG_CACHE_HOME/summoner`           |
//! | state  | `%LOCALAPPDATA%\Summoner\State` | `~/Library/Application Support/Summoner/State` | `$XDG_STATE_HOME/summoner` |
//!
//! Setting `SUMMONER_HOME` relocates everything under that directory (portable mode,
//! tests, containers).

use std::path::{Path, PathBuf};

/// Operating-system family used to pick conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
}

impl Os {
    /// OS family of the running binary (non-desktop targets follow Linux/XDG rules).
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            Os::Linux
        }
    }
}

/// Resolved application directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
    pub state: PathBuf,
}

fn non_empty(v: Option<String>) -> Option<PathBuf> {
    v.filter(|s| !s.is_empty()).map(PathBuf::from)
}

impl AppDirs {
    /// Pure resolution from an environment lookup (injectable for tests).
    pub fn resolve(os: Os, env: &dyn Fn(&str) -> Option<String>) -> Self {
        if let Some(root) = non_empty(env("SUMMONER_HOME")) {
            return Self {
                config: root.join("config"),
                data: root.join("data"),
                cache: root.join("cache"),
                state: root.join("state"),
            };
        }
        match os {
            Os::Windows => {
                let local = non_empty(env("LOCALAPPDATA"))
                    .or_else(|| {
                        non_empty(env("USERPROFILE")).map(|p| p.join("AppData").join("Local"))
                    })
                    .unwrap_or_else(|| PathBuf::from("."));
                let roaming = non_empty(env("APPDATA")).unwrap_or_else(|| local.clone());
                let base = local.join("Summoner");
                Self {
                    config: roaming.join("Summoner"),
                    data: base.clone(),
                    cache: base.join("Cache"),
                    state: base.join("State"),
                }
            }
            Os::MacOs => {
                let home = non_empty(env("HOME")).unwrap_or_else(|| PathBuf::from("."));
                let support = home
                    .join("Library")
                    .join("Application Support")
                    .join("Summoner");
                Self {
                    config: support.clone(),
                    data: support.clone(),
                    cache: home.join("Library").join("Caches").join("Summoner"),
                    state: support.join("State"),
                }
            }
            Os::Linux => {
                let home = non_empty(env("HOME"));
                let xdg = |var: &str, fallback: &[&str]| -> PathBuf {
                    // XDG: relative paths in the variables must be ignored.
                    if let Some(p) =
                        non_empty(env(var)).filter(|p| p.to_string_lossy().starts_with('/'))
                    {
                        return p.join("summoner");
                    }
                    let mut p = home.clone().unwrap_or_else(|| PathBuf::from("."));
                    for part in fallback {
                        p = p.join(part);
                    }
                    p.join("summoner")
                };
                Self {
                    config: xdg("XDG_CONFIG_HOME", &[".config"]),
                    data: xdg("XDG_DATA_HOME", &[".local", "share"]),
                    cache: xdg("XDG_CACHE_HOME", &[".cache"]),
                    state: xdg("XDG_STATE_HOME", &[".local", "state"]),
                }
            }
        }
    }

    /// Directories for the running process.
    pub fn current() -> Self {
        Self::resolve(Os::current(), &|k| std::env::var(k).ok())
    }
}

/// Path of a small state file (lock files, auto-save, window state) in the state
/// directory. The directory is created on demand; if that fails the bare file name is
/// returned so callers degrade to the previous working-directory behaviour.
pub fn state_file(name: &str) -> PathBuf {
    let dir = AppDirs::current().state;
    if std::fs::create_dir_all(&dir).is_ok() {
        dir.join(name)
    } else {
        PathBuf::from(name)
    }
}

/// Make an absolute Windows path safe beyond `MAX_PATH` (260) by adding the `\\?\`
/// verbatim prefix. Other OSes and short or relative paths are returned unchanged.
pub fn long_path_safe(os: Os, p: &Path) -> PathBuf {
    if os != Os::Windows {
        return p.to_path_buf();
    }
    let s = p.to_string_lossy();
    if s.starts_with(r"\\?\") || s.len() < 240 {
        return p.to_path_buf();
    }
    let bytes = s.as_bytes();
    if s.starts_with(r"\\") {
        PathBuf::from(format!(r"\\?\UNC\{}", &s[2..]))
    } else if bytes.len() > 2 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        PathBuf::from(format!(r"\\?\{}", s.replace('/', "\\")))
    } else {
        p.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }

    #[test]
    fn windows_uses_appdata_and_localappdata() {
        let d = AppDirs::resolve(
            Os::Windows,
            &env_of(&[
                ("APPDATA", r"C:\Users\A\AppData\Roaming"),
                ("LOCALAPPDATA", r"C:\Users\A\AppData\Local"),
            ]),
        );
        assert_eq!(
            d.config,
            Path::new(r"C:\Users\A\AppData\Roaming").join("Summoner")
        );
        assert_eq!(
            d.data,
            Path::new(r"C:\Users\A\AppData\Local").join("Summoner")
        );
        assert!(d.cache.ends_with("Cache"));
        assert!(d.state.ends_with("State"));
    }

    #[test]
    fn windows_falls_back_to_userprofile() {
        let d = AppDirs::resolve(Os::Windows, &env_of(&[("USERPROFILE", r"C:\Users\B")]));
        assert!(d
            .data
            .starts_with(Path::new(r"C:\Users\B").join("AppData").join("Local")));
    }

    #[test]
    fn macos_uses_library_dirs() {
        let d = AppDirs::resolve(Os::MacOs, &env_of(&[("HOME", "/Users/c")]));
        assert_eq!(
            d.config,
            Path::new("/Users/c/Library/Application Support/Summoner")
        );
        assert_eq!(d.cache, Path::new("/Users/c/Library/Caches/Summoner"));
    }

    #[test]
    fn linux_honours_xdg_and_defaults() {
        let d = AppDirs::resolve(Os::Linux, &env_of(&[("HOME", "/home/d")]));
        assert_eq!(d.config, Path::new("/home/d/.config/summoner"));
        assert_eq!(d.data, Path::new("/home/d/.local/share/summoner"));
        assert_eq!(d.cache, Path::new("/home/d/.cache/summoner"));
        assert_eq!(d.state, Path::new("/home/d/.local/state/summoner"));
        let d = AppDirs::resolve(
            Os::Linux,
            &env_of(&[
                ("HOME", "/home/d"),
                ("XDG_CONFIG_HOME", "/cfg"),
                ("XDG_STATE_HOME", "relative"),
            ]),
        );
        assert_eq!(d.config, Path::new("/cfg/summoner"));
        // Relative XDG values are invalid per spec and ignored.
        assert_eq!(d.state, Path::new("/home/d/.local/state/summoner"));
    }

    #[test]
    fn summoner_home_overrides_everything() {
        for os in [Os::Windows, Os::MacOs, Os::Linux] {
            let d = AppDirs::resolve(
                os,
                &env_of(&[("SUMMONER_HOME", "/portable"), ("HOME", "/x")]),
            );
            assert_eq!(d.state, Path::new("/portable/state"));
            assert_eq!(d.config, Path::new("/portable/config"));
        }
    }

    #[test]
    fn unicode_and_spaces_are_preserved() {
        let d = AppDirs::resolve(Os::MacOs, &env_of(&[("HOME", "/Users/Jörg Müller 音楽")]));
        assert!(d.data.to_string_lossy().contains("Jörg Müller 音楽"));
    }

    #[test]
    fn long_windows_paths_get_verbatim_prefix() {
        let long = format!(r"C:\{}\f.wav", "a".repeat(250));
        let p = long_path_safe(Os::Windows, Path::new(&long));
        assert!(p.to_string_lossy().starts_with(r"\\?\C:\"));
        let unc = format!(r"\\srv\share\{}", "b".repeat(250));
        assert!(long_path_safe(Os::Windows, Path::new(&unc))
            .to_string_lossy()
            .starts_with(r"\\?\UNC\srv"));
        assert_eq!(
            long_path_safe(Os::Windows, Path::new(r"C:\short.wav")),
            Path::new(r"C:\short.wav")
        );
        assert_eq!(
            long_path_safe(Os::Linux, Path::new(&long)),
            Path::new(&long)
        );
    }

    #[test]
    fn state_file_lives_under_summoner_home() {
        // Only this test sets SUMMONER_HOME; others use the injectable resolver.
        let tmp = std::env::temp_dir().join(format!("summoner_paths_{}", std::process::id()));
        std::env::set_var("SUMMONER_HOME", &tmp);
        let f = state_file("x.lock");
        std::env::remove_var("SUMMONER_HOME");
        assert_eq!(f, tmp.join("state").join("x.lock"));
        assert!(tmp.join("state").is_dir());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}