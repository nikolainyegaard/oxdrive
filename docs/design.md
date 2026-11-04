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
| SETTINGS_SIZE | 900 x 680 | The settings window |
| SIDEBAR_WIDTH | 240 | Settings navigation column |
| TEXT_HEADING | 28 | Settings page title |
| TEXT_SMALL | 12 | Storage line, hints |
| CARD | #2B2B2B | Settings cards and the selected sidebar item |
| CARD_RADIUS | 4 | Card corners |

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

Top to bottom: title row (`OneDrive - <tenant>` or `OneDrive`, Settings at the right), status headline behind a coloured dot (green idle, blue syncing, yellow paused or warning, red otherwise; the dot stands in for the status glyph until our SVG icons land), recent activity list (or "No recent activity"), spacer, footer bar on the card colour with three equal text buttons: Open folder, View online, and the action: Pause, Resume when paused, Sign in when blocked.

An activity row is the file name, then `Uploaded to <folder>` or `Deleted from <folder>` with the folder in the accent colour (`OneDrive` for the sync root), then a relative time: `just now`, `14 seconds ago`, `18 hours ago`. The time is computed when the flyout is drawn, so it only moves on the next event.

## Settings window

Reference is the Windows "OneDrive Settings" window: a normal window, 900 x 680, sidebar at the left with four pages and the storage line at the bottom, one page at the right with a heading and cards. Each card row is a label at the left and, for a switch, "On" or "Off" plus the toggle at the right.

| Page | Rows | Not mirrored |
|------|------|--------------|
| Sync and backup | Preferences: Start OneDrive when I sign in | Back up important folders; pause on a metered network |
| Account | Name and address row with the profile picker when there is more than one and Sign in when blocked; drive card with the title and the storage line; sign-in address card with Look up | Avatar, Add account, unlink, Choose folders |
| Notifications | Notify me when syncing is paused | Loading files from other accounts |
| About | oxdrive version, client version, links: source code, report a problem, the onedrive client | Device and user IDs, insider program |

Sidebar items are flat text; the selected page gets the card colour and a 3 px accent bar at the left. Grey buttons (`theme::BUTTON`) are for actions like Look up and Sign in; only links are blue.

The storage line is `109.18 GB used of 1 TB (10%)` over a 4 px bar, from `--display-quota`, which runs on its own thread each time the monitor starts (not on a reauth run, which has no token yet) and after a sign-in lands. Until it answers the line shows the free space from the monitor's start-up block, and before that "Storage unknown until the client reports it". Sizes use powers of 1024, like the client, so a 1 TiB plan reads `1 TB`.
