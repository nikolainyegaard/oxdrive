//! Design tokens, taken from the Windows 11 OneDrive flyout in dark mode. Views name a token,
//! never a colour or a size.

use iced::theme::Palette;
use iced::{Color, Theme};

pub const FLYOUT_SIZE: (u32, u32) = (360, 520);
pub const FLYOUT_MARGIN: i32 = 8;
pub const PADDING: f32 = 16.0;
pub const GAP: f32 = 8.0;
pub const TEXT_TITLE: f32 = 20.0;
pub const TEXT_BODY: f32 = 14.0;

pub const BACKGROUND: Color = Color::from_rgb(0.125, 0.125, 0.125);
pub const TEXT: Color = Color::WHITE;
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.8, 0.8, 0.8);
pub const ACCENT: Color = Color::from_rgb(0.0, 0.47, 0.83);
pub const SUCCESS: Color = Color::from_rgb(0.42, 0.8, 0.37);
pub const WARNING: Color = Color::from_rgb(0.99, 0.88, 0.0);
pub const DANGER: Color = Color::from_rgb(1.0, 0.6, 0.64);

pub fn theme() -> Theme {
    Theme::custom(
        String::from("OneDrive dark"),
        Palette {
            background: BACKGROUND,
            text: TEXT,
            primary: ACCENT,
            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
        },
    )
}

pub const SETTINGS_SIZE: (u32, u32) = (900, 680);
pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const TEXT_HEADING: f32 = 28.0;
pub const TEXT_SMALL: f32 = 12.0;
pub const CARD_RADIUS: f32 = 4.0;
/// Cards and the selected sidebar item: one step lighter than the background.
pub const CARD: Color = Color::from_rgb(0.17, 0.17, 0.17);
/// Hovered sidebar item: between the background and a card.
pub const HOVER: Color = Color::from_rgb(0.15, 0.15, 0.15);
/// Grey buttons like Look up, Manage storage and Choose folders in the Windows client.
pub const BUTTON: Color = Color::from_rgb(0.23, 0.23, 0.23);
pub const BUTTON_HOVER: Color = Color::from_rgb(0.27, 0.27, 0.27);
pub const BOLD: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..iced::Font::DEFAULT
};
pub const SEMIBOLD: iced::Font = iced::Font {
    weight: iced::font::Weight::Semibold,
    ..iced::Font::DEFAULT
};
