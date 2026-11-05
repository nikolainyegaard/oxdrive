# Architecture

oxdrive is a tray front end for the [abraunegg/onedrive](https://github.com/abraunegg/onedrive) command line client. The client does the syncing; oxdrive spawns it, reads its output and shows the result the way the Windows OneDrive tray app does.

## Workspace

```
oxdrive/
├── Cargo.toml              # workspace, shared versions, lint set, release profile
├── clippy.toml             # the lint exceptions for tests
├── apps/oxdrive/           # the binary: calls oxdrive_ui::run and maps the error to an exit code
├── crates/core/            # pure logic, no I/O, unit tested
│   ├── src/client.rs       # line parser: client stdout to Event, fixture tested
│   ├── src/profiles.rs     # OneDriveGUI profiles file: name and config dir per account
│   ├── src/realm.rs        # home realm discovery reply to the flyout title; see client.md
│   ├── src/settings.rs     # our settings file: preferences and per-profile account facts
│   ├── src/status.rs       # SyncState: the seven tray states and their copy; Activity rows
│   └── tests/fixtures/     # real client output, scrubbed; see client.md
├── crates/native/          # everything that touches the OS
│   ├── src/autostart.rs    # the XDG autostart desktop file
│   ├── src/icons.rs        # the icon override directory; tray pixmaps, app icon and desktop entry
│   ├── src/client.rs       # spawns onedrive --monitor, reads its output on two threads
│   ├── src/desktop.rs      # xdg-open and notify-send
│   ├── src/popup.rs        # the sign-in popup: xdg-open shim and a Chromium app window
│   ├── src/realm.rs        # asks Microsoft for the tenant name through curl
│   ├── src/settings.rs     # reads and writes ~/.config/oxdrive/settings
│   ├── examples/realm.rs   # prints the title for an address
│   └── src/tray.rs         # StatusNotifierItem through ksni, on its own thread
├── crates/ui/              # iced views, theme, the layer-shell daemon
│   ├── src/lib.rs          # App state, Message, update, the tray channel bridge, run()
│   ├── src/flyout.rs       # the flyout view
│   ├── src/icons.rs        # our SVG glyphs, compiled in, overridable from ~/.config/oxdrive/icons
│   ├── src/settings.rs     # the settings window: sidebar, five pages
│   └── src/theme.rs        # design tokens
├── assets/                 # our own icons, when drawn
└── docs/
```

Dependency direction is strictly downward: `apps/oxdrive` → `crates/ui` → `crates/native` → `crates/core`. `core` depends on nothing in the workspace and on no external crate. Nothing below the app opens a window.

## Threads and messages

There is no async runtime in our code. The ksni tray service runs on its own thread (ksni's `blocking` mode, backed by `async-io` internally). It sends `TrayEvent`s over a `futures` unbounded channel. The receiving end is parked in a static until iced asks for the subscription, because `Subscription::run` takes a plain function pointer that cannot capture anything. The subscription maps each event to a `Message::Tray`, and `update` reacts.

The flyout is a wlr-layer-shell surface. `update` creates it with `Message::NewLayerShell` on a tray click and removes it with `Message::RemoveWindow` on the next click or on Hide. Those two variants come from the `to_layer_message(multi)` macro on `Message`.

The client follows the same shape. `native::client::spawn` starts `onedrive --monitor` with stdout and stderr piped, one reader thread each, both parsing through `core::client::parse_line` and sending `ClientEvent`s into a second parked receiver. `App::on_client` applies them: state changes go to `set_state`, uploads and deletes become activity rows (newest first, ten kept), an exit sets Disabled with the reason as the detail line. Reads are capped at 16 KiB per line.

Pause drops the `ClientHandle` (SIGTERM, reaped on a thread) and sets Paused; resume spawns a new client for the same profile. `App` keeps a clone of the client sender so the subscription stream survives between processes, and `on_client` ignores everything while Paused, because the stopping client still reports its shutdown.

Sign in reruns the client with `--reauth` through the same `start_client` path as resume, with the state set to Blocked. The client opens the browser and waits on its own loopback listener; the parser turns its "Opening the Microsoft authorisation URL" line into `SignInOpened`, which keeps Blocked and sets the detail line, and the first `SyncStarted` clears it and opens the flyout at the last click position, as the Windows client does. oxdrive never sees the URL or the code.

Quit drops the `ClientHandle` before `iced::exit`; the drop sends SIGTERM through `kill` and hands the `Child` to a reaper thread, so the UI never waits. The client finishes its checkpoint on its own, even after oxdrive has exited. The client runs with `--disable-notifications`: oxdrive owns what the desktop shows.

The settings window is an xdg toplevel opened with `Message::NewBaseWindow` from the flyout's Settings button or the tray menu; `view` picks the settings view for that window id and the flyout for everything else, and the daemon's title callback names it. The compositor closes it, which arrives as a `Window::Closed` event with the id. Settings changes are saved on every toggle; the tenant lookup runs on iced's thread pool through `Task::perform` so the UI thread never waits on the network, and its result is written to the profile's section.

The tenant name for the title comes from one `curl` call to Microsoft's home realm discovery, not from the client and not from an HTTP crate: `native::realm::lookup` runs the process, `core::realm::title` reads the one field that matters. It runs once per sign-in address and the result belongs in the profile.

## Dependencies

One line each. A new direct dependency gets a row here in the same change.

| Crate | Why |
|-------|-----|
| iced 0.14 | The UI toolkit. Features `wgpu`, `wayland`, `thread-pool`, and `svg` for the in-app icons (resvg; about 2 MB of binary, measured 2026-11-04). |
| iced_exwlshell 0.21 | wlr-layer-shell surfaces and xdg windows for iced, which is what makes an anchored flyout possible on Wayland. Successor of iced_layershell 0.19; 0.21 sets the window's app id, which the desktop needs to find our icon (gotchas.md). |
| ksni 0.3 | Pure Rust StatusNotifierItem over D-Bus. Features `blocking` and `async-io`; no tokio. |
| futures 0.3 | The channel between the tray thread and the iced subscription, and stream adapters. |
| signal-hook 0.3 | Safe SIGTERM/SIGINT/SIGHUP handling without `unsafe` in our code; a signal becomes a Quit so the client is stopped too. Feature `iterator` only. |
| resvg 0.45 | Rasterises our tray SVGs into the pixmaps the StatusNotifierItem carries; already in the build through iced's `svg` feature, so no new code is linked. No features. |
