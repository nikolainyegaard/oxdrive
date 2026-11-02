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
    /// The process is gone. The text says how, for the flyout.
    Exited(String),
}

/// The running monitor process. Dropping it asks the client to stop and waits for it.
pub struct ClientHandle(Child);

impl Drop for ClientHandle {
    fn drop(&mut self) {
        // SIGTERM lets the client checkpoint its database; `Child::kill` would be SIGKILL.
        let _ = Command::new("kill")
            .args(["-TERM", &self.0.id().to_string()])
            .status();
        let _ = self.0.wait();
    }
}

/// Starts `onedrive --monitor` for one config directory.
///
/// # Errors
/// The client binary is missing or could not be started.
pub fn spawn(
    config_dir: &Path,
    events: UnboundedSender<ClientEvent>,
) -> oxdrive_core::Result<ClientHandle> {
    let mut child = Command::new("onedrive")
        .arg("--confdir")
        .arg(config_dir)
        .arg("--monitor")
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
    Ok(ClientHandle(child))
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
