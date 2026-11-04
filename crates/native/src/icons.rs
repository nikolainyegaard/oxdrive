//! Icon files on disk. Built-in SVGs are compiled in; a file of the same name under
//! `~/.config/oxdrive/icons/` replaces one, so the whole set can be swapped (docs/config.md).
//!
//! The tray host needs files, not bytes: `install` lays the seven state icons out as a tiny
//! icon theme under the runtime directory, which the tray announces as `IconThemePath`, and
//! puts the app icon and a desktop entry where the desktop finds them by app id.

use std::path::{Path, PathBuf};

use oxdrive_core::status::SyncState;

use crate::tray::APP_ID;

const INDEX_THEME: &str = "[Icon Theme]\nName=hicolor\nComment=oxdrive tray icons\nDirectories=scalable/status,scalable/apps\n\n[scalable/status]\nSize=22\nType=Scalable\nMinSize=8\nMaxSize=512\nContext=Status\n\n[scalable/apps]\nSize=22\nType=Scalable\nMinSize=8\nMaxSize=512\nContext=Applications\n";

fn config_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CONFIG_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")),
        |c| Some(PathBuf::from(c)),
    )?;
    Some(dir.join(APP_ID))
}

fn data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME").map_or_else(
        || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")),
        |d| Some(PathBuf::from(d)),
    )
}

/// `icons/<name>.svg` under the config directory, when the user has put one there.
#[must_use]
pub fn custom(name: &str) -> Option<PathBuf> {
    let path = config_dir()?.join("icons").join(format!("{name}.svg"));
    path.is_file().then_some(path)
}

/// The user's file of that name, else the built-in bytes.
fn bytes(name: &str, builtin: &'static [u8]) -> Vec<u8> {
    custom(name)
        .and_then(|p| std::fs::read(p).ok())
        .unwrap_or_else(|| builtin.to_vec())
}

fn tray_builtin(state: SyncState) -> &'static [u8] {
    macro_rules! asset {
        ($name:literal) => {
            include_bytes!(concat!("../../../assets/tray/", $name, ".svg"))
        };
    }
    match state {
        SyncState::Idle => asset!("idle"),
        SyncState::Syncing => asset!("syncing"),
        SyncState::Paused => asset!("paused"),
        SyncState::Warning => asset!("warning"),
        SyncState::Error => asset!("error"),
        SyncState::Blocked => asset!("blocked"),
        SyncState::Disabled => asset!("disabled"),
    }
}

/// Writes the file only when its content differs, so an unchanged start touches nothing.
fn put(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|old| old == content) {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, content)
}

/// Lays out the tray icon theme under the runtime directory and returns its root for
/// `IconThemePath`, and installs the app icon and desktop entry under the data directory.
/// `None` when the theme could not be written; the tray then falls back to theme icon names.
#[must_use]
pub fn install() -> Option<PathBuf> {
    let root = crate::popup::runtime_dir().ok()?.join("icons");
    let status = root.join("hicolor/scalable/status");
    put(&root.join("hicolor/index.theme"), INDEX_THEME.as_bytes()).ok()?;
    for state in SyncState::ALL {
        let name = format!("tray-{}", state.slug());
        put(
            &status.join(format!("{APP_ID}-{}.svg", state.slug())),
            &bytes(&name, tray_builtin(state)),
        )
        .ok()?;
    }
    let app = bytes("app", include_bytes!("../../../assets/app.svg"));
    put(
        &root.join(format!("hicolor/scalable/apps/{APP_ID}.svg")),
        &app,
    )
    .ok()?;
    if let Some(data) = data_dir() {
        let _ = put(
            &data.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg")),
            &app,
        );
        if let Ok(exe) = std::env::current_exe() {
            let entry = format!(
                "[Desktop Entry]\nType=Application\nName=OneDrive\nComment=Keeps your OneDrive in sync\nExec={}\nIcon={APP_ID}\nTerminal=false\nCategories=Network;\n",
                exe.display()
            );
            let _ = put(
                &data.join(format!("applications/{APP_ID}.desktop")),
                entry.as_bytes(),
            );
        }
    }
    Some(root)
}
