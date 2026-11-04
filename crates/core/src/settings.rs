//! Our settings file, `~/.config/oxdrive/settings`: `key = value` lines, then one `[section]` per
//! account named after its profile. Autostart is not here; the autostart desktop file is the
//! truth for that. See docs/config.md.

use std::collections::BTreeMap;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Desktop notification when syncing is paused.
    pub notify_when_paused: bool,
    /// Copy the app icon into the user's icon theme, which is where the desktop takes the window
    /// header, taskbar and launcher icons from. On until turned off.
    pub theme_icon: bool,
    /// The profile that runs; the first one when unset or unknown.
    pub profile: Option<String>,
    /// Per profile name: what the user typed and what Microsoft answered.
    pub accounts: BTreeMap<String, Account>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Account {
    /// The sign-in address; only its domain is ever sent anywhere.
    pub email: String,
    /// `OneDrive - <tenant>` from `crate::realm::title`, kept so startup needs no network.
    pub title: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notify_when_paused: false,
            theme_icon: true,
            profile: None,
            accounts: BTreeMap::new(),
        }
    }
}

impl Settings {
    /// Reads the file's text. Unknown keys are dropped, missing ones take their default.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        let mut section: Option<String> = None;
        for line in text.lines().map(str::trim) {
            if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = Some(name.to_owned());
                settings.accounts.entry(name.to_owned()).or_default();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match (&section, key) {
                (None, "notify_when_paused") => settings.notify_when_paused = value == "true",
                (None, "theme_icon") => settings.theme_icon = value != "false",
                (None, "profile") if !value.is_empty() => settings.profile = Some(value.into()),
                (Some(name), "email") => {
                    settings.accounts.entry(name.clone()).or_default().email = value.into();
                }
                (Some(name), "title") if !value.is_empty() => {
                    settings.accounts.entry(name.clone()).or_default().title = Some(value.into());
                }
                _ => {}
            }
        }
        settings
    }

    /// The file's text. `parse(render())` gives the same settings back.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = format!(
            "notify_when_paused = {}\ntheme_icon = {}\n",
            self.notify_when_paused, self.theme_icon
        );
        if let Some(profile) = &self.profile {
            let _ = writeln!(out, "profile = {profile}");
        }
        for (name, account) in &self.accounts {
            let _ = write!(out, "\n[{name}]\nemail = {}\n", account.email);
            if let Some(title) = &account.title {
                let _ = writeln!(out, "title = {title}");
            }
        }
        out
    }

    /// The flyout title for a profile, `OneDrive` until its tenant is known.
    #[must_use]
    pub fn title(&self, profile: &str) -> String {
        self.accounts
            .get(profile)
            .and_then(|a| a.title.clone())
            .unwrap_or_else(|| "OneDrive".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut settings = Settings {
            notify_when_paused: true,
            theme_icon: false,
            profile: Some("Work".into()),
            accounts: BTreeMap::new(),
        };
        settings.accounts.insert(
            "Work".into(),
            Account {
                email: "user@example.com".into(),
                title: Some("OneDrive - Example AS".into()),
            },
        );
        settings.accounts.insert("Home".into(), Account::default());
        assert_eq!(Settings::parse(&settings.render()), settings);
        assert_eq!(settings.title("Work"), "OneDrive - Example AS");
        assert_eq!(settings.title("Home"), "OneDrive");
        assert_eq!(settings.title("Nowhere"), "OneDrive");
    }

    #[test]
    fn empty_and_garbage_give_defaults() {
        assert_eq!(Settings::parse(""), Settings::default());
        assert!(Settings::parse("theme_icon = nonsense").theme_icon);
        assert_eq!(
            Settings::parse("profile =\nnonsense\n= x"),
            Settings::default()
        );
    }
}
