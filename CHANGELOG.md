# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Workspace scaffold: core, native, ui crates and the binary
- Tray icon with the seven sync states and a Quit menu
- Flyout anchored above the tray, toggled by clicking the icon
- Runs the onedrive client in monitor mode for the first OneDriveGUI profile, or `~/.config/onedrive`
- Tray state follows the client: looking for changes, synced, sign in needed, not running
- Pause and resume syncing from the tray menu or the flyout
- Client runs with its own desktop notifications off
- Flyout closes on a click anywhere else; the Hide button is gone
- Flyout centres on the tray icon when the host reports the click position
- SIGTERM, SIGINT and SIGHUP stop the client before oxdrive exits
- Sign in from the flyout when the token has expired; the client opens the browser and syncing resumes and the flyout opens once the sign-in lands
- Recent activity list with uploads and deletes
- Quit stops the client cleanly
- Tenant name lookup for the flyout title, `OneDrive - <tenant>` or `OneDrive - Personal`, through Microsoft's home realm discovery
- Settings window with Sync and backup, Account, Notifications and About pages, from the flyout or the tray menu
- Storage bar and `used of total` line from `--display-quota`
- Flyout activity rows with folder and relative time, status dot, footer bar
- Sign in opens a popup window in a Chromium-family browser when one is installed, closed when the sign-in lands
- Resume also restarts a client that stopped on its own
- Our own icons: seven tray states and the app icon, installed as an icon theme the tray announces, plus in-app glyphs in the flyout header, status line, activity rows, footer and settings sidebar; any of them replaceable from ~/.config/oxdrive/icons
- Desktop entry and app icon under ~/.local/share, so the settings window and the launcher show the OneDrive cloud
- Start OneDrive when I sign in, through an XDG autostart entry
- Profile picker and sign-in address per profile, saved in `~/.config/oxdrive/settings`
- Notify me when syncing is paused, through notify-send

[Unreleased]: https://github.com/nikolainyegaard/oxdrive/commits/main
