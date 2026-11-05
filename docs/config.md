# Config and paths

Implemented so far: `native::client::profiles` reads OneDriveGUI's `~/.config/onedrive-gui/profiles` (parsed in `core::profiles`, only `config_file` is used) and falls back to one profile at `~/.config/onedrive`. The profile list is still OneDriveGUI's; oxdrive only remembers which one runs.

`~/.config/oxdrive/settings` is ours, parsed and rendered by `core::settings`, written by `native::settings` through a temp file and a rename, directory mode 0700:

```
notify_when_paused = false
theme_icon = true
profile = Nikolai

[Nikolai]
email = user@example.com
title = OneDrive - Example AS
```

Top-level keys are the preferences (`theme_icon`, on by default, keeps a copy of the app icon in `~/.local/share/icons/hicolor/scalable/apps/oxdrive.svg` with a desktop entry in `~/.local/share/applications/`, which is where the desktop takes the window header, taskbar and launcher icons from; off removes both); one section per profile holds the sign-in address and the title Microsoft answered, so startup needs no network. Autostart is not a key: `~/.config/autostart/oxdrive.desktop` exists or it does not (`native::autostart`). The rest is intent:

- XDG base directories under the app name: `~/.config/oxdrive/` for settings and the profile list, `~/.local/state/oxdrive/` for logs, the recent activity list (`<profile>.recent`, one row per line, newest first, ten rows), the last tray click x (`icon.x`) and the sign-in popup's browser profile (`popup-profile/`). Directories created with owner-only permissions, files written through a temp file and a rename.
- A profile is a name, the path of a client config directory, and the sign-in address, which gives the flyout its `OneDrive - <tenant>` title through `core::realm` (see client.md). The first import reads OneDriveGUI's `~/.config/onedrive-gui/profiles` so an existing setup carries over.
- Icons: drop `<name>.svg` into `~/.config/oxdrive/icons/` to replace a built-in one. The icon set is seven slots: `app.svg` (launcher, window header, flyout header and the idle tray icon), then `tray-syncing.svg`, `tray-paused.svg`, `tray-warning.svg`, `tray-error.svg`, `tray-blocked.svg` and `tray-disabled.svg`. The Icons page of the settings window fills them: one box per slot, click to choose an SVG or PNG through `kdialog` or `zenity`, Save copies all seven in (a set is all or nothing; a PNG is stored as an SVG of its own size with the PNG embedded, so everything downstream reads one format) and `Use built-in icons` removes them after a yes in the same dialog tool. The in-app glyphs named in design.md can be replaced by hand only. The tray and app icons are read at start and on Save, the in-app glyphs each time a view is drawn. This is how to put the official OneDrive set in place; oxdrive ships none of it.
- No environment variables for user settings.
- Secrets stay with the client; oxdrive stores none.
