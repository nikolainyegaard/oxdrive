# Gotchas

Dated. Newest first. Each entry is something that looks wrong until you know why, or a dead end nobody should walk into again.

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
