//! StatusNotifierItem tray icon over D-Bus through ksni. Runs on its own thread; the UI hears
//! about clicks through a channel and pushes state changes back through the handle.

use futures::channel::mpsc::UnboundedSender;
use ksni::blocking::TrayMethods as _;
use oxdrive_core::status::SyncState;

pub const APP_ID: &str = "oxdrive";

/// What the tray thread tells the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// Left click on the icon, with the click position in global screen pixels. Plasma passes the
    /// real position; a D-Bus `Activate ii 0 0` from a shell gives `x: 0`.
    Activate {
        x: i32,
    },
    /// "Pause syncing" or "Resume syncing", whichever the menu showed.
    TogglePause,
    Settings,
    Quit,
}

pub struct Tray {
    state: SyncState,
    events: UnboundedSender<TrayEvent>,
    /// Rasterised at start, one entry per state. Empty means theme icon names.
    pixmaps: Vec<(SyncState, Vec<ksni::Icon>)>,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        APP_ID.into()
    }
    fn title(&self) -> String {
        "OneDrive".into()
    }
    fn icon_name(&self) -> String {
        if self.pixmaps.is_empty() {
            self.state.icon_name().into()
        } else {
            String::new()
        }
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.pixmaps
            .iter()
            .find(|(state, _)| *state == self.state)
            .map(|(_, icons)| icons.clone())
            .unwrap_or_default()
    }
    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "OneDrive".into(),
            description: self.state.headline().into(),
            ..Default::default()
        }
    }
    fn activate(&mut self, x: i32, _y: i32) {
        let _ = self.events.unbounded_send(TrayEvent::Activate { x });
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let paused = self.state == SyncState::Paused;
        vec![
            StandardItem {
                label: if paused {
                    "Resume syncing"
                } else {
                    "Pause syncing"
                }
                .into(),
                icon_name: if paused {
                    "media-playback-start"
                } else {
                    "media-playback-pause"
                }
                .into(),
                enabled: paused || self.state != SyncState::Disabled,
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.events.unbounded_send(TrayEvent::TogglePause);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Settings".into(),
                icon_name: "configure".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.events.unbounded_send(TrayEvent::Settings);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit OneDrive".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.events.unbounded_send(TrayEvent::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Turns SIGTERM, SIGINT and SIGHUP into a Quit, so a logout or a `kill` still stops the client.
/// Without this the client would outlive oxdrive and hold the config directory.
///
/// # Errors
/// The signal handler could not be registered.
pub fn quit_on_signal(events: UnboundedSender<TrayEvent>) -> oxdrive_core::Result<()> {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
    let mut signals = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP])
        .map_err(|e| format!("could not listen for termination signals: {e}"))?;
    std::thread::spawn(move || {
        if signals.forever().next().is_some() {
            let _ = events.unbounded_send(TrayEvent::Quit);
        }
    });
    Ok(())
}

/// Handle to the running tray service. Dropping it removes the icon.
pub struct TrayHandle(ksni::blocking::Handle<Tray>);

impl TrayHandle {
    pub fn set_state(&self, state: SyncState) {
        self.0.update(|tray| tray.state = state);
    }
}

/// Starts the tray service on a background thread.
///
/// # Errors
/// Returns a message when no StatusNotifier host is on the session bus (no tray on this desktop).
pub fn spawn(events: UnboundedSender<TrayEvent>) -> oxdrive_core::Result<TrayHandle> {
    Tray {
        state: SyncState::Idle,
        events,
        pixmaps: {
            crate::icons::install();
            SyncState::ALL
                .into_iter()
                .map(|state| (state, crate::icons::tray_pixmaps(state)))
                .filter(|(_, icons)| !icons.is_empty())
                .collect()
        },
    }
    .spawn()
    .map(TrayHandle)
    .map_err(|e| format!("could not register the tray icon: {e}"))
}
