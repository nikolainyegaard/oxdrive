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
| ICON | 16 | Status mark, sidebar page icons |
| ICON_LARGE | 20 | Cloud and gear in the flyout header, footer buttons |
| ICON_FILE | 32 | File icon on an activity row |
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

The seven states are the Windows set. Our icons: seven SVGs under `assets/tray/`, a white cloud silhouette on a 22 px grid with a badge at the bottom right per state (green check, blue arrows, grey pause, yellow exclamation, red cross, red lock), the disabled cloud at half opacity with no badge. `assets/app.svg` is the same cloud filled blue: the app icon for the window header, the task bar and the launcher. The Microsoft set is reference only; a user who wants it drops it into the override directory (config.md).

## Flyout layout

Top to bottom: title row (blue cloud, `OneDrive - <tenant>` or `OneDrive`, gear at the right for Settings), status headline behind its mark (check in green when idle, sync arrows in blue, pause in yellow, warning triangle in yellow, error cross in red, key in red when blocked), recent activity list (or "No recent activity"), spacer, footer bar on the card colour with three equal buttons, icon over label: folder for Open folder, globe for View online, and the action (pause, play for Resume, key for Sign in): Pause, Resume when paused or stopped, Sign in when blocked.

An activity row is the file name, then `Uploaded to <folder>` or `Deleted from <folder>` with the folder in the accent colour (`OneDrive` for the sync root), then a relative time: `just now`, `14 seconds ago`, `18 hours ago`. The time is computed when the flyout is drawn, so it only moves on the next event.

## In-app icons

Fifteen SVGs under `assets/icons/`, drawn by us: 20 x 20 viewBox, 1.5 px white strokes, round caps, no fill. They are compiled in (`ui::icons`) and recoloured per use through iced's svg style, so one file serves every colour. A file of the same name under `~/.config/oxdrive/icons/` replaces the built-in one at the next view (config.md).

| Name | Used for |
|------|----------|
| cloud, gear | Flyout header |
| check, sync, pause, warning, error, key | Status mark per state; sync, person, bell, info are also the sidebar pages |
| file | Activity row |
| folder, globe, play | Footer buttons |

The tray icons travel as files: `native::icons::install` lays them out as a `hicolor` theme under `$XDG_RUNTIME_DIR/oxdrive/icons/`, which the tray announces as `IconThemePath`, with the icon names `oxdrive-<state>`. When that fails, the tray falls back to theme icon names (`SyncState::icon_name`). The app icon also goes to `~/.local/share/icons/hicolor/scalable/apps/oxdrive.svg` with a desktop entry in `~/.local/share/applications/`, which is how Plasma finds the icon for windows with the app id `oxdrive`.

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
