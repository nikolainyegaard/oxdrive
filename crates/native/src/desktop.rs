//! The desktop's own tools, run rather than linked: `xdg-open`, `notify-send` and the file dialog.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Opens a URL or a path with the default handler. Failures are silent; there is nowhere to show
/// them that the user would be looking at.
pub fn open(target: &str) {
    let _ = Command::new("xdg-open")
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// A desktop notification from OneDrive.
pub fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["--app-name=OneDrive", "--icon=folder-cloud", summary, body])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Asks the user for an SVG file through the desktop's dialog, `kdialog` then `zenity`. Blocks
/// until the dialog closes; `None` when it was cancelled or neither tool is installed.
#[must_use]
pub fn pick_svg() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").unwrap_or_default();
    let kdialog = Command::new("kdialog")
        .args(["--title", "Choose an icon", "--getopenfilename"])
        .arg(&home)
        .arg("image/svg+xml")
        .output();
    let out = match kdialog {
        Ok(out) => out,
        Err(_) => Command::new("zenity")
            .args([
                "--file-selection",
                "--title=Choose an icon",
                "--file-filter=SVG icons | *.svg",
            ])
            .output()
            .ok()?,
    };
    out.status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
        .filter(|p| p.is_file())
}
