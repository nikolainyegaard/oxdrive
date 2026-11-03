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

[Unreleased]: https://github.com/nikolainyegaard/oxdrive/commits/main
