//! Our settings file, `~/.config/oxdrive/settings`: `key = value` lines, then one `[section]` per
//! account named after its profile. Autostart is not here; the autostart desktop file is the
//! truth for that. See docs/config.md.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::profiles::Profile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Desktop notification when syncing is paused.
    pub notify_when_paused: bool,
    /// Stop the client while NetworkManager says the connection is metered.
    pub pause_on_metered: bool,
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
    /// The client's config directory. A section without one is not a profile, only a memory.
    pub config_dir: Option<PathBuf>,
    /// For a SharePoint library: the account whose sign-in it uses.
    pub library_of: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notify_when_paused: false,
            pause_on_metered: false,
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
                (None, "pause_on_metered") => settings.pause_on_metered = value == "true",
                (None, "theme_icon") => settings.theme_icon = value != "false",
                (None, "profile") if !value.is_empty() => settings.profile = Some(value.into()),
                (Some(name), "email") => {
                    settings.accounts.entry(name.clone()).or_default().email = value.into();
                }
                (Some(name), "title") if !value.is_empty() => {
                    settings.accounts.entry(name.clone()).or_default().title = Some(value.into());
                }
                (Some(name), "library_of") if !value.is_empty() => {
                    settings
                        .accounts
                        .entry(name.clone())
                        .or_default()
                        .library_of = Some(value.into());
                }
                (Some(name), "config_dir") if !value.is_empty() => {
                    settings
                        .accounts
                        .entry(name.clone())
                        .or_default()
                        .config_dir = Some(value.into());
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
            "notify_when_paused = {}\npause_on_metered = {}\ntheme_icon = {}\n",
            self.notify_when_paused, self.pause_on_metered, self.theme_icon
        );
        if let Some(profile) = &self.profile {
            let _ = writeln!(out, "profile = {profile}");
        }
        for (name, account) in &self.accounts {
            let _ = write!(out, "\n[{name}]\nemail = {}\n", account.email);
            if let Some(title) = &account.title {
                let _ = writeln!(out, "title = {title}");
            }
            if let Some(dir) = &account.config_dir {
                let _ = writeln!(out, "config_dir = {}", dir.display());
            }
            if let Some(owner) = &account.library_of {
                let _ = writeln!(out, "library_of = {owner}");
            }
        }
        out
    }

    /// The accounts with a config directory, by name.
    #[must_use]
    pub fn profiles(&self) -> Vec<Profile> {
        self.accounts
            .iter()
            .filter_map(|(name, a)| {
                Some(Profile {
                    name: name.clone(),
                    config_dir: a.config_dir.clone()?,
                })
            })
            .collect()
    }

    /// The SharePoint libraries that use `account`'s sign-in.
    #[must_use]
    pub fn libraries(&self, account: &str) -> Vec<Profile> {
        self.profiles()
            .into_iter()
            .filter(|p| self.is_library(&p.name))
            .filter(|p| {
                self.accounts
                    .get(&p.name)
                    .and_then(|a| a.library_of.as_deref())
                    == Some(account)
            })
            .collect()
    }

    /// Whether the profile is a library rather than an account.
    #[must_use]
    pub fn is_library(&self, name: &str) -> bool {
        self.accounts
            .get(name)
            .is_some_and(|a| a.library_of.is_some())
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
            pause_on_metered: true,
            theme_icon: false,
            profile: Some("Work".into()),
            accounts: BTreeMap::new(),
        };
        settings.accounts.insert(
            "Work".into(),
            Account {
                email: "user@example.com".into(),
                title: Some("OneDrive - Example AS".into()),
                config_dir: Some("/home/user/.config/onedrive/accounts/work".into()),
                library_of: None,
            },
        );
        settings.accounts.insert("Home".into(), Account::default());
        settings.accounts.insert(
            "Ops - Documents".into(),
            Account {
                config_dir: Some("/x".into()),
                library_of: Some("Work".into()),
                ..Account::default()
            },
        );
        assert_eq!(Settings::parse(&settings.render()), settings);
        assert_eq!(settings.profiles().len(), 2);
        assert_eq!(
            settings.libraries("Work"),
            vec![Profile {
                name: "Ops - Documents".into(),
                config_dir: "/x".into()
            }]
        );
        assert!(settings.libraries("Home").is_empty());
        assert!(!settings.is_library("Work"));
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
