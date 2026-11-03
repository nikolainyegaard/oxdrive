# Gotchas

Dated. Newest first. Each entry is something that looks wrong until you know why, or a dead end nobody should walk into again.

## 2026-11-02: client stderr is read like stdout

The fixtures were captured with `2>&1`, so which stream carries the AADSTS and `ERROR:` lines is unknown. Both pipes go through the same parser; a line is a line. Do not drop stderr when tidying this up.

## 2026-11-02: the client is stopped with `kill -TERM`, not `Child::kill`

`Child::kill` is SIGKILL. The client wants SIGTERM to checkpoint and vacuum its sqlite database (it prints "Received termination signal" and takes a few seconds). Running `kill` is the one-line way to send SIGTERM without a `libc` or `nix` dependency.

## 2026-10-30: winit-core is pinned to 0.31.0-beta.2

`iced_exdevtools 0.19.1`, pulled in by `iced_layershell`, fails to compile against `winit-core 0.31.0-beta.4` (a non-exhaustive match on `NativeKeyCode` in its keymap). Cargo picks beta.4 by default. The pin lives in `apps/oxdrive/Cargo.toml` as a direct dependency with an exact version, because that is the only way to force a pre-release. Check the iced_layershell changelog before every `cargo update` and drop the pin once it releases against a stable winit 0.31.

## 2026-10-30: egui and eframe were tried and rejected

winit 0.30 on Wayland implements `set_visible`, `set_minimized(false)`, `focus_window` and `set_window_level` as no-ops (see `platform_impl/linux/wayland/window/mod.rs` in the crate source). eframe therefore cannot hide and re-show a window, so a tray app is impossible with it there. A bare winit loop with glutin and egui_glow works by creating the window on click and dropping it to hide, but KWin centres the result and the user cannot move an undecorated toplevel. iced_layershell gives an anchored surface in a third of the code. Do not reopen without a new fact, such as layer-shell support landing in winit (rust-windowing/winit issue 2582).

## 2026-10-30: ksni needs the async-io feature next to blocking

`ksni` with `default-features = false, features = ["blocking"]` alone fails inside zbus with "Either async-io (default) or tokio must be enabled". Add `async-io`. Never `tokio`; that would bring a second runtime into the binary.

## 2026-10-30: the iced boot closure must be Fn

`iced_layershell::daemon` takes the boot function as `Fn`, so it cannot move the tray handle into `App` directly. The handle goes through a `Mutex<Option<TrayHandle>>` and is taken on first boot. Same trick for the tray receiver: `Subscription::run` wants a function pointer, so the receiver waits in a static.

## 2026-10-30: a D-Bus Activate carries no activation token

Triggering the tray from a shell with `busctl ... Activate ii 0 0` opens the flyout but, under Wayland, nothing can hand focus to it without an xdg-activation token. Plasma passes a token on a real click (`ksni::current_xdg_activation_token`). Focus behaviour is only tested by clicking.

## 2026-11-03: never wait for the client on the UI thread

`Child::wait` after SIGTERM takes one to two seconds while the client checkpoints. Done in `Drop` on the UI thread it froze the app on every pause. The handle now moves the `Child` to a throwaway thread that does the wait. On Quit that thread dies with the process; the client still finishes its shutdown as an orphan.

## 2026-11-03: the client sends its own desktop notifications

In monitor mode the client notifies on every start and stop of a sync through libnotify. Spawned with `--disable-notifications`, which is a plain flag and does not touch the user's config file.

## 2026-11-03: a signal to oxdrive must reach the client

`kill oxdrive` or a session logout used to leave `onedrive --monitor` running as an orphan, and the next oxdrive start then found the config directory locked and showed "OneDrive isn't running". `tray::quit_on_signal` forwards SIGTERM, SIGINT and SIGHUP as a Quit, which drops the client handle like the menu item does.

## 2026-11-03: the click position is global, the layer-shell margin is per output

Plasma passes the click position to `Activate` in global compositor pixels (observed 4735,1413 on a 5120 wide panel output). A layer-shell margin counts from the edge of the output the surface lands on. iced_layershell does not expose output geometry, so the flyout is centred on the icon only when the panel's output is at x = 0, which it is on this machine. A margin past the output's right edge would push the surface off screen; nothing clamps it yet.
