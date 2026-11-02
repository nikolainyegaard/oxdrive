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
│   ├── src/status.rs       # SyncState: the seven tray states and their copy
│   └── tests/fixtures/     # real client output, scrubbed; see client.md
├── crates/native/          # everything that touches the OS
│   └── src/tray.rs         # StatusNotifierItem through ksni, on its own thread
├── crates/ui/              # iced views, theme, the layer-shell daemon
│   ├── src/lib.rs          # App state, Message, update, the tray channel bridge, run()
│   ├── src/flyout.rs       # the flyout view
│   └── src/theme.rs        # design tokens
├── assets/                 # our own icons, when drawn
└── docs/
```

Dependency direction is strictly downward: `apps/oxdrive` → `crates/ui` → `crates/native` → `crates/core`. `core` depends on nothing in the workspace and on no external crate. Nothing below the app opens a window.

## Threads and messages

There is no async runtime in our code. The ksni tray service runs on its own thread (ksni's `blocking` mode, backed by `async-io` internally). It sends `TrayEvent`s over a `futures` unbounded channel. The receiving end is parked in a static until iced asks for the subscription, because `Subscription::run` takes a plain function pointer that cannot capture anything. The subscription maps each event to a `Message::Tray`, and `update` reacts.

The flyout is a wlr-layer-shell surface. `update` creates it with `Message::NewLayerShell` on a tray click and removes it with `Message::RemoveWindow` on the next click or on Hide. Those two variants come from the `to_layer_message(multi)` macro on `Message`.

Future blocking work (spawning the client, reading its stdout, writing config files) follows the same shape: a `std::thread` in `crates/native` sending typed events over a channel into a subscription.

## Dependencies

One line each. A new direct dependency gets a row here in the same change.

| Crate | Why |
|-------|-----|
| iced 0.14 | The UI toolkit. Features `wgpu`, `wayland`, `thread-pool` only. |
| iced_layershell 0.19 | wlr-layer-shell surfaces for iced, which is what makes an anchored flyout possible on Wayland. |
| ksni 0.3 | Pure Rust StatusNotifierItem over D-Bus. Features `blocking` and `async-io`; no tokio. |
| futures 0.3 | The channel between the tray thread and the iced subscription, and stream adapters. |
| winit-core =0.31.0-beta.2 | Pin only, see gotchas.md. Not used directly. |
