# Config and paths

Not implemented yet. Intent:

- XDG base directories under the app name: `~/.config/oxdrive/` for settings and the profile list, `~/.local/state/oxdrive/` for logs. Directories created with owner-only permissions, files written through a temp file and a rename.
- A profile is a name plus the path of a client config directory. The first import reads OneDriveGUI's `~/.config/onedrive-gui/profiles` so an existing setup carries over.
- No environment variables for user settings.
- Secrets stay with the client; oxdrive stores none.
