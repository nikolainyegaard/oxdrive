//! In-app glyphs: our own SVGs under `assets/icons/`, compiled in, recoloured per use.
//! A file of the same name under `~/.config/oxdrive/icons/` replaces one (docs/config.md).

use iced::widget::svg::{self, Handle, Svg};
use iced::{Color, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Gear,
    Check,
    Sync,
    Pause,
    Play,
    Warning,
    Error,
    Key,
    File,
    Folder,
    Globe,
    Person,
    Bell,
    Info,
    Image,
    ArrowDown,
    Plus,
}

impl Icon {
    /// The file stem under `assets/icons/`, and the override name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Gear => "gear",
            Self::Check => "check",
            Self::Sync => "sync",
            Self::Pause => "pause",
            Self::Play => "play",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Key => "key",
            Self::File => "file",
            Self::Folder => "folder",
            Self::Globe => "globe",
            Self::Person => "person",
            Self::Bell => "bell",
            Self::Info => "info",
            Self::Image => "image",
            Self::ArrowDown => "arrow-down",
            Self::Plus => "plus",
        }
    }

    fn bytes(self) -> &'static [u8] {
        macro_rules! asset {
            ($name:literal) => {
                include_bytes!(concat!("../../../assets/icons/", $name, ".svg"))
            };
        }
        match self {
            Self::Gear => asset!("gear"),
            Self::Check => asset!("check"),
            Self::Sync => asset!("sync"),
            Self::Pause => asset!("pause"),
            Self::Play => asset!("play"),
            Self::Warning => asset!("warning"),
            Self::Error => asset!("error"),
            Self::Key => asset!("key"),
            Self::File => asset!("file"),
            Self::Folder => asset!("folder"),
            Self::Globe => asset!("globe"),
            Self::Person => asset!("person"),
            Self::Bell => asset!("bell"),
            Self::Info => asset!("info"),
            Self::Image => asset!("image"),
            Self::ArrowDown => asset!("arrow-down"),
            Self::Plus => asset!("plus"),
        }
    }

    fn handle(self) -> Handle {
        // ponytail: one stat per icon per view to honour a dropped-in override; cache the
        // lookup if the flyout ever redraws often enough for it to show.
        match oxdrive_native::icons::custom(self.name()) {
            Some(path) => Handle::from_path(path),
            None => Handle::from_memory(self.bytes()),
        }
    }
}

/// The app icon in its own colours: the same file the tray and the launcher show.
pub fn app<'a, Message: 'a>(size: f32) -> Element<'a, Message> {
    picture(Handle::from_memory(oxdrive_native::icons::app_svg()), size)
}

/// An SVG shown as is, in its own colours.
pub fn picture<'a, Message: 'a>(handle: Handle, size: f32) -> Element<'a, Message> {
    Svg::new(handle).width(size).height(size).into()
}

/// The icon at `size` pixels square, every stroke in `color`.
pub fn icon<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    Svg::new(icon.handle())
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_a_small_svg() {
        for icon in [
            Icon::Gear,
            Icon::Check,
            Icon::Sync,
            Icon::Pause,
            Icon::Play,
            Icon::Warning,
            Icon::Error,
            Icon::Key,
            Icon::File,
            Icon::Folder,
            Icon::Globe,
            Icon::Person,
            Icon::Bell,
            Icon::Info,
            Icon::Image,
            Icon::ArrowDown,
            Icon::Plus,
        ] {
            let text = std::str::from_utf8(icon.bytes()).unwrap_or_default();
            assert!(text.starts_with("<svg"), "{} is not an svg", icon.name());
            assert!(text.len() < 1024, "{} is too big", icon.name());
        }
    }
}
