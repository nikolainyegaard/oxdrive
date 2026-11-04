//! Icon files on disk. Built-in SVGs are compiled in; a file of the same name under
//! `~/.config/oxdrive/icons/` replaces one, so the whole set can be swapped (docs/config.md).
//!
//! The set is seven slots (`SLOTS`): the app icon, which is also the idle tray icon, the
//! launcher, the window header and the flyout header, and one tray icon per other state. A user
//! set replaces the slots by name under `~/.config/oxdrive/icons/`, from the settings window or
//! by hand.
//!
//! The tray gets pixmaps over D-Bus, rasterised here with resvg: Plasma ignored both layouts of
//! `IconThemePath` (probed 2026-11-04), and every host honours `IconPixmap`.

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

/// The icon slots, file name and label, in the order the settings page shows them.
pub const SLOTS: [(&str, &str); 7] = [
    ("app", "App"),
    ("tray-syncing", "Syncing"),
    ("tray-paused", "Paused"),
    ("tray-warning", "Warning"),
    ("tray-error", "Error"),
    ("tray-blocked", "Sign in"),
    ("tray-disabled", "Stopped"),
];

fn builtin(slot: &str) -> &'static [u8] {
    macro_rules! asset {
        ($path:literal) => {
            include_bytes!(concat!("../../../assets/", $path))
        };
    }
    match slot {
        "tray-syncing" => asset!("tray/syncing.svg"),
        "tray-paused" => asset!("tray/paused.svg"),
        "tray-warning" => asset!("tray/warning.svg"),
        "tray-error" => asset!("tray/error.svg"),
        "tray-blocked" => asset!("tray/blocked.svg"),
        "tray-disabled" => asset!("tray/disabled.svg"),
        _ => asset!("app.svg"),
    }
}

/// The SVG in a slot: the user's file, else ours.
#[must_use]
pub fn slot_svg(slot: &str) -> Vec<u8> {
    bytes(slot, builtin(slot))
}

/// The app icon, the user's `app.svg` or ours.
#[must_use]
pub fn app_svg() -> Vec<u8> {
    slot_svg("app")
}

fn slot_for(state: SyncState) -> String {
    match state {
        SyncState::Idle => "app".to_owned(),
        other => format!("tray-{}", other.slug()),
    }
}

/// Copies one file per slot into the icons directory and refreshes the installed app icon.
///
/// # Errors
/// A file could not be read or written.
pub fn replace_set(picks: &[(&str, &Path)]) -> oxdrive_core::Result<()> {
    use std::os::unix::fs::DirBuilderExt as _;
    let dir = config_dir().ok_or("no home directory")?.join("icons");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    for (slot, from) in picks {
        std::fs::copy(from, dir.join(format!("{slot}.svg")))
            .map_err(|e| format!("could not copy {}: {e}", from.display()))?;
    }
    if installed() {
        install();
    }
    Ok(())
}

/// Removes the user's slot files, so the built-in set shows again.
pub fn clear_set() {
    for (slot, _) in SLOTS {
        if let Some(path) = custom(slot) {
            let _ = std::fs::remove_file(path);
        }
    }
    if installed() {
        install();
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
    let svg = slot_svg(&slot_for(state));
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

fn theme_files() -> Option<(PathBuf, PathBuf)> {
    let data = data_dir()?;
    Some((
        data.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg")),
        data.join(format!("applications/{APP_ID}.desktop")),
    ))
}

/// Whether the app icon sits in the user's icon theme.
#[must_use]
pub fn installed() -> bool {
    theme_files().is_some_and(|(svg, _)| svg.is_file())
}

/// Copies the app icon and a desktop entry into the user's icon theme and application list. The
/// desktop resolves the window header, taskbar and launcher icons by app id from there, never
/// from the running window, so without this they show a generic icon. Best effort.
pub fn install() {
    let Some((svg, desktop)) = theme_files() else {
        return;
    };
    let _ = put(&svg, &app_svg());
    if let Ok(exe) = std::env::current_exe() {
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName=OneDrive\nComment=Keeps your OneDrive in sync\nExec={}\nIcon={APP_ID}\nTerminal=false\nCategories=Network;\n",
            exe.display()
        );
        let _ = put(&desktop, entry.as_bytes());
    }
    icon_changed();
}

/// Removes what `install` wrote.
pub fn uninstall() {
    let Some((svg, desktop)) = theme_files() else {
        return;
    };
    let _ = std::fs::remove_file(svg);
    let _ = std::fs::remove_file(desktop);
    icon_changed();
}

/// Tells KDE's icon loader to drop its cache, so the new icon shows without a new login.
fn icon_changed() {
    let _ = std::process::Command::new("dbus-send")
        .args([
            "--session",
            "--type=signal",
            "/KIconLoader",
            "org.kde.KIconLoader.iconChanged",
            "int32:0",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}
