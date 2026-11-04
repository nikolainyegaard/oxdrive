//! The settings file under `~/.config/oxdrive/`, written through a temp file and a rename, and
//! the recent activity under `~/.local/state/oxdrive/`.

use std::path::PathBuf;

use oxdrive_core::settings::Settings;
use oxdrive_core::status::Activity;

/// `$XDG_CONFIG_HOME/oxdrive`, else `~/.config/oxdrive`.
fn config_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CONFIG_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")),
        |c| Some(PathBuf::from(c)),
    )?;
    Some(dir.join("oxdrive"))
}

fn path() -> Option<PathBuf> {
    Some(config_dir()?.join("settings"))
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

/// `$XDG_STATE_HOME/oxdrive`, else `~/.local/state/oxdrive`: the recent activity per profile.
fn recent_path(profile: &str) -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_STATE_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")),
        |s| Some(PathBuf::from(s)),
    )?;
    Some(dir.join("oxdrive").join(format!("{profile}.recent")))
}

/// The activity rows saved for a profile, newest first; empty when there is no file yet.
#[must_use]
pub fn load_recent(profile: &str) -> Vec<Activity> {
    recent_path(profile)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|text| text.lines().filter_map(Activity::parse).collect())
        .unwrap_or_default()
}

/// Best effort: the list is a convenience, nothing is lost when it does not land.
pub fn save_recent(profile: &str, rows: &[Activity]) {
    let Some(path) = recent_path(profile) else {
        return;
    };
    let text: String = rows.iter().map(|r| r.render() + "\n").collect();
    let _ = path
        .parent()
        .map(std::fs::create_dir_all)
        .and_then(|_| std::fs::write(path, text).ok());
}
