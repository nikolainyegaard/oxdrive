//! iced views, theme and the layer-shell daemon. The flyout is a wlr-layer-shell surface created
//! on tray click and removed on the next click, so it never is a window the user can drag.

mod flyout;
pub mod theme;

use std::sync::{Mutex, PoisonError};

use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::stream::BoxStream;
use iced::{Element, Subscription, Task};
use iced_layershell::reexport::{
    Anchor, IcedId, KeyboardInteractivity, Layer, NewLayerShellSettings,
};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use oxdrive_core::status::SyncState;
use oxdrive_native::tray::{self, TrayEvent, TrayHandle};

/// The tray thread's receiver is parked here until the iced runtime asks for the subscription,
/// because `Subscription::run` takes a plain fn pointer and cannot capture it.
static TRAY_RX: Mutex<Option<mpsc::UnboundedReceiver<TrayEvent>>> = Mutex::new(None);

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Tray(TrayEvent),
    Hide,
}

pub struct App {
    pub state: SyncState,
    pub account: String,
    pub recent: Vec<String>,
    flyout: Option<IcedId>,
    /// Dropping the handle removes the icon, so it lives as long as the app.
    tray: Option<TrayHandle>,
}

impl App {
    /// Changes the sync state everywhere it shows: the flyout and the tray icon.
    pub fn set_state(&mut self, state: SyncState) {
        self.state = state;
        if let Some(tray) = &self.tray {
            tray.set_state(state);
        }
    }
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tray(TrayEvent::Activate) if app.flyout.is_none() => {
            let id = IcedId::unique();
            app.flyout = Some(id);
            Task::done(Message::NewLayerShell {
                id,
                settings: NewLayerShellSettings {
                    size: Some(theme::FLYOUT_SIZE),
                    layer: Layer::Top,
                    anchor: Anchor::Bottom | Anchor::Right,
                    margin: Some((0, 0, theme::FLYOUT_MARGIN, theme::FLYOUT_MARGIN)),
                    exclusive_zone: Some(0),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                },
            })
        }
        Message::Tray(TrayEvent::Activate) | Message::Hide => match app.flyout.take() {
            Some(id) => Task::done(Message::RemoveWindow(id)),
            None => Task::none(),
        },
        Message::Tray(TrayEvent::Quit) => iced::exit(),
        _ => Task::none(),
    }
}

fn view(app: &App, _id: IcedId) -> Element<'_, Message> {
    flyout::view(app)
}

fn subscription(_: &App) -> Subscription<Message> {
    Subscription::run(tray_events)
}

fn tray_events() -> BoxStream<'static, Message> {
    let rx = TRAY_RX
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    match rx {
        Some(rx) => rx.map(Message::Tray).boxed(),
        None => futures::stream::pending().boxed(),
    }
}

/// Registers the tray icon and runs the UI until the user quits.
///
/// # Errors
/// Plain text for the terminal: no tray host, or the compositor refused the layer-shell surface.
pub fn run() -> oxdrive_core::Result<()> {
    let (tx, rx) = mpsc::unbounded();
    *TRAY_RX.lock().unwrap_or_else(PoisonError::into_inner) = Some(rx);
    // The boot closure must be `Fn`, so the handle is handed over through an Option.
    let tray = Mutex::new(Some(tray::spawn(tx)?));
    let boot = move || App {
        state: SyncState::Idle,
        account: "Not signed in".into(),
        recent: Vec::new(),
        flyout: None,
        tray: tray.lock().unwrap_or_else(PoisonError::into_inner).take(),
    };
    iced_layershell::daemon(boot, tray::APP_ID, update, view)
        .subscription(subscription)
        .theme(|_: &App, _: IcedId| theme::theme())
        .settings(Settings {
            layer_settings: LayerShellSettings {
                start_mode: StartMode::Background,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .map_err(|e| format!("could not start the window system connection: {e}"))
}
