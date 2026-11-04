//! The flyout: account header, status line, recent activity, footer actions.

use iced::widget::{button, column, row, space, text};
use iced::{Element, Length};

use oxdrive_core::status::SyncState;

use crate::theme;
use crate::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(&app.title).size(theme::TEXT_TITLE),
        text(&app.account)
            .size(theme::TEXT_BODY)
            .color(theme::TEXT_SECONDARY),
        text(app.state.headline()).size(theme::TEXT_BODY),
    ]
    .extend(app.detail.as_deref().map(|detail| {
        text(detail)
            .size(theme::TEXT_BODY)
            .color(theme::TEXT_SECONDARY)
            .into()
    }))
    .spacing(theme::GAP / 2.0);

    let recent: Element<'_, Message> = if app.recent.is_empty() {
        text("No recent activity")
            .size(theme::TEXT_BODY)
            .color(theme::TEXT_SECONDARY)
            .into()
    } else {
        column(
            app.recent
                .iter()
                .map(|name| text(name).size(theme::TEXT_BODY).into()),
        )
        .spacing(theme::GAP / 2.0)
        .into()
    };

    let (action_label, action) = match app.state {
        SyncState::Blocked => ("Sign in", Message::SignIn),
        SyncState::Paused => ("Resume", Message::TogglePause),
        _ => ("Pause", Message::TogglePause),
    };
    let footer = row![
        button(text("Open folder").size(theme::TEXT_BODY)),
        button(text("View online").size(theme::TEXT_BODY)),
        space::horizontal(),
        button(text(action_label).size(theme::TEXT_BODY)).on_press(action),
    ]
    .spacing(theme::GAP);

    column![header, recent, space::vertical(), footer]
        .spacing(theme::PADDING)
        .padding(theme::PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
