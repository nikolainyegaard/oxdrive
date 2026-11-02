# Performance

Budgets are the numbers to hold or beat. Measured on the release profile (fat LTO, one codegen unit, stripped) on Fedora 45, Plasma 6.7, 2026-10-30, power profile `balanced`.

| Measure | 0.1.0 scaffold | with client (2026-11-02) | Budget |
|---------|----------------|--------------------------|--------|
| Stripped binary | 10.3 MB | 10.2 MB | 12 MB |
| Idle RSS, tray only, flyout never opened | 10.0 MB | 10.3 MB | 12 MB |
| RSS with the flyout open | 70.9 MB | 78.2 MB first open, 72.8 MB second | 75 MB |
| RSS after the flyout is closed | not measured | 61.0 MB | none yet |

The open-flyout number is almost all wgpu: the surface, the pipelines and the font atlas. Closing the surface gives back the surface but not the device: RSS settles around 61 MB after the first open and stays there. The first-open figure is 3 MB over budget; the second is under, so part of it is one-time wgpu setup. Not chased yet.

## How to measure

Binary size:

```
cargo build --release && ls -l target/release/oxdrive
```

Memory, with the app running (`pgrep -x oxdrive` gives the pid):

```
ps -o rss= -p $(pgrep -x oxdrive)
```

Open the flyout without clicking, for a scripted check:

```
busctl --user call $(busctl --user list --no-pager | awk '/StatusNotifierItem/{print $1}' | head -1) /StatusNotifierItem org.kde.StatusNotifierItem Activate ii 0 0
```

That picks the first StatusNotifierItem on the bus; with several tray apps running, match the pid with `busctl --user status <name>` first.

State the power profile (`cat /sys/firmware/acpi/platform_profile`) next to every number you record.
