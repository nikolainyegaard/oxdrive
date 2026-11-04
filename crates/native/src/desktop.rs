//! The desktop's own tools, run rather than linked: `xdg-open` and `notify-send`.

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
