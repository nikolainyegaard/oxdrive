# Config and paths

Implemented so far: `native::client::profiles` reads OneDriveGUI's `~/.config/onedrive-gui/profiles` (parsed in `core::profiles`, only `config_file` is used) and falls back to one profile at `~/.config/onedrive`. The profile list is still OneDriveGUI's; oxdrive only remembers which one runs.

`~/.config/oxdrive/settings` is ours, parsed and rendered by `core::settings`, written by `native::settings` through a temp file and a rename, directory mode 0700:

```
notify_when_paused = false
profile = Nikolai

[Nikolai]
email = user@example.com
title = OneDrive - Example AS
```

Top-level keys are the preferences; one section per profile holds the sign-in address and the title Microsoft answered, so startup needs no network. Autostart is not a key: `~/.config/autostart/oxdrive.desktop` exists or it does not (`native::autostart`). The rest is intent:

- XDG base directories under the app name: `~/.config/oxdrive/` for settings and the profile list, `~/.local/state/oxdrive/` for logs. Directories created with owner-only permissions, files written through a temp file and a rename.
- A profile is a name, the path of a client config directory, and the sign-in address, which gives the flyout its `OneDrive - <tenant>` title through `core::realm` (see client.md). The first import reads OneDriveGUI's `~/.config/onedrive-gui/profiles` so an existing setup carries over.
- No environment variables for user settings.
- Secrets stay with the client; oxdrive stores none.
