//! The onedrive client as a child process. Its stdout and stderr are read line by line on two
//! threads and parsed by `oxdrive_core::client`; the UI hears the result through a channel.

use std::io::{BufRead as _, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;

use futures::channel::mpsc::UnboundedSender;
use oxdrive_core::client::{Event, parse_line};
use oxdrive_core::profiles::{self, Profile};

/// Longest line read before the rest is dropped; the client prints paths, nothing longer.
const MAX_LINE: u64 = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent {
    Event(Event),
    /// From `account::fetch`: the sign-in address, the flyout title, the photo.
    Email(String),
    Title(String),
    Avatar(Vec<u8>),
    /// The process is gone. The text says how, for the flyout.
    Exited(String),
}

/// The running monitor process. Dropping it asks the client to stop; a reaper thread waits
/// so the UI thread never does. The client takes a second or two to checkpoint and exit.
pub struct ClientHandle(Option<Child>);

impl Drop for ClientHandle {
    fn drop(&mut self) {
        let Some(mut child) = self.0.take() else {
            return;
        };
        // SIGTERM lets the client checkpoint its database; `Child::kill` would be SIGKILL.
        let _ = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status();
        thread::spawn(move || child.wait());
    }
}

/// Starts `onedrive --monitor` for one config directory. With `reauth` the client drops its
/// token first, opens the browser itself and keeps syncing once the sign-in lands.
///
/// # Errors
/// The client binary is missing or could not be started.
pub fn spawn(
    config_dir: &Path,
    reauth: bool,
    events: UnboundedSender<ClientEvent>,
) -> oxdrive_core::Result<ClientHandle> {
    let mut command = Command::new("onedrive");
    command
        .arg("--confdir")
        .arg(config_dir)
        .arg("--monitor")
        .arg("--disable-notifications");
    if reauth {
        command.arg("--reauth");
        if let Some(path) = crate::popup::child_path() {
            command.env("PATH", path);
        }
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start the onedrive client: {e}"))?;
    if let Some(out) = child.stdout.take() {
        let tx = events.clone();
        thread::spawn(move || read_lines(out, &tx));
    }
    if let Some(err) = child.stderr.take() {
        thread::spawn(move || read_lines(err, &events));
    }
    Ok(ClientHandle(Some(child)))
}

fn read_lines(source: impl Read, events: &UnboundedSender<ClientEvent>) {
    let mut reader = BufReader::new(source);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.by_ref().take(MAX_LINE).read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                if let Some(event) = parse_line(&line)
                    && events.unbounded_send(ClientEvent::Event(event)).is_err()
                {
                    break;
                }
            }
        }
    }
    let _ = events.unbounded_send(ClientEvent::Exited(
        "The onedrive client stopped. Check its output for the reason.".into(),
    ));
}

/// Runs `onedrive --display-quota` on its own thread and reports the `Total:` and `Used:` lines.
/// The client allows it beside a running monitor; it only reads the token and asks Graph.
pub fn quota(config_dir: &Path, events: UnboundedSender<ClientEvent>) {
    let config_dir = config_dir.to_owned();
    thread::spawn(move || {
        let Ok(output) = Command::new("onedrive")
            .arg("--confdir")
            .arg(config_dir)
            .arg("--display-quota")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        else {
            return;
        };
        for event in String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_line)
        {
            if events.unbounded_send(ClientEvent::Event(event)).is_err() {
                break;
            }
        }
    });
}

/// Profiles from OneDriveGUI's file, or a single default pointing at `~/.config/onedrive`.
pub fn profiles() -> Vec<Profile> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    let gui = home.join(".config/onedrive-gui/profiles");
    let found = std::fs::read_to_string(&gui)
        .map(|text| profiles::parse(&text))
        .unwrap_or_default();
    if found.is_empty() {
        vec![Profile {
            name: "OneDrive".into(),
            config_dir: home.join(".config/onedrive"),
        }]
    } else {
        found
    }
}

/// A config directory for a new profile, `~/.config/onedrive/accounts/<name>`, with a `config`
/// naming `~/OneDrive_<name>` as the sync folder so profiles do not share one. An existing
/// `config` is kept.
///
/// # Errors
/// The directory or file could not be made.
pub fn create_config_dir(name: &str) -> oxdrive_core::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let dir = home.join(".config/onedrive/accounts").join(name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not make {}: {e}", dir.display()))?;
    let config = dir.join("config");
    if !config.exists() {
        std::fs::write(&config, format!("sync_dir = \"~/OneDrive_{name}\"\n"))
            .map_err(|e| format!("could not write {}: {e}", config.display()))?;
    }
    Ok(dir)
}

/// The profile's local sync folder, read from its `config` file.
#[must_use]
pub fn sync_dir(profile: &Profile) -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let config = std::fs::read_to_string(profile.config_dir.join("config")).unwrap_or_default();
    profiles::sync_dir(&config, &home)
}

/// `onedrive --version` as printed, for the About page.
#[must_use]
pub fn version() -> Option<String> {
    let output = Command::new("onedrive").arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().next()?.trim().to_owned()).filter(|s| !s.is_empty())
}
