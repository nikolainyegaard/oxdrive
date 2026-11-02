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
use oxdrive_core::client::Event;
use oxdrive_core::status::SyncState;
use oxdrive_native::client::{self, ClientEvent, ClientHandle};
use oxdrive_native::tray::{self, TrayEvent, TrayHandle};

/// The tray thread's receiver is parked here until the iced runtime asks for the subscription,
/// because `Subscription::run` takes a plain fn pointer and cannot capture it.
static TRAY_RX: Mutex<Option<mpsc::UnboundedReceiver<TrayEvent>>> = Mutex::new(None);
static CLIENT_RX: Mutex<Option<mpsc::UnboundedReceiver<ClientEvent>>> = Mutex::new(None);

/// Activity rows kept in the flyout.
const RECENT_MAX: usize = 10;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Tray(TrayEvent),
    Client(ClientEvent),
    Hide,
}

pub struct App {
    pub state: SyncState,
    pub account: String,
    pub recent: Vec<String>,
    /// Shown under the headline when the client has something to say.
    pub detail: Option<String>,
    flyout: Option<IcedId>,
    /// Dropping the handle removes the icon, so it lives as long as the app.
    tray: Option<TrayHandle>,
    client: Option<ClientHandle>,
}

impl App {
    /// Changes the sync state everywhere it shows: the flyout and the tray icon.
    pub fn set_state(&mut self, state: SyncState) {
        self.state = state;
        if let Some(tray) = &self.tray {
            tray.set_state(state);
        }
    }

    fn push_recent(&mut self, row: String) {
        self.recent.insert(0, row);
        self.recent.truncate(RECENT_MAX);
    }

    fn on_client(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::Event(event) => {
                match &event {
                    Event::Uploaded(name) => self.push_recent(format!("Uploaded {name}")),
                    Event::Deleted(name) => self.push_recent(format!("Deleted {name}")),
                    Event::AuthRequired => {
                        self.detail =
                            Some("Your sign-in expired. Sign in again to continue syncing.".into());
                    }
                    Event::SyncStarted
                    | Event::SyncComplete
                    | Event::FreeSpace(_)
                    | Event::AccountType(_)
                    | Event::ShuttingDown => {}
                }
                if let Some(state) = event.state() {
                    self.set_state(state);
                }
            }
            ClientEvent::Exited(why) => {
                self.client = None;
                if self.state != SyncState::Blocked {
                    self.detail = Some(why);
                    self.set_state(SyncState::Disabled);
                }
            }
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
        Message::Tray(TrayEvent::Quit) => {
            // Drop first: the handle sends SIGTERM and waits for the client to checkpoint.
            app.client = None;
            iced::exit()
        }
        Message::Client(event) => {
            app.on_client(event);
            Task::none()
        }
        _ => Task::none(),
    }
}

fn view(app: &App, _id: IcedId) -> Element<'_, Message> {
    flyout::view(app)
}

fn subscription(_: &App) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run(tray_events),
        Subscription::run(client_events),
    ])
}

fn client_events() -> BoxStream<'static, Message> {
    let rx = CLIENT_RX
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    match rx {
        Some(rx) => rx.map(Message::Client).boxed(),
        None => futures::stream::pending().boxed(),
    }
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
    // ponytail: first profile only; a profile switcher comes with settings
    let profile = client::profiles().into_iter().next();
    let (client_tx, client_rx) = mpsc::unbounded();
    *CLIENT_RX.lock().unwrap_or_else(PoisonError::into_inner) = Some(client_rx);
    let (client, state, detail) = match &profile {
        Some(p) => match client::spawn(&p.config_dir, client_tx) {
            Ok(handle) => (Some(handle), SyncState::Syncing, None),
            Err(why) => (None, SyncState::Disabled, Some(why)),
        },
        None => (
            None,
            SyncState::Disabled,
            Some("No OneDrive account is set up.".into()),
        ),
    };
    let client = Mutex::new(client);
    let account = profile.map_or_else(|| "Not signed in".into(), |p| p.name);
    let boot = move || App {
        state,
        account: account.clone(),
        recent: Vec::new(),
        detail: detail.clone(),
        flyout: None,
        tray: tray.lock().unwrap_or_else(PoisonError::into_inner).take(),
        client: client.lock().unwrap_or_else(PoisonError::into_inner).take(),
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
