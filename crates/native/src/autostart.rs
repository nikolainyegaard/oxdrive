//! "Start OneDrive when I sign in": an XDG autostart desktop file pointing at this binary.

use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CONFIG_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")),
        |c| Some(PathBuf::from(c)),
    )?;
    Some(dir.join("autostart").join("oxdrive.desktop"))
}

/// Whether the desktop file exists.
#[must_use]
pub fn enabled() -> bool {
    path().is_some_and(|p| p.exists())
}

/// Writes or removes the desktop file.
///
/// # Errors
/// The file could not be written or removed.
pub fn set(on: bool) -> oxdrive_core::Result<()> {
    let path = path().ok_or("no home directory")?;
    if !on {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("could not remove the autostart entry: {e}"))
            }
            _ => Ok(()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| format!("could not find this binary: {e}"))?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=OneDrive\nComment=Keeps your OneDrive in sync\nExec={}\nIcon=folder-cloud\nTerminal=false\n",
        exe.display()
    );
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    std::fs::write(&path, entry).map_err(|e| format!("could not write the autostart entry: {e}"))
}
