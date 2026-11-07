# oxdrive

A OneDrive tray client for Linux, written in Rust. It aims to feel like the Windows OneDrive tray app: a cloud in the system tray, a flyout above it with your account, sync status and recent activity, and the usual actions. Syncing itself is done by the [abraunegg/onedrive](https://github.com/abraunegg/onedrive) client, which oxdrive runs for you.

Early days. The tray icon and the flyout exist, and the client runs in monitor mode for your first OneDriveGUI profile (or `~/.config/onedrive` without one). The tray follows its state and the flyout lists uploads and deletes. Pause and resume live in the tray menu and the flyout. When the sign-in expires, the flyout offers Sign in, which opens the Microsoft login in a small window when Chrome, Chromium, Edge or Brave is installed, else in your browser. Settings opens from the flyout or the tray menu: autostart, the account's sign-in address, which gives the `OneDrive - <tenant>` title, a profile picker, and a pause notification. The Sync button on a SharePoint document library hands its `odopen://` link to the tray, which syncs the library to `~/OneDrive - <tenant>/<Site> - <Library>` with the account's existing sign-in.

## Requirements

- Linux with Wayland and a compositor that supports wlr-layer-shell (KDE Plasma, Sway, Hyprland and other wlroots compositors). GNOME is not supported yet.
- A system tray that speaks StatusNotifierItem (built into Plasma; GNOME needs the AppIndicator extension).
- `onedrive` 2.5.11 or newer.

## Build

```
cargo build --release
target/release/oxdrive
```

## Licence

MIT. See `LICENSE`.
