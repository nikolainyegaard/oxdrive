//! The settings window: a sidebar with five pages and the storage line, like the Windows client.

use iced::widget::svg::Handle;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, space, text, text_input, toggler,
};
use iced::{Alignment, Border, Element, Length, Theme};

use oxdrive_core::status::{SyncState, human_size, storage_line};

use crate::icons::{Icon, icon, picture};
use crate::theme;
use crate::{App, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    SyncAndBackup,
    Account,
    Notifications,
    Icons,
    About,
}

impl Page {
    const ALL: [Self; 5] = [
        Self::SyncAndBackup,
        Self::Account,
        Self::Notifications,
        Self::Icons,
        Self::About,
    ];

    fn icon(self) -> Icon {
        match self {
            Self::SyncAndBackup => Icon::Sync,
            Self::Account => Icon::Person,
            Self::Notifications => Icon::Bell,
            Self::Icons => Icon::Image,
            Self::About => Icon::Info,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SyncAndBackup => "Sync and backup",
            Self::Account => "Account",
            Self::Notifications => "Notifications",
            Self::Icons => "Icons",
            Self::About => "About",
        }
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let nav = column(Page::ALL.map(|page| {
        let selected = page == app.page;
        let marker = container(space::horizontal())
            .width(3)
            .height(16)
            .style(move |_| container::Style {
                background: selected.then(|| theme::ACCENT.into()),
                border: Border {
                    radius: 2.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            });
        button(
            row![
                marker,
                icon(page.icon(), theme::ICON, theme::TEXT),
                text(page.label()).size(theme::TEXT_BODY)
            ]
            .spacing(theme::GAP + 4.0)
            .align_y(Alignment::Center),
        )
        .style(move |_, status| nav_style(selected, status))
        .padding([theme::GAP, theme::GAP])
        .width(Length::Fill)
        .on_press(Message::ShowPage(page))
        .into()
    }))
    .spacing(4);

    let storage = column![text("Storage").size(theme::TEXT_BODY).font(theme::SEMIBOLD)]
        .extend(app.quota.map(|(used, total)| {
            progress_bar(0.0..=1.0, used as f32 / total.max(1) as f32)
                .girth(4)
                .into()
        }))
        .push(
            text(storage_text(app))
                .size(theme::TEXT_SMALL)
                .color(theme::TEXT_SECONDARY),
        )
        .spacing(theme::GAP)
        .padding([0.0, theme::PADDING]);

    let sidebar = column![nav, space::vertical(), storage]
        .width(theme::SIDEBAR_WIDTH)
        .height(Length::Fill)
        .spacing(theme::PADDING);

    let page: Element<'_, Message> = match app.page {
        Page::SyncAndBackup => sync_and_backup(app),
        Page::Account => account(app),
        Page::Notifications => notifications(app),
        Page::Icons => icons(app),
        Page::About => about(app),
    };
    let content = column![
        text(app.page.label())
            .size(theme::TEXT_HEADING)
            .font(theme::SEMIBOLD),
        page
    ]
    .extend(app.notice.as_deref().map(|notice| {
        text(notice)
            .size(theme::TEXT_BODY)
            .color(theme::WARNING)
            .into()
    }))
    .spacing(theme::PADDING * 1.5)
    .width(Length::Fill)
    .padding([theme::GAP, theme::PADDING]);

    row![sidebar, content]
        .spacing(theme::PADDING)
        .padding([theme::PADDING * 2.0, theme::PADDING])
        .into()
}

fn nav_style(selected: bool, status: button::Status) -> button::Style {
    let background = match (selected, status) {
        (true, _) | (false, button::Status::Pressed) => Some(theme::CARD.into()),
        (false, button::Status::Hovered) => Some(theme::HOVER.into()),
        (false, _) => None,
    };
    button::Style {
        background,
        text_color: theme::TEXT,
        border: Border {
            radius: theme::CARD_RADIUS.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn grey(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(
            match status {
                button::Status::Hovered => theme::BUTTON_HOVER,
                _ => theme::BUTTON,
            }
            .into(),
        ),
        text_color: theme::TEXT,
        border: Border {
            radius: theme::CARD_RADIUS.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn sync_and_backup(app: &App) -> Element<'_, Message> {
    section(
        "Preferences",
        card(toggle_row(
            "Start OneDrive when I sign in",
            app.autostart,
            Message::SetAutostart,
        )),
    )
}

fn account(app: &App) -> Element<'_, Message> {
    let names: Vec<String> = app.profiles.iter().map(|p| p.name.clone()).collect();
    let profile = app.profile.as_ref().map(|p| p.name.clone());
    let mut who = row![
        column![
            text(&app.account)
                .size(theme::TEXT_BODY)
                .font(theme::SEMIBOLD),
            text(&app.email)
                .size(theme::TEXT_BODY)
                .color(theme::TEXT_SECONDARY),
        ]
        .spacing(4),
        space::horizontal(),
    ]
    .align_y(Alignment::Center);
    if names.len() > 1 {
        who =
            who.push(pick_list(names, profile, Message::SelectProfile).text_size(theme::TEXT_BODY));
    }
    if app.state == SyncState::Blocked {
        who = who.push(
            button(text("Sign in").size(theme::TEXT_BODY))
                .style(grey)
                .on_press(Message::SignIn),
        );
    }

    let drive = card(
        row![
            column![
                text(&app.title).size(theme::TEXT_BODY),
                text(storage_text(app))
                    .size(theme::TEXT_SMALL)
                    .color(theme::TEXT_SECONDARY),
            ]
            .spacing(4),
            space::horizontal(),
        ]
        .align_y(Alignment::Center),
    );

    let lookup = card(
        column![
            text("Sign-in address").size(theme::TEXT_BODY),
            text("Used only to look up the account name")
                .size(theme::TEXT_SMALL)
                .color(theme::TEXT_SECONDARY),
            row![
                text_input("you@example.com", &app.email)
                    .size(theme::TEXT_BODY)
                    .on_input(Message::EmailChanged)
                    .on_submit(Message::LookUpTitle),
                button(text("Look up").size(theme::TEXT_BODY))
                    .style(grey)
                    .on_press(Message::LookUpTitle),
            ]
            .spacing(theme::GAP)
            .align_y(Alignment::Center),
        ]
        .spacing(theme::GAP),
    );

    column![who, drive, lookup]
        .spacing(theme::PADDING * 1.5)
        .into()
}

fn notifications(app: &App) -> Element<'_, Message> {
    card(toggle_row(
        "Notify me when syncing is paused",
        app.notify_when_paused(),
        Message::SetNotifyPaused,
    ))
    .into()
}

/// Two rows of slots: the icons in use above on the window colour, the files picked for the new
/// set below on buttons. Save copies the files in; a set is all seven or nothing.
fn icons(app: &App) -> Element<'_, Message> {
    use oxdrive_native::icons::{SLOTS, read, slot_svg};
    const BOX: f32 = 64.0;
    fn current(content: Element<'_, Message>) -> container::Container<'_, Message> {
        container(content)
            .width(BOX)
            .height(BOX)
            .center(BOX)
            .style(|_| container::Style {
                background: Some(theme::BACKGROUND.into()),
                border: Border {
                    radius: theme::CARD_RADIUS.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            })
    }
    let slot = |i: usize, (name, label): (&'static str, &'static str)| {
        let in_use = picture(Handle::from_memory(slot_svg(name)), theme::ICON_FILE);
        let picked: Element<'_, Message> = match app
            .icon_picks
            .get(i)
            .and_then(|p| p.as_ref())
            .and_then(|p| read(p))
        {
            Some(svg) => picture(Handle::from_memory(svg), theme::ICON_FILE),
            None => text("+")
                .size(theme::TEXT_HEADING)
                .color(theme::TEXT_SECONDARY)
                .into(),
        };
        column![
            current(in_use),
            text(label)
                .size(theme::TEXT_SMALL)
                .color(theme::TEXT_SECONDARY),
            icon(Icon::ArrowDown, theme::ICON, theme::TEXT_SECONDARY),
            button(container(picked).center(BOX))
                .style(grey)
                .padding(0)
                .width(BOX)
                .height(BOX)
                .on_press(Message::PickIcon(i)),
        ]
        .spacing(4)
        .width(BOX)
        .align_x(Alignment::Center)
        .into()
    };
    let slots = row(SLOTS.into_iter().enumerate().map(|(i, s)| slot(i, s))).spacing(theme::GAP);
    column![
        section("Icon set", card(column![
            slots,
            row![
                button(text("Save").size(theme::TEXT_BODY))
                    .style(grey)
                    .padding([theme::GAP, theme::PADDING])
                    .on_press(Message::SaveIcons),
                button(text("Use built-in icons").size(theme::TEXT_BODY))
                    .style(grey)
                    .padding([theme::GAP, theme::PADDING])
                    .on_press(Message::ResetIcons),
            ]
            .spacing(theme::GAP),
        ]
        .spacing(theme::PADDING))),
        text("The first icon is the app: launcher, window header, flyout header and the tray when everything is synced. The rest are the tray in each other state. Click a box to choose an SVG or PNG file.")
            .size(theme::TEXT_SMALL)
            .color(theme::TEXT_SECONDARY),
        section(
            "Desktop",
            card(toggle_row(
                "Use the app icon for the window header, taskbar and launcher",
                app.settings.theme_icon,
                Message::SetThemeIcon,
            )),
        ),
    ]
    .spacing(theme::PADDING)
    .into()
}

fn about(app: &App) -> Element<'_, Message> {
    let link = |label: &'static str, url: &'static str| {
        button(text(label).size(theme::TEXT_BODY).color(theme::ACCENT))
            .style(button::text)
            .padding(0)
            .on_press(Message::OpenUrl(url))
    };
    column![
        section(
            "OneDrive info",
            card(
                column![
                    text(format!("oxdrive version: {}", env!("CARGO_PKG_VERSION")))
                        .size(theme::TEXT_BODY),
                    text(format!("Client version: {}", app.client_version)).size(theme::TEXT_BODY),
                ]
                .spacing(theme::GAP)
            ),
        ),
        section(
            "Related links",
            column![
                link("Source code", "https://github.com/nikolainyegaard/oxdrive"),
                link(
                    "Report a problem",
                    "https://github.com/nikolainyegaard/oxdrive/issues"
                ),
                link(
                    "The onedrive client",
                    "https://github.com/abraunegg/onedrive"
                ),
            ]
            .spacing(theme::GAP),
        ),
        text("MIT licence. Not affiliated with Microsoft.")
            .size(theme::TEXT_SMALL)
            .color(theme::TEXT_SECONDARY),
    ]
    .spacing(theme::PADDING * 1.5)
    .into()
}

/// The quota line once `--display-quota` answered, free space from the monitor until then.
fn storage_text(app: &App) -> String {
    match (app.quota, app.free_space) {
        (Some((used, total)), _) if total > 0 => storage_line(used, total),
        (_, Some(free)) => format!("{} free", human_size(free)),
        _ => "Storage unknown until the client reports it".to_owned(),
    }
}

fn section<'a>(title: &'a str, body: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    column![
        text(title).size(theme::TEXT_BODY).font(theme::SEMIBOLD),
        body.into()
    ]
    .spacing(theme::GAP)
    .into()
}

fn toggle_row<'a>(label: &'a str, on: bool, message: fn(bool) -> Message) -> Element<'a, Message> {
    row![
        text(label).size(theme::TEXT_BODY),
        space::horizontal(),
        text(if on { "On" } else { "Off" }).size(theme::TEXT_BODY),
        toggler(on).on_toggle(message),
    ]
    .spacing(theme::GAP)
    .align_y(Alignment::Center)
    .into()
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> container::Container<'a, Message> {
    container(content)
        .padding(theme::PADDING)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(theme::CARD.into()),
            border: Border {
                radius: theme::CARD_RADIUS.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
}
