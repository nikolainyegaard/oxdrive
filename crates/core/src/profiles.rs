//! OneDriveGUI's `~/.config/onedrive-gui/profiles` file: INI sections named after the account,
//! each with a `config_file` key pointing at the client's config. Only that key is read.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// The client's config directory, the parent of `config_file`.
    pub config_dir: PathBuf,
}

/// Parses the file's text. Sections without a usable `config_file` are skipped.
pub fn parse(text: &str) -> Vec<Profile> {
    let mut profiles = Vec::new();
    let mut name: Option<&str> = None;
    for line in text.lines().map(str::trim) {
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            name = Some(section);
        } else if let Some(section) = name
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "config_file"
            && let Some(dir) = Path::new(value.trim()).parent()
            && !dir.as_os_str().is_empty()
        {
            profiles.push(Profile {
                name: section.to_owned(),
                config_dir: dir.to_path_buf(),
            });
            name = None;
        }
    }
    profiles
}

/// The `sync_dir` from the client's `config` text: unquoted, `~` expanded, defaulting to
/// `~/OneDrive` like the client does.
pub fn sync_dir(config: &str, home: &Path) -> PathBuf {
    let value = config
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "sync_dir")
        .map(|(_, value)| value.trim().trim_matches('"'))
        .filter(|value| !value.is_empty())
        .unwrap_or("~/OneDrive");
    match value.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_dir_is_unquoted_expanded_and_defaulted() {
        let home = Path::new("/home/user");
        assert_eq!(
            sync_dir("sync_dir = \"~/OneDrive_Work\"\n", home),
            PathBuf::from("/home/user/OneDrive_Work")
        );
        assert_eq!(
            sync_dir(
                "# sync_dir = \"/x\"\nsync_dir = \"/srv/od\"\ndrive_id = \"b!x\"\n",
                home
            ),
            PathBuf::from("/srv/od")
        );
        assert_eq!(sync_dir("", home), PathBuf::from("/home/user/OneDrive"));
    }

    #[test]
    fn reads_onedrive_gui_profiles() {
        let text = "[Work]\nconfig_file = /home/user/.config/onedrive/accounts/work/config\nauto_sync = True\n\n[Broken]\nauto_sync = True\n";
        let got = parse(text);
        assert_eq!(
            got,
            vec![Profile {
                name: "Work".into(),
                config_dir: "/home/user/.config/onedrive/accounts/work".into(),
            }]
        );
    }

    #[test]
    fn empty_and_garbage_give_nothing() {
        assert!(parse("").is_empty());
        assert!(parse("config_file = /x/config\n[A]\nconfig_file = \n").is_empty());
    }
}
