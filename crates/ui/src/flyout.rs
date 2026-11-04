//! The flyout: account header, status line, recent activity, footer actions.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Alignment, Border, Element, Length};

use oxdrive_core::status::{SyncState, ago};

use crate::theme;
use crate::{Activity, App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        row![
            text(&app.title)
                .size(theme::TEXT_TITLE)
                .font(theme::SEMIBOLD),
            space::horizontal(),
            button(text("Settings").size(theme::TEXT_BODY))
                .style(button::text)
                .on_press(Message::OpenSettings),
        ]
        .align_y(Alignment::Center),
        row![
            // ponytail: a coloured dot stands in for the status glyph until our SVG icons land.
            dot(status_colour(app.state)),
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

    let (action_label, action) = match app.state {
        SyncState::Blocked => ("Sign in", Message::SignIn),
        SyncState::Paused | SyncState::Disabled => ("Resume", Message::TogglePause),
        _ => ("Pause", Message::TogglePause),
    };
    let footer = container(
        row![
            footer_button("Open folder", None),
            footer_button("View online", None),
            footer_button(action_label, Some(action)),
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
    column![
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
        text(ago(item.at.elapsed().as_secs()))
            .size(theme::TEXT_SMALL)
            .color(theme::TEXT_SECONDARY),
    ]
    .spacing(2)
    .padding([0.0, theme::PADDING])
    .into()
}

fn footer_button(label: &str, message: Option<Message>) -> Element<'_, Message> {
    button(text(label).size(theme::TEXT_BODY).center())
        .style(button::text)
        .width(Length::Fill)
        .on_press_maybe(message)
        .into()
}

fn status_colour(state: SyncState) -> iced::Color {
    match state {
        SyncState::Idle => theme::SUCCESS,
        SyncState::Syncing => theme::ACCENT,
        SyncState::Paused | SyncState::Warning => theme::WARNING,
        SyncState::Blocked | SyncState::Disabled | SyncState::Error => theme::DANGER,
    }
}

fn dot<'a>(colour: iced::Color) -> Element<'a, Message> {
    container(space::horizontal())
        .width(10)
        .height(10)
        .style(move |_| container::Style {
            background: Some(colour.into()),
            border: Border {
                radius: 5.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}
