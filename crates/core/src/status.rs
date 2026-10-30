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

    /// Icon theme name used until the project ships its own tray icons.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_has_user_facing_copy() {
        for state in [
            SyncState::Idle,
            SyncState::Syncing,
            SyncState::Paused,
            SyncState::Warning,
            SyncState::Error,
            SyncState::Blocked,
            SyncState::Disabled,
        ] {
            assert!(!state.headline().is_empty());
            assert!(!state.icon_name().is_empty());
        }
    }
}
