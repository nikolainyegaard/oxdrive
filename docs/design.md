# Design

Reference is the Windows 11 OneDrive tray client in dark mode. Match its structure and its copy; use the onedrive client's vocabulary for anything the Windows client has no word for.

## Tokens

All in `crates/ui/src/theme.rs`. Views name a token, never a colour or size.

| Token | Value | Use |
|-------|-------|-----|
| FLYOUT_SIZE | 360 x 520 | The flyout surface |
| FLYOUT_MARGIN | 8 | Gap to the screen edges |
| PADDING | 16 | Flyout padding and section gap |
| GAP | 8 | Inside a section |
| TEXT_TITLE | 20 | "OneDrive" header |
| TEXT_BODY | 14 | Everything else |
| BACKGROUND | #202020 | Flyout background |
| TEXT | #FFFFFF | Primary text |
| TEXT_SECONDARY | #CCCCCC | Account line, empty states |
| ACCENT | #0078D4 | Buttons |
| SUCCESS / WARNING / DANGER | #6CCB5F / #FCE100 / #FF99A4 | State colouring, later |

## Tray states and copy

| State | Headline | Icon (theme name, interim) |
|-------|----------|-----------------------------|
| Idle | Your files are synced | folder-cloud |
| Syncing | Looking for changes | folder-sync |
| Paused | Sync paused | media-playback-pause |
| Warning | Some files didn't sync | dialog-warning |
| Error | Sync problems | dialog-error |
| Blocked | Sign in to sync | dialog-password |
| Disabled | OneDrive isn't running | folder-cloud |

The seven states are the Windows set. Our icons: seven SVGs under `assets/tray/`, one cloud silhouette with a badge per state, drawn by us. The Microsoft set is reference only.

## Flyout layout

Top to bottom: "OneDrive" title, account line, status headline, recent activity list (or "No recent activity"), spacer, footer with Open folder, View online, and Hide at the right. Settings goes into the footer when there is something to set.
