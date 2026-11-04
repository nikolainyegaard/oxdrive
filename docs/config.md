# Config and paths

Implemented so far: `native::client::profiles` reads OneDriveGUI's `~/.config/onedrive-gui/profiles` (parsed in `core::profiles`, only `config_file` is used) and falls back to one profile at `~/.config/onedrive`. The app runs the first profile; switching comes with settings. The rest is intent:

- XDG base directories under the app name: `~/.config/oxdrive/` for settings and the profile list, `~/.local/state/oxdrive/` for logs. Directories created with owner-only permissions, files written through a temp file and a rename.
- A profile is a name, the path of a client config directory, and the sign-in address, which gives the flyout its `OneDrive - <tenant>` title through `core::realm` (see client.md). The first import reads OneDriveGUI's `~/.config/onedrive-gui/profiles` so an existing setup carries over.
- No environment variables for user settings.
- Secrets stay with the client; oxdrive stores none.
