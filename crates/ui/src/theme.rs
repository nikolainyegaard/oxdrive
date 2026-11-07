//! Design tokens, taken from the Windows 11 OneDrive flyout in dark mode. Views name a token,
//! never a colour or a size.

use iced::theme::Palette;
use iced::{Color, Theme};

pub const FLYOUT_SIZE: (u32, u32) = (360, 520);
pub const FLYOUT_MARGIN: i32 = 8;
pub const PADDING: f32 = 16.0;
pub const GAP: f32 = 8.0;
pub const TEXT_TITLE: f32 = 16.0;
pub const TEXT_BODY: f32 = 14.0;

pub const BACKGROUND: Color = Color::from_rgb(0.125, 0.125, 0.125);
pub const TEXT: Color = Color::WHITE;
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.667, 0.667, 0.667);
/// Toggles and the sidebar marker: the light accent Windows uses on dark (#4cc2ff).
pub const ACCENT: Color = Color::from_rgb(0.298, 0.761, 1.0);
/// Links (#99ebff).
pub const LINK: Color = Color::from_rgb(0.6, 0.922, 1.0);
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

pub const ICON: f32 = 16.0;
pub const ICON_LARGE: f32 = 20.0;
pub const ICON_FILE: f32 = 32.0;
/// The account photo on the Account page: 95 px in the Windows client.
pub const AVATAR: f32 = 96.0;

pub const SETTINGS_SIZE: (u32, u32) = (900, 680);
pub const SIDEBAR_WIDTH: f32 = 208.0;
/// Sidebar to content, measured from the Windows client (measure.py over its screenshots).
pub const SIDEBAR_GAP: f32 = 44.0;
/// Cards: 66 px tall around a toggle, text 16 px in, toggle 22 px from the right.
pub const CARD_PADDING: iced::Padding = iced::Padding {
    top: 22.0,
    right: 22.0,
    bottom: 22.0,
    left: 16.0,
};
/// Around the whole window: cards 8 px from the left, 48 px of air on the right.
pub const WINDOW_PADDING: iced::Padding = iced::Padding {
    top: 20.0,
    right: 48.0,
    bottom: 20.0,
    left: GAP,
};
pub const TEXT_HEADING: f32 = 28.0;
pub const TEXT_SMALL: f32 = 12.0;
pub const CARD_RADIUS: f32 = 4.0;
/// Cards and the selected sidebar item: one step lighter than the background.
pub const CARD: Color = Color::from_rgb(0.17, 0.17, 0.17);
/// Hovered sidebar item: between the background and a card.
pub const HOVER: Color = Color::from_rgb(0.15, 0.15, 0.15);
/// Grey buttons like Look up, Manage storage and Choose folders in the Windows client.
pub const BUTTON: Color = Color::from_rgb(0.216, 0.216, 0.216);
pub const BUTTON_HOVER: Color = Color::from_rgb(0.27, 0.27, 0.27);
pub const BOLD: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..iced::Font::DEFAULT
};
pub const SEMIBOLD: iced::Font = iced::Font {
    weight: iced::font::Weight::Semibold,
    ..iced::Font::DEFAULT
};
