//! The seven tray states, mirrored from the Windows client, and what the flyout says in each.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncState {
    #[default]
    Idle,
    Syncing,
    Paused,
    Warning,
    Error,
    Blocked,
    Disabled,
}

impl SyncState {
    /// Status line under the account name.
    pub fn headline(self) -> &'static str {
        match self {
            Self::Idle => "Your files are synced",
            Self::Syncing => "Looking for changes",
            Self::Paused => "Sync paused",
            Self::Warning => "Some files didn't sync",
            Self::Error => "Sync problems",
            Self::Blocked => "Sign in to sync",
            Self::Disabled => "OneDrive isn't running",
        }
    }

    pub const ALL: [Self; 7] = [
        Self::Idle,
        Self::Syncing,
        Self::Paused,
        Self::Warning,
        Self::Error,
        Self::Blocked,
        Self::Disabled,
    ];

    /// The lowercase name: file stems under `assets/tray/` and the `tray-<slug>` override.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Syncing => "syncing",
            Self::Paused => "paused",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Blocked => "blocked",
            Self::Disabled => "disabled",
        }
    }

    /// Icon theme name, the fallback when our own tray icons could not be installed.
    pub fn icon_name(self) -> &'static str {
        match self {
            Self::Idle => "folder-cloud",
            Self::Syncing => "folder-sync",
            Self::Paused => "media-playback-pause",
            Self::Warning => "dialog-warning",
            Self::Error => "dialog-error",
            Self::Blocked => "dialog-password",
            Self::Disabled => "folder-cloud",
        }
    }
}

/// Bytes as the Windows client shows them: `913.98 GB`, `1 TB`. Powers of 1024, like the
/// client's own `--display-quota` output, so a 1 TiB plan reads `1 TB`.
#[must_use]
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = UNITS.iter();
    let mut name = unit.next().unwrap_or(&"bytes");
    while size >= 1024.0
        && let Some(next) = unit.next()
    {
        size /= 1024.0;
        name = next;
    }
    if size.fract() == 0.0 || *name == "bytes" {
        format!("{size:.0} {name}")
    } else {
        format!("{size:.2} {name}")
    }
}

/// `109.18 GB used of 1 TB (11%)`, the storage line under the settings sidebar.
#[must_use]
pub fn storage_line(used: u64, total: u64) -> String {
    let percent = used.saturating_mul(100).checked_div(total).unwrap_or(0);
    format!(
        "{} used of {} ({percent}%)",
        human_size(used),
        human_size(total)
    )
}

/// `just now`, `14 seconds ago`, `18 hours ago`: the activity timestamp.
#[must_use]
pub fn ago(seconds: u64) -> String {
    let (count, unit) = match seconds {
        0..=4 => return "just now".to_owned(),
        s @ 5..=59 => (s, "second"),
        s @ 60..=3599 => (s / 60, "minute"),
        s @ 3600..=86_399 => (s / 3600, "hour"),
        s => (s / 86_400, "day"),
    };
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} {unit}{plural} ago")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_the_windows_client() {
        assert_eq!(human_size(0), "0 bytes");
        assert_eq!(human_size(981_377_796_762), "913.98 GB");
        assert_eq!(human_size(1_099_511_627_776), "1 TB");
        assert_eq!(
            storage_line(117_227_812_949, 1_099_511_627_776),
            "109.18 GB used of 1 TB (10%)"
        );
        assert_eq!(storage_line(1, 0), "1 bytes used of 0 bytes (0%)");
    }

    #[test]
    fn timestamps_read_like_the_windows_flyout() {
        assert_eq!(ago(0), "just now");
        assert_eq!(ago(14), "14 seconds ago");
        assert_eq!(ago(60), "1 minute ago");
        assert_eq!(ago(18 * 3600), "18 hours ago");
        assert_eq!(ago(3 * 86_400), "3 days ago");
    }

    #[test]
    fn every_state_has_user_facing_copy() {
        for state in SyncState::ALL {
            assert!(!state.headline().is_empty());
            assert!(!state.icon_name().is_empty());
            assert!(!state.slug().is_empty());
        }
    }
}

/// One row of the flyout's recent activity. Kept on disk between runs (native::settings), one
/// row per line: `<unix seconds>\t<action>\t<folder>\t<name>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub name: String,
    /// `Uploaded to` or `Deleted from`.
    pub action: &'static str,
    pub folder: String,
    /// Unix seconds.
    pub at: u64,
}

impl Activity {
    /// Parses one saved line; `None` for anything we did not write.
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let mut fields = line.splitn(4, '\t');
        let at = fields.next()?.parse().ok()?;
        let action = match fields.next()? {
            "Uploaded to" => "Uploaded to",
            "Deleted from" => "Deleted from",
            _ => return None,
        };
        Some(Self {
            folder: fields.next()?.to_owned(),
            name: fields.next()?.to_owned(),
            action,
            at,
        })
    }

    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.at, self.action, self.folder, self.name
        )
    }
}

#[cfg(test)]
mod activity_tests {
    use super::Activity;

    #[test]
    fn activity_rows_survive_a_round_trip() {
        let row = Activity {
            name: "Overtid 2026-10-31.xlsx".into(),
            action: "Uploaded to",
            folder: "Microsoft Teams Chat Files".into(),
            at: 1_762_250_000,
        };
        assert_eq!(Activity::parse(&row.render()), Some(row));
        assert_eq!(Activity::parse("x\tUploaded to\ta\tb"), None);
        assert_eq!(Activity::parse("1\tRenamed\ta\tb"), None);
    }
}
