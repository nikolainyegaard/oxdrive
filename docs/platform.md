# Platform

Linux, Wayland, a desktop with a StatusNotifier host and a compositor that offers `zwlr_layer_shell_v1`. Developed on KDE Plasma 6.7 (KWin). Check what a compositor offers with `wayland-info | grep -E 'zwlr_layer_shell_v1|org_kde_plasma_shell|xdg_activation_v1'`.

## Tray

StatusNotifierItem over the session bus through `ksni`. Our own icons reach the host through `IconThemePath`, a directory in icon theme layout under the runtime directory (design.md); Plasma reads it, the GNOME AppIndicator extension does too. Plasma shows it natively. GNOME needs the AppIndicator extension, same as every other SNI app. The icon is currently an icon-theme name per state (`SyncState::icon_name`); our own pixmaps come with `assets/`.

Left click calls `Tray::activate` with the click position in global screen pixels, which goes out as `TrayEvent::Activate { x }`. The flyout is anchored bottom-left with a left margin that centres it on x, or bottom-right when x is 0 (a shell-triggered Activate, or no click yet). The last x is kept so the flyout can open on its own after a sign-in lands; opened that way it has no focus, so click-off does not close it until it has been clicked once. The right-click menu is built in `Tray::menu`: pause or resume, and Quit.

## Flyout

A layer-shell surface on the `Top` layer, anchored `Bottom | Right`, with an 8 px margin on those two edges, size from `theme::FLYOUT_SIZE`, exclusive zone 0 (it overlaps rather than pushes), keyboard interactivity `OnDemand`. Created and removed per click; it never exists while hidden, which is also what keeps idle memory low.

The surface cannot be dragged. That is by design and matches Windows.

GNOME has no layer-shell. The planned fallback is an ordinary iced window, decided when someone needs it.

## What Wayland will not do

A toplevel cannot be positioned, hidden and re-shown, raised or given focus by the app itself. The settings window is one, so a second Settings press while it is open does nothing; the window stays where the compositor put it. Layer-shell sidesteps positioning. Focus comes through xdg-activation tokens, which Plasma passes to the tray on a real click. See gotchas.md for what this ruled out.
