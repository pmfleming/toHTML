use std::path::PathBuf;

pub(crate) fn config_file(name: &str) -> Option<PathBuf> {
    app_dir(config_root()?).map(|dir| dir.join(name))
}

pub(crate) fn data_file(name: &str) -> Option<PathBuf> {
    app_dir(data_root()?).map(|dir| dir.join(name))
}

fn app_dir(root: PathBuf) -> Option<PathBuf> {
    Some(root.join("html-editor"))
}

#[cfg(target_os = "windows")]
fn config_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "windows")]
fn data_root() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn config_root() -> Option<PathBuf> {
    home_dir().map(|home| home.join("Library").join("Application Support"))
}

#[cfg(target_os = "macos")]
fn data_root() -> Option<PathBuf> {
    config_root()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn config_root() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn data_root() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".local").join("share")))
}

#[cfg(any(target_os = "macos", all(unix, not(target_os = "macos"))))]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
