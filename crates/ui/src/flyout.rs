//! The flyout: account header, status line, recent activity, footer actions.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Alignment, Element, Length};

use oxdrive_core::status::{Activity, SyncState, ago};

use crate::icons::{self, Icon, icon};
use crate::theme;
use crate::{App, Message, now};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        row![
            icons::app(theme::ICON_LARGE),
            text(&app.title)
                .size(theme::TEXT_TITLE)
                .font(theme::SEMIBOLD),
            space::horizontal(),
            button(icon(Icon::Gear, theme::ICON_LARGE, theme::TEXT))
                .style(button::text)
                .on_press(Message::OpenSettings),
        ]
        .spacing(theme::GAP)
        .align_y(Alignment::Center),
        row![
            status_icon(app.state),
            text(app.state.headline()).size(theme::TEXT_BODY),
        ]
        .spacing(theme::GAP)
        .align_y(Alignment::Center),
    ]
    .extend(app.detail.as_deref().map(|detail| {
        text(detail)
            .size(theme::TEXT_BODY)
            .color(theme::TEXT_SECONDARY)
            .into()
    }))
    .spacing(theme::GAP)
    .padding(theme::PADDING);

    let recent: Element<'_, Message> = if app.recent.is_empty() {
        container(
            text("No recent activity")
                .size(theme::TEXT_BODY)
                .color(theme::TEXT_SECONDARY),
        )
        .padding([0.0, theme::PADDING])
        .into()
    } else {
        scrollable(column(app.recent.iter().map(activity)).spacing(theme::GAP)).into()
    };

    let (action_icon, action_label, action) = match app.state {
        SyncState::Blocked => (Icon::Key, "Sign in", Message::SignIn),
        SyncState::Paused | SyncState::Disabled => (Icon::Play, "Resume", Message::TogglePause),
        _ => (Icon::Pause, "Pause", Message::TogglePause),
    };
    let footer = container(
        row![
            footer_button(Icon::Folder, "Open folder", None),
            footer_button(Icon::Globe, "View online", None),
            footer_button(action_icon, action_label, Some(action)),
        ]
        .spacing(theme::GAP),
    )
    .padding(theme::PADDING)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(theme::CARD.into()),
        ..container::Style::default()
    });

    column![header, recent, space::vertical(), footer]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn activity(item: &Activity) -> Element<'_, Message> {
    let lines = column![
        text(&item.name).size(theme::TEXT_BODY),
        row![
            text(item.action)
                .size(theme::TEXT_BODY)
                .color(theme::TEXT_SECONDARY),
            text(&item.folder)
                .size(theme::TEXT_BODY)
                .color(theme::ACCENT),
        ]
        .spacing(4),
        text(ago(now().saturating_sub(item.at)))
            .size(theme::TEXT_SMALL)
            .color(theme::TEXT_SECONDARY),
    ]
    .spacing(2);
    row![
        icon(Icon::File, theme::ICON_FILE, theme::TEXT_SECONDARY),
        lines
    ]
    .spacing(theme::GAP)
    .padding([0.0, theme::PADDING])
    .align_y(Alignment::Center)
    .into()
}

fn footer_button(glyph: Icon, label: &str, message: Option<Message>) -> Element<'_, Message> {
    button(
        column![
            icon(glyph, theme::ICON_LARGE, theme::TEXT),
            text(label).size(theme::TEXT_BODY)
        ]
        .spacing(theme::GAP)
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .style(button::text)
    .width(Length::Fill)
    .on_press_maybe(message)
    .into()
}

/// The glyph and colour of the status line, after the Windows flyout.
fn status_icon<'a>(state: SyncState) -> Element<'a, Message> {
    let (glyph, colour) = match state {
        SyncState::Idle => (Icon::Check, theme::SUCCESS),
        SyncState::Syncing => (Icon::Sync, theme::ACCENT),
        SyncState::Paused => (Icon::Pause, theme::WARNING),
        SyncState::Warning => (Icon::Warning, theme::WARNING),
        SyncState::Error | SyncState::Disabled => (Icon::Error, theme::DANGER),
        SyncState::Blocked => (Icon::Key, theme::DANGER),
    };
    icon(glyph, theme::ICON, colour)
}
