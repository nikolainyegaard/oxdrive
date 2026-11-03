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
use oxdrive_core::profiles::Profile;
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
    TogglePause,
    /// Run the client with `--reauth`; it opens the browser and waits for the sign-in.
    SignIn,
    /// Close the flyout: the surface lost keyboard focus, which is how "click outside" reaches us.
    Hide,
}

pub struct App {
    pub state: SyncState,
    pub account: String,
    pub recent: Vec<String>,
    /// Shown under the headline when the client has something to say.
    pub detail: Option<String>,
    profile: Option<Profile>,
    flyout: Option<IcedId>,
    /// Where the icon was last clicked, so the flyout can open without a click after a sign-in.
    icon_x: i32,
    /// Dropping the handle removes the icon, so it lives as long as the app.
    tray: Option<TrayHandle>,
    client: Option<ClientHandle>,
    /// Kept so the client subscription stream outlives any one client process.
    client_tx: mpsc::UnboundedSender<ClientEvent>,
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

    /// Starts a client for the current profile; dropping any old handle stops it first.
    fn start_client(&mut self, reauth: bool, state: SyncState) {
        let Some(profile) = &self.profile else { return };
        self.client = None;
        match client::spawn(&profile.config_dir, reauth, self.client_tx.clone()) {
            Ok(handle) => {
                self.client = Some(handle);
                self.detail = None;
                self.set_state(state);
            }
            Err(why) => {
                self.detail = Some(why);
                self.set_state(SyncState::Disabled);
            }
        }
    }

    /// Stops the client (SIGTERM) or starts it again for the current profile.
    fn toggle_pause(&mut self) {
        if self.state == SyncState::Paused {
            self.start_client(false, SyncState::Syncing);
        } else if self.client.take().is_some() {
            self.set_state(SyncState::Paused);
        }
    }

    /// Applies a client event. True when a sign-in just landed: the first sync after Blocked.
    fn on_client(&mut self, event: ClientEvent) -> bool {
        // The stopping client still reports its shutdown; a pause is not an error.
        if self.state == SyncState::Paused {
            return false;
        }
        let signed_in =
            self.state == SyncState::Blocked && event == ClientEvent::Event(Event::SyncStarted);
        match event {
            ClientEvent::Event(event) => {
                match &event {
                    Event::Uploaded(name) => self.push_recent(format!("Uploaded {name}")),
                    Event::Deleted(name) => self.push_recent(format!("Deleted {name}")),
                    Event::AuthRequired => {
                        self.detail =
                            Some("Your sign-in expired. Sign in again to continue syncing.".into());
                    }
                    Event::SignInOpened => {
                        self.detail = Some("Finish signing in in your browser.".into());
                    }
                    Event::SyncStarted => self.detail = None,
                    Event::SyncComplete
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
        signed_in
    }
}

/// Creates the flyout surface, centred on x when the host gave a position, else in the corner.
fn open_flyout(app: &mut App, x: i32) -> Task<Message> {
    let id = IcedId::unique();
    app.flyout = Some(id);
    // ponytail: x is global but the margin is per output, so this is right only when the
    // panel's output sits at x = 0; subtract the output origin once iced_layershell exposes it
    let (anchor, margin) = if x > 0 {
        let half = i32::try_from(theme::FLYOUT_SIZE.0 / 2).unwrap_or(0);
        let left = (x - half).max(theme::FLYOUT_MARGIN);
        (
            Anchor::Bottom | Anchor::Left,
            (0, 0, theme::FLYOUT_MARGIN, left),
        )
    } else {
        (
            Anchor::Bottom | Anchor::Right,
            (0, 0, theme::FLYOUT_MARGIN, 0),
        )
    };
    Task::done(Message::NewLayerShell {
        id,
        settings: NewLayerShellSettings {
            size: Some(theme::FLYOUT_SIZE),
            layer: Layer::Top,
            anchor,
            margin: Some(margin),
            exclusive_zone: Some(0),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            ..Default::default()
        },
    })
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tray(TrayEvent::Activate { x }) if app.flyout.is_none() => {
            app.icon_x = x;
            open_flyout(app, x)
        }
        Message::Tray(TrayEvent::Activate { .. }) | Message::Hide => match app.flyout.take() {
            Some(id) => Task::done(Message::RemoveWindow(id)),
            None => Task::none(),
        },
        Message::Tray(TrayEvent::Quit) => {
            // Drop first: the handle sends SIGTERM so the client checkpoints on its way out.
            app.client = None;
            iced::exit()
        }
        Message::Tray(TrayEvent::TogglePause) | Message::TogglePause => {
            app.toggle_pause();
            Task::none()
        }
        Message::SignIn => {
            app.start_client(true, SyncState::Blocked);
            Task::none()
        }
        Message::Client(event) => {
            // A finished sign-in shows the flyout, as if the icon had been clicked.
            if app.on_client(event) && app.flyout.is_none() {
                open_flyout(app, app.icon_x)
            } else {
                Task::none()
            }
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
        // The flyout takes keyboard focus on demand; a click anywhere else takes it away.
        iced::event::listen_with(|event, _, _| match event {
            iced::Event::Window(iced::window::Event::Unfocused) => Some(Message::Hide),
            _ => None,
        }),
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
    tray::quit_on_signal(tx.clone())?;
    let tray = Mutex::new(Some(tray::spawn(tx)?));
    // ponytail: first profile only; a profile switcher comes with settings
    let profile = client::profiles().into_iter().next();
    let (client_tx, client_rx) = mpsc::unbounded();
    *CLIENT_RX.lock().unwrap_or_else(PoisonError::into_inner) = Some(client_rx);
    let (client, state, detail) = match &profile {
        Some(p) => match client::spawn(&p.config_dir, false, client_tx.clone()) {
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
    let account = profile
        .as_ref()
        .map_or_else(|| "Not signed in".into(), |p| p.name.clone());
    let boot = move || App {
        state,
        account: account.clone(),
        recent: Vec::new(),
        detail: detail.clone(),
        profile: profile.clone(),
        flyout: None,
        icon_x: 0,
        tray: tray.lock().unwrap_or_else(PoisonError::into_inner).take(),
        client: client.lock().unwrap_or_else(PoisonError::into_inner).take(),
        client_tx: client_tx.clone(),
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
