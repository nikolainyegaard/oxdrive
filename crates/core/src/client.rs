//! Turns one line of the onedrive client's stdout into an event. Line formats come from
//! the fixtures in `tests/fixtures/`; see docs/client.md for where each one was seen.

use crate::status::SyncState;

/// What a line of client output means to the tray. Lines that mean nothing give `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SyncStarted,
    SyncComplete,
    Uploaded(String),
    Deleted(String),
    /// Bytes remaining, from the start-up block or from `--display-quota`.
    FreeSpace(u64),
    /// `Total:` and `Used:` from `--display-quota`, in bytes.
    TotalSpace(u64),
    UsedSpace(u64),
    /// `business` or `personal`, as printed.
    AccountType(String),
    /// The client wants `--reauth` and will exit.
    AuthRequired,
    /// The client opened the browser and is waiting on its loopback listener.
    SignInOpened,
    ShuttingDown,
}

impl Event {
    /// The tray state this event moves to, if it moves it at all.
    pub fn state(&self) -> Option<SyncState> {
        match self {
            Self::SyncStarted => Some(SyncState::Syncing),
            Self::SyncComplete => Some(SyncState::Idle),
            Self::AuthRequired | Self::SignInOpened => Some(SyncState::Blocked),
            Self::ShuttingDown => Some(SyncState::Disabled),
            Self::Uploaded(_)
            | Self::Deleted(_)
            | Self::FreeSpace(_)
            | Self::TotalSpace(_)
            | Self::UsedSpace(_)
            | Self::AccountType(_) => None,
        }
    }
}

/// Parse one line, without its trailing newline.
pub fn parse_line(line: &str) -> Option<Event> {
    let line = line.trim_end();
    if line == "Starting a sync with Microsoft OneDrive" {
        return Some(Event::SyncStarted);
    }
    if line == "Sync with Microsoft OneDrive is complete" {
        return Some(Event::SyncComplete);
    }
    if let Some(rest) = line.strip_prefix("Uploading new file: ") {
        let name = rest.strip_suffix(" ... done")?;
        return Some(Event::Uploaded(name.to_owned()));
    }
    if let Some(name) = line.strip_prefix("Successfully deleted item from Microsoft OneDrive: ") {
        return Some(Event::Deleted(name.to_owned()));
    }
    if let Some(rest) = line
        .strip_prefix("Remaining Free Space:")
        .or_else(|| line.strip_prefix("Remaining:"))
    {
        return bytes_in_brackets(rest).map(Event::FreeSpace);
    }
    if let Some(rest) = line.strip_prefix("Total:") {
        return bytes_in_brackets(rest).map(Event::TotalSpace);
    }
    if let Some(rest) = line.strip_prefix("Used:") {
        return bytes_in_brackets(rest).map(Event::UsedSpace);
    }
    if let Some(rest) = line.strip_prefix("Account Type:") {
        return Some(Event::AccountType(rest.trim().to_owned()));
    }
    if line.starts_with("ERROR: You will need to issue a --reauth") {
        return Some(Event::AuthRequired);
    }
    if line.starts_with("Opening the Microsoft authorisation URL") {
        return Some(Event::SignInOpened);
    }
    if line.starts_with("Received termination signal") {
        return Some(Event::ShuttingDown);
    }
    None
}

/// Splits a path the client printed into the folder the flyout names and the file name.
/// Files in the sync root belong to `OneDrive`.
#[must_use]
pub fn folder_and_name(path: &str) -> (&str, &str) {
    let path = path.strip_prefix("./").unwrap_or(path);
    match path.rsplit_once('/') {
        Some((folder, name)) => (folder.rsplit('/').next().unwrap_or(folder), name),
        None => ("OneDrive", path),
    }
}

/// The number in a trailing `(<n> bytes)`.
fn bytes_in_brackets(text: &str) -> Option<u64> {
    let (_, tail) = text.rsplit_once('(')?;
    tail.strip_suffix(" bytes)")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(fixture: &str) -> Vec<Event> {
        fixture.lines().filter_map(parse_line).collect()
    }

    #[test]
    fn monitor_run_with_upload_and_delete() {
        let got = events(include_str!("../tests/fixtures/monitor-upload-delete.txt"));
        assert_eq!(
            got,
            vec![
                Event::AccountType("business".into()),
                Event::FreeSpace(981_377_796_762),
                Event::SyncStarted,
                Event::SyncComplete,
                Event::Uploaded("oxdrive-fixture-test.txt".into()),
                Event::Deleted("oxdrive-fixture-test.txt".into()),
                Event::ShuttingDown,
            ]
        );
    }

    #[test]
    fn expired_auth_blocks() {
        let got = events(include_str!("../tests/fixtures/monitor-auth-expired.txt"));
        assert_eq!(got, vec![Event::AuthRequired]);
        assert_eq!(got[0].state(), Some(SyncState::Blocked));
    }

    #[test]
    fn sign_in_run_blocks_until_the_browser_answers() {
        let got = events(include_str!("../tests/fixtures/monitor-sign-in.txt"));
        assert_eq!(got, vec![Event::SignInOpened]);
        assert_eq!(got[0].state(), Some(SyncState::Blocked));
    }

    #[test]
    fn display_quota_gives_free_space() {
        let got = events(include_str!("../tests/fixtures/display-quota.txt"));
        assert_eq!(
            got,
            vec![
                Event::FreeSpace(981_377_796_762),
                Event::TotalSpace(1_099_511_627_776),
                Event::UsedSpace(117_227_812_949),
            ]
        );
    }

    #[test]
    fn sync_status_and_config_are_silent() {
        assert!(events(include_str!("../tests/fixtures/display-sync-status.txt")).is_empty());
        assert!(events(include_str!("../tests/fixtures/display-config.txt")).is_empty());
    }

    #[test]
    fn paths_split_into_folder_and_name() {
        assert_eq!(folder_and_name("a.txt"), ("OneDrive", "a.txt"));
        assert_eq!(
            folder_and_name("./Screenshots/a.png"),
            ("Screenshots", "a.png")
        );
        assert_eq!(
            folder_and_name("Work/Reports/q3.xlsx"),
            ("Reports", "q3.xlsx")
        );
    }

    #[test]
    fn malformed_lines_are_ignored() {
        assert_eq!(parse_line("Uploading new file: x"), None);
        assert_eq!(parse_line("Remaining: lots (many bytes)"), None);
        assert_eq!(parse_line(""), None);
    }
}
