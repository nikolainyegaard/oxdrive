//! The sign-in popup: a Chromium-family browser in `--app` mode, so the Microsoft login opens
//! as a small window like the Windows client's, not a tab.
//!
//! The onedrive client ignores `$BROWSER` and runs `xdg-open <url>` from `PATH` (v2.5.11,
//! probed 2026-11-04). So the `--reauth` child gets a directory at the front of its `PATH`
//! holding an `xdg-open` symlink back to this binary; `main` sees it was called by that name
//! and opens the popup instead of starting the tray.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BROWSERS: [&str; 7] = [
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "microsoft-edge",
    "brave-browser",
    "brave",
];

/// The first Chromium-family browser on `PATH`.
#[must_use]
pub fn browser() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| BROWSERS.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

/// `PATH` for the `--reauth` child, with our `xdg-open` shim first. `None` when there is no
/// browser for a popup or the shim could not be made; the client then uses the real xdg-open.
#[must_use]
pub fn child_path() -> Option<std::ffi::OsString> {
    browser()?;
    let dir = shim_dir().ok()?;
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::join_paths(std::iter::once(dir).chain(std::env::split_paths(&path))).ok()
}

/// True when this process was started as `xdg-open`: the client wants a URL opened.
#[must_use]
pub fn called_as_shim() -> bool {
    std::env::args_os()
        .next()
        .map(PathBuf::from)
        .and_then(|exe| exe.file_name().map(|n| n == "xdg-open"))
        .unwrap_or(false)
}

/// Opens the URL as an app window in a profile of its own, so the window is a process we own
/// and `close` can end it once the sign-in lands. The profile lives in the state directory,
/// so the Microsoft session survives and a later sign-in is one click, not a full login.
/// Returns as soon as the browser is started.
///
/// # Errors
/// No browser, or it could not be started.
pub fn open(url: &str) -> oxdrive_core::Result<()> {
    let browser = browser().ok_or("no Chromium-family browser for the sign-in popup")?;
    let dir = runtime_dir()?;
    let profile = crate::settings::state_dir()
        .ok_or("no state directory for the sign-in popup")?
        .join("popup-profile");
    let child = Command::new(browser)
        .arg(format!("--app={url}"))
        .arg(format!("--user-data-dir={}", profile.display()))
        .args([
            "--window-size=480,720",
            "--no-first-run",
            "--no-default-browser-check",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("could not open the sign-in popup: {e}"))?;
    std::fs::write(dir.join("popup.pid"), child.id().to_string())
        .map_err(|e| format!("could not note the popup: {e}"))
}

/// Closes the popup `open` started, if one is still around. Nothing to show on failure.
pub fn close() {
    let Ok(dir) = runtime_dir() else { return };
    let pid_file = dir.join("popup.pid");
    if let Ok(pid) = std::fs::read_to_string(&pid_file) {
        let _ = Command::new("kill").args(["-TERM", pid.trim()]).status();
    }
    let _ = std::fs::remove_file(pid_file);
}

pub(crate) fn runtime_dir() -> oxdrive_core::Result<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".cache")))
        .ok_or("no runtime directory for the sign-in popup")?;
    let dir = base.join("oxdrive");
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not make {}: {e}", dir.display()))?;
    Ok(dir)
}

fn shim_dir() -> oxdrive_core::Result<PathBuf> {
    let dir = runtime_dir()?.join("bin");
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not make {}: {e}", dir.display()))?;
    let shim = dir.join("xdg-open");
    let exe = std::env::current_exe().map_err(|e| format!("no path to oxdrive: {e}"))?;
    if std::fs::read_link(&shim).ok().as_ref() != Some(&exe) {
        let _ = std::fs::remove_file(&shim);
        std::os::unix::fs::symlink(&exe, &shim)
            .map_err(|e| format!("could not link {}: {e}", shim.display()))?;
    }
    Ok(dir)
}
