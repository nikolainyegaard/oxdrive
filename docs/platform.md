# Platform

Linux, Wayland, a desktop with a StatusNotifier host and a compositor that offers `zwlr_layer_shell_v1`. Developed on KDE Plasma 6.7 (KWin). Check what a compositor offers with `wayland-info | grep -E 'zwlr_layer_shell_v1|org_kde_plasma_shell|xdg_activation_v1'`.

## Tray

StatusNotifierItem over the session bus through `ksni`. Our own icons reach the host as `IconPixmap`, ARGB32 at 22 and 44 px (design.md); `IconThemePath` did not work on Plasma 6. Plasma shows it natively. GNOME needs the AppIndicator extension, same as every other SNI app. Theme icon names (`SyncState::icon_name`) are the fallback when an SVG does not parse.

Left click calls `Tray::activate` with the click position in global screen pixels, which goes out as `TrayEvent::Activate { x }`. The flyout is anchored bottom-left with a left margin that centres it on x, or bottom-right when x is 0 (a shell-triggered Activate, or no click yet). The last x is kept so the flyout can open on its own after a sign-in lands; opened that way it has no focus, so click-off does not close it until it has been clicked once. The right-click menu is built in `Tray::menu`: pause or resume, and Quit.

## Flyout

A layer-shell surface on the `Top` layer, anchored `Bottom | Right`, with an 8 px margin on those two edges, size from `theme::FLYOUT_SIZE`, exclusive zone 0 (it overlaps rather than pushes), keyboard interactivity `OnDemand`. Created and removed per click; it never exists while hidden, which is also what keeps idle memory low.

The surface cannot be dragged. That is by design and matches Windows.

GNOME has no layer-shell. The planned fallback is an ordinary iced window, decided when someone needs it.

## What Wayland will not do

A toplevel cannot be positioned, hidden and re-shown, or raised by the app itself. The settings window is one; it stays where the compositor put it. Layer-shell sidesteps positioning. Focus comes through `xdg_activation_v1`: a second Settings press while the window is open sends `RequestActivationToken` for it (iced_exwlshell 0.21), minted with the serial of the click on the flyout, and `activate` on the token makes the compositor raise and focus the window. Plasma passes a token to the tray on a real click the same way. See gotchas.md for what this ruled out.
