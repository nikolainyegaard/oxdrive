//! StatusNotifierItem tray icon over D-Bus through ksni. Runs on its own thread; the UI hears
//! about clicks through a channel and pushes state changes back through the handle.

use futures::channel::mpsc::UnboundedSender;
use ksni::blocking::TrayMethods as _;
use oxdrive_core::status::SyncState;

pub const APP_ID: &str = "oxdrive";

/// What the tray thread tells the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// Left click on the icon.
    Activate,
    Quit,
}

pub struct Tray {
    state: SyncState,
    events: UnboundedSender<TrayEvent>,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        APP_ID.into()
    }
    fn title(&self) -> String {
        "OneDrive".into()
    }
    fn icon_name(&self) -> String {
        self.state.icon_name().into()
    }
    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "OneDrive".into(),
            description: self.state.headline().into(),
            ..Default::default()
        }
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.events.unbounded_send(TrayEvent::Activate);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
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
    }
    .spawn()
    .map(TrayHandle)
    .map_err(|e| format!("could not register the tray icon: {e}"))
}
