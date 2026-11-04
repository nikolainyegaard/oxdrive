//! Icon files on disk. Built-in SVGs are compiled in; a file of the same name under
//! `~/.config/oxdrive/icons/` replaces one, so the whole set can be swapped (docs/config.md).
//!
//! The tray gets pixmaps over D-Bus, rasterised here with resvg: Plasma ignored both layouts of
//! `IconThemePath` (probed 2026-11-04), and every host honours `IconPixmap`. The app icon and a
//! desktop entry go where the desktop finds them by app id.

use std::path::{Path, PathBuf};

use oxdrive_core::status::SyncState;

use crate::tray::APP_ID;

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

/// The tray pixmaps for one state, 22 and 44 px, ARGB32 in network byte order as the
/// StatusNotifierItem spec wants them. Empty when the SVG does not parse.
#[must_use]
pub fn tray_pixmaps(state: SyncState) -> Vec<ksni::Icon> {
    let svg = bytes(&format!("tray-{}", state.slug()), tray_builtin(state));
    let Ok(tree) = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default()) else {
        return Vec::new();
    };
    [22, 44]
        .into_iter()
        .filter_map(|size| rasterise(&tree, size))
        .collect()
}

fn rasterise(tree: &resvg::usvg::Tree, size: u32) -> Option<ksni::Icon> {
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;
    let scale = size as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let data = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.alpha(), c.red(), c.green(), c.blue()]
        })
        .collect();
    Some(ksni::Icon {
        width: i32::try_from(size).ok()?,
        height: i32::try_from(size).ok()?,
        data,
    })
}

/// Installs the app icon and desktop entry under the data directory, which is how the desktop
/// finds the icon for windows with our app id. Best effort; nothing to show when it fails.
pub fn install() {
    let Some(data) = data_dir() else { return };
    let app = bytes("app", include_bytes!("../../../assets/app.svg"));
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
