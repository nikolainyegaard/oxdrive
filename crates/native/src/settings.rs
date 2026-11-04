//! The settings file under `~/.config/oxdrive/`, written through a temp file and a rename.

use std::path::PathBuf;

use oxdrive_core::settings::Settings;

fn path() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CONFIG_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")),
        |c| Some(PathBuf::from(c)),
    )?;
    Some(dir.join("oxdrive").join("settings"))
}

/// The saved settings, or the defaults when there is no file yet.
#[must_use]
pub fn load() -> Settings {
    path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|text| Settings::parse(&text))
        .unwrap_or_default()
}

/// # Errors
/// The directory or file could not be written.
pub fn save(settings: &Settings) -> oxdrive_core::Result<()> {
    use std::os::unix::fs::DirBuilderExt as _;
    let path = path().ok_or("no home directory")?;
    let dir = path.parent().ok_or("no config directory")?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, settings.render())
        .and_then(|()| std::fs::rename(&tmp, &path))
        .map_err(|e| format!("could not save settings: {e}"))
}
