//! SharePoint's Sync button: own the `odopen://` scheme, hand the link to the running tray
//! over a socket, and turn it into a profile with its own `drive_id` and folder.

use std::io::{Read as _, Write as _};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;

use futures::channel::mpsc::UnboundedSender;
use oxdrive_core::profiles::Profile;
use oxdrive_core::sharepoint::SyncLink;

use crate::client::ClientEvent;

/// Writes a desktop file that takes `odopen://` links and makes it the default handler.
///
/// # Errors
/// The file could not be written.
pub fn register_handler() -> oxdrive_core::Result<()> {
    let home = std::env::var_os("HOME").ok_or("no home directory")?;
    let dir = PathBuf::from(home).join(".local/share/applications");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("could not find this binary: {e}"))?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=OneDrive\nComment=Keeps your OneDrive in sync\n\
         Exec={} %u\nIcon=oxdrive\nTerminal=false\nNoDisplay=true\nMimeType=x-scheme-handler/odopen;\n",
        exe.display()
    );
    std::fs::write(dir.join("oxdrive.desktop"), entry)
        .map_err(|e| format!("could not write the desktop entry: {e}"))?;
    let _ = std::process::Command::new("xdg-mime")
        .args(["default", "oxdrive.desktop", "x-scheme-handler/odopen"])
        .status();
    Ok(())
}

fn socket_path() -> Option<PathBuf> {
    Some(crate::settings::state_dir()?.join("odopen.sock"))
}

/// Gives the link to a running tray. False when none is listening.
#[must_use]
pub fn hand_off(url: &str) -> bool {
    socket_path()
        .and_then(|p| UnixStream::connect(p).ok())
        .is_some_and(|mut s| s.write_all(url.as_bytes()).is_ok())
}

/// On its own thread: every link written to the socket becomes `ClientEvent::SyncLink`.
pub fn listen(events: UnboundedSender<ClientEvent>) {
    let Some(path) = socket_path() else {
        return;
    };
    let _ = std::fs::remove_file(&path);
    let Ok(listener) = UnixListener::bind(&path) else {
        return;
    };
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut url = String::new();
            // ponytail: a link is a few hundred bytes; a peer that sends more is cut off.
            if stream.take(4096).read_to_string(&mut url).is_ok() {
                let _ = events.unbounded_send(ClientEvent::SyncLink(url.trim().to_owned()));
            }
        }
    });
}

/// On its own thread: looks the library's drive up with the account's token, makes a config
/// directory for it under `~/.config/onedrive/accounts/<Site - Library>` syncing to
/// `~/<tenant folder>/<Site - Library>`, copies the account's token in so it needs no second
/// sign-in, and reports `ClientEvent::Library`.
pub fn add_library(
    link: SyncLink,
    account_dir: &Path,
    tenant_folder: &str,
    events: UnboundedSender<ClientEvent>,
) {
    let account_dir = account_dir.to_owned();
    let tenant_folder = tenant_folder.to_owned();
    thread::spawn(move || {
        let _ = events.unbounded_send(ClientEvent::Library(create(
            &link,
            &account_dir,
            &tenant_folder,
        )));
    });
}

fn create(link: &SyncLink, account_dir: &Path, tenant_folder: &str) -> Result<Profile, String> {
    let token = crate::account::refresh(account_dir, "Files.ReadWrite.All%20Sites.ReadWrite.All")
        .ok_or("could not refresh the sign-in; sign in again and retry")?
        .1;
    let url = format!(
        "https://graph.microsoft.com/v1.0/sites/{}/lists/{}/drive?$select=id",
        link.site_id, link.list_id
    );
    let json = crate::account::graph(&url, &token)
        .ok_or_else(|| format!("could not find the library {}", link.name()))?;
    let json = String::from_utf8_lossy(&json);
    let drive_id = crate::account::field(&json, "id")
        .ok_or("Microsoft returned no drive id")?
        .to_owned();
    let name = link.name();
    let dir = crate::client::create_config_dir(&name)?;
    std::fs::write(
        dir.join("config"),
        format!("sync_dir = \"~/{tenant_folder}/{name}\"\ndrive_id = \"{drive_id}\"\n"),
    )
    .map_err(|e| format!("could not write the library config: {e}"))?;
    std::fs::copy(account_dir.join("refresh_token"), dir.join("refresh_token"))
        .map_err(|e| format!("could not copy the sign-in: {e}"))?;
    Ok(Profile {
        name,
        config_dir: dir,
    })
}
