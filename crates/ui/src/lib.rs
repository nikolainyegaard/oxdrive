//! iced views, theme and the layer-shell daemon. The flyout is a wlr-layer-shell surface created
//! on tray click and removed on the next click, so it never is a window the user can drag.

mod flyout;
mod settings;
pub mod theme;

use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::stream::BoxStream;
use iced::{Element, Subscription, Task};
use iced_layershell::actions::IcedXdgWindowSettings;
use iced_layershell::reexport::{
    Anchor, IcedId, KeyboardInteractivity, Layer, NewLayerShellSettings,
};
use iced_layershell::settings::{LayerShellSettings, Settings as DaemonSettings, StartMode};
use iced_layershell::to_layer_message;
use oxdrive_core::client::{Event, folder_and_name};
use oxdrive_core::profiles::Profile;
use oxdrive_core::settings::Settings;
use oxdrive_core::status::SyncState;
use oxdrive_native::client::{self, ClientEvent, ClientHandle};
use oxdrive_native::tray::{self, TrayEvent, TrayHandle};
use oxdrive_native::{autostart, desktop, realm};

pub use settings::Page;

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
    /// A surface lost keyboard focus. For the flyout that is how "click outside" reaches us.
    Unfocused(IcedId),
    OpenSettings,
    ShowPage(Page),
    /// The compositor closed a window; only the settings window matters.
    WindowClosed(IcedId),
    SetAutostart(bool),
    SetNotifyPaused(bool),
    EmailChanged(String),
    /// Ask Microsoft for the tenant behind the typed address.
    LookUpTitle,
    TitleFound(oxdrive_core::Result<String>),
    SelectProfile(String),
    OpenUrl(&'static str),
}

/// One row of the flyout's recent activity.
pub struct Activity {
    pub name: String,
    /// `Uploaded to` or `Deleted from`.
    pub action: &'static str,
    pub folder: String,
    pub at: Instant,
}

pub struct App {
    pub state: SyncState,
    /// The flyout header: `OneDrive - <tenant>` once the address is known, else `OneDrive`.
    pub title: String,
    pub account: String,
    pub recent: Vec<Activity>,
    /// Shown under the headline when the client has something to say.
    pub detail: Option<String>,
    profile: Option<Profile>,
    profiles: Vec<Profile>,
    settings: Settings,
    flyout: Option<IcedId>,
    settings_window: Option<IcedId>,
    pub page: Page,
    /// What went wrong on the settings page, if anything.
    pub notice: Option<String>,
    pub autostart: bool,
    /// The sign-in address as typed, saved once a lookup succeeds.
    pub email: String,
    pub free_space: Option<u64>,
    /// `(used, total)` from `--display-quota`, for the storage bar.
    pub quota: Option<(u64, u64)>,
    pub client_version: String,
    /// Where the icon was last clicked, so the flyout can open without a click after a sign-in.
    icon_x: i32,
    /// Dropping the handle removes the icon, so it lives as long as the app.
    tray: Option<TrayHandle>,
    client: Option<ClientHandle>,
    /// Kept so the client subscription stream outlives any one client process.
    client_tx: mpsc::UnboundedSender<ClientEvent>,
}

impl App {
    pub fn notify_when_paused(&self) -> bool {
        self.settings.notify_when_paused
    }

    fn save_settings(&mut self) {
        self.notice = oxdrive_native::settings::save(&self.settings).err();
    }

    /// Switches to a named profile: title, address, and a fresh client.
    fn select_profile(&mut self, name: &str) {
        let Some(profile) = self.profiles.iter().find(|p| p.name == name).cloned() else {
            return;
        };
        self.title = self.settings.title(name);
        self.email = self
            .settings
            .accounts
            .get(name)
            .map(|a| a.email.clone())
            .unwrap_or_default();
        self.account = profile.name.clone();
        self.recent.clear();
        self.free_space = None;
        self.quota = None;
        self.profile = Some(profile);
        self.settings.profile = Some(name.to_owned());
        self.save_settings();
        self.start_client(false, SyncState::Syncing);
    }

    /// Changes the sync state everywhere it shows: the flyout and the tray icon.
    pub fn set_state(&mut self, state: SyncState) {
        self.state = state;
        if let Some(tray) = &self.tray {
            tray.set_state(state);
        }
    }

    fn push_recent(&mut self, action: &'static str, path: &str) {
        let (folder, name) = folder_and_name(path);
        self.recent.insert(
            0,
            Activity {
                name: name.to_owned(),
                action,
                folder: folder.to_owned(),
                at: Instant::now(),
            },
        );
        self.recent.truncate(RECENT_MAX);
    }

    /// Starts a client for the current profile; dropping any old handle stops it first.
    fn start_client(&mut self, reauth: bool, state: SyncState) {
        let Some(profile) = self.profile.clone() else {
            return;
        };
        self.client = None;
        match client::spawn(&profile.config_dir, reauth, self.client_tx.clone()) {
            Ok(handle) => {
                self.client = Some(handle);
                self.detail = None;
                self.set_state(state);
                if !reauth {
                    client::quota(&profile.config_dir, self.client_tx.clone());
                }
            }
            Err(why) => {
                self.detail = Some(why);
                self.set_state(SyncState::Disabled);
            }
        }
    }

    /// Stops the client (SIGTERM) or starts it again for the current profile.
    fn toggle_pause(&mut self) {
        // Paused, or stopped for any reason: start again. Only a running client gets paused.
        if self.state == SyncState::Paused || self.client.is_none() {
            self.start_client(false, SyncState::Syncing);
        } else if self.client.take().is_some() {
            self.set_state(SyncState::Paused);
            if self.settings.notify_when_paused {
                desktop::notify("OneDrive", SyncState::Paused.headline());
            }
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
                    Event::Uploaded(path) => self.push_recent("Uploaded to", path),
                    Event::Deleted(path) => self.push_recent("Deleted from", path),
                    Event::AuthRequired => {
                        self.detail =
                            Some("Your sign-in expired. Sign in again to continue syncing.".into());
                    }
                    Event::SignInOpened => {
                        self.detail = Some("Finish signing in in your browser.".into());
                    }
                    Event::SyncStarted => self.detail = None,
                    Event::FreeSpace(bytes) => self.free_space = Some(*bytes),
                    Event::UsedSpace(used) => {
                        self.quota = Some((*used, self.quota.map_or(0, |(_, total)| total)));
                    }
                    Event::TotalSpace(total) => {
                        self.quota = Some((self.quota.map_or(0, |(used, _)| used), *total));
                    }
                    Event::SyncComplete | Event::AccountType(_) | Event::ShuttingDown => {}
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
        if signed_in && let Some(profile) = &self.profile {
            client::quota(&profile.config_dir, self.client_tx.clone());
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
        Message::Tray(TrayEvent::Activate { .. }) => match app.flyout.take() {
            Some(id) => Task::done(Message::RemoveWindow(id)),
            None => Task::none(),
        },
        // Only the flyout closes on blur; the settings window losing focus is just a window.
        Message::Unfocused(id) if app.flyout == Some(id) => {
            app.flyout = None;
            Task::done(Message::RemoveWindow(id))
        }
        Message::Unfocused(_) => Task::none(),
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
        Message::Tray(TrayEvent::Settings) | Message::OpenSettings => {
            // An open window cannot be raised from here (see platform.md); it stays where it is.
            if app.settings_window.is_some() {
                return Task::none();
            }
            let id = IcedId::unique();
            app.settings_window = Some(id);
            app.notice = None;
            Task::done(Message::NewBaseWindow {
                id,
                settings: IcedXdgWindowSettings {
                    size: Some(theme::SETTINGS_SIZE),
                    client_side_decorations: false,
                },
            })
        }
        Message::ShowPage(page) => {
            app.page = page;
            app.notice = None;
            Task::none()
        }
        Message::WindowClosed(id) => {
            if app.settings_window == Some(id) {
                app.settings_window = None;
            }
            Task::none()
        }
        Message::SetAutostart(on) => {
            match autostart::set(on) {
                Ok(()) => app.autostart = on,
                Err(why) => app.notice = Some(why),
            }
            Task::none()
        }
        Message::SetNotifyPaused(on) => {
            app.settings.notify_when_paused = on;
            app.save_settings();
            Task::none()
        }
        Message::EmailChanged(email) => {
            app.email = email;
            Task::none()
        }
        Message::LookUpTitle => {
            let email = app.email.trim().to_owned();
            if !email.contains('@') {
                app.notice = Some("Type the address you sign in with.".into());
                return Task::none();
            }
            app.notice = Some("Asking Microsoft...".into());
            // ponytail: the lookup blocks one thread of iced's pool for up to ten seconds; a
            // dedicated thread and channel if the pool ever matters
            Task::perform(
                futures::future::lazy(move |_| realm::lookup(&email)),
                Message::TitleFound,
            )
        }
        Message::TitleFound(Ok(title)) => {
            if let Some(name) = app.profile.as_ref().map(|p| p.name.clone()) {
                let account = app.settings.accounts.entry(name).or_default();
                account.email = app.email.trim().to_owned();
                account.title = Some(title.clone());
                app.save_settings();
            }
            app.title = title;
            Task::none()
        }
        Message::TitleFound(Err(why)) => {
            app.notice = Some(why);
            Task::none()
        }
        Message::SelectProfile(name) => {
            app.select_profile(&name);
            Task::none()
        }
        Message::OpenUrl(url) => {
            desktop::open(url);
            Task::none()
        }
        _ => Task::none(),
    }
}

fn view(app: &App, id: IcedId) -> Element<'_, Message> {
    if app.settings_window == Some(id) {
        settings::view(app)
    } else {
        flyout::view(app)
    }
}

fn subscription(_: &App) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run(tray_events),
        Subscription::run(client_events),
        // The flyout takes keyboard focus on demand; a click anywhere else takes it away.
        iced::event::listen_with(|event, _, id| match event {
            iced::Event::Window(iced::window::Event::Unfocused) => Some(Message::Unfocused(id)),
            iced::Event::Window(iced::window::Event::Closed) => Some(Message::WindowClosed(id)),
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
    let settings = oxdrive_native::settings::load();
    let profiles = client::profiles();
    let profile = settings
        .profile
        .as_deref()
        .and_then(|name| profiles.iter().find(|p| p.name == name))
        .or_else(|| profiles.first())
        .cloned();
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
    let (title, email) = profile.as_ref().map_or_else(
        || ("OneDrive".to_owned(), String::new()),
        |p| {
            let email = settings
                .accounts
                .get(&p.name)
                .map(|a| a.email.clone())
                .unwrap_or_default();
            (settings.title(&p.name), email)
        },
    );
    let autostart = autostart::enabled();
    let client_version = client::version().unwrap_or_else(|| "not installed".into());
    let boot = move || App {
        state,
        title: title.clone(),
        profiles: profiles.clone(),
        settings: settings.clone(),
        settings_window: None,
        page: Page::default(),
        notice: None,
        autostart,
        email: email.clone(),
        free_space: None,
        quota: None,
        client_version: client_version.clone(),
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
        .title(|app: &App, id: IcedId| {
            (app.settings_window == Some(id)).then(|| "OneDrive Settings".to_owned())
        })
        .theme(|_: &App, _: IcedId| theme::theme())
        .settings(DaemonSettings {
            layer_settings: LayerShellSettings {
                start_mode: StartMode::Background,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .map_err(|e| format!("could not start the window system connection: {e}"))
}
