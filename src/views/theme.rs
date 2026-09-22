use std::fmt;

use clap::ValueEnum;
use ratatui::style::Color;

pub type Rgb = (u8, u8, u8);

/// Theme selection is intentionally explicit. Terminals do not expose a
/// portable, reliable light/dark-background query, so `auto` inherits the
/// terminal's own foreground/background instead of guessing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub secondary: Color,
    pub blue: Color,
    pub cyan: Color,
    pub green: Color,
    pub magenta: Color,
    pub purple: Color,
    pub orange: Color,
    pub yellow: Color,
    pub red: Color,
    pub comment: Color,
    pub selection: Color,
    pub border: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct CliTheme {
    pub mode: ThemeMode,
    pub foreground: Rgb,
    pub translation: Rgb,
    pub blue: Rgb,
    pub cyan: Rgb,
    pub green: Rgb,
    pub magenta: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub muted: Rgb,
    pub selection: Rgb,
    pub border: Rgb,
}

impl ThemeMode {
    pub const fn tui(self) -> Theme {
        match self {
            Self::Auto => Theme {
                background: Color::Reset,
                foreground: Color::Reset,
                secondary: Color::LightBlue,
                blue: Color::LightBlue,
                cyan: Color::LightCyan,
                green: Color::LightGreen,
                magenta: Color::LightMagenta,
                purple: Color::LightMagenta,
                orange: Color::LightCyan,
                yellow: Color::LightYellow,
                red: Color::LightRed,
                comment: Color::Reset,
                selection: Color::DarkGray,
                border: Color::LightBlue,
            },
            Self::Dark => Theme {
                background: Color::Rgb(26, 27, 38),
                foreground: Color::Rgb(230, 237, 243),
                secondary: Color::Rgb(192, 202, 245),
                blue: Color::Rgb(122, 162, 247),
                cyan: Color::Rgb(125, 207, 255),
                green: Color::Rgb(158, 206, 106),
                magenta: Color::Rgb(187, 154, 247),
                purple: Color::Rgb(157, 124, 216),
                orange: Color::Rgb(255, 158, 100),
                yellow: Color::Rgb(224, 175, 104),
                red: Color::Rgb(247, 118, 142),
                comment: Color::Rgb(169, 177, 214),
                selection: Color::Rgb(51, 65, 90),
                border: Color::Rgb(86, 95, 137),
            },
            Self::Light => Theme {
                background: Color::Rgb(250, 250, 247),
                foreground: Color::Rgb(31, 41, 55),
                secondary: Color::Rgb(55, 65, 81),
                blue: Color::Rgb(30, 64, 175),
                cyan: Color::Rgb(3, 105, 122),
                green: Color::Rgb(21, 128, 61),
                magenta: Color::Rgb(126, 34, 206),
                purple: Color::Rgb(109, 40, 217),
                orange: Color::Rgb(154, 52, 18),
                yellow: Color::Rgb(133, 77, 14),
                red: Color::Rgb(185, 28, 28),
                comment: Color::Rgb(75, 85, 99),
                selection: Color::Rgb(226, 232, 240),
                border: Color::Rgb(100, 116, 139),
            },
        }
    }

    pub const fn cli(self) -> CliTheme {
        match self {
            Self::Auto => CliTheme {
                mode: self,
                foreground: (230, 237, 243),
                translation: (192, 202, 245),
                blue: (122, 162, 247),
                cyan: (125, 207, 255),
                green: (158, 206, 106),
                magenta: (187, 154, 247),
                orange: (255, 158, 100),
                yellow: (224, 175, 104),
                muted: (169, 177, 214),
                selection: (51, 65, 90),
                border: (86, 95, 137),
            },
            Self::Dark => CliTheme {
                mode: self,
                foreground: (230, 237, 243),
                translation: (192, 202, 245),
                blue: (122, 162, 247),
                cyan: (125, 207, 255),
                green: (158, 206, 106),
                magenta: (187, 154, 247),
                orange: (255, 158, 100),
                yellow: (224, 175, 104),
                muted: (169, 177, 214),
                selection: (51, 65, 90),
                border: (86, 95, 137),
            },
            Self::Light => CliTheme {
                mode: self,
                foreground: (31, 41, 55),
                translation: (55, 65, 81),
                blue: (30, 64, 175),
                cyan: (3, 105, 122),
                green: (21, 128, 61),
                magenta: (126, 34, 206),
                orange: (154, 52, 18),
                yellow: (133, 77, 14),
                muted: (75, 85, 99),
                selection: (226, 232, 240),
                border: (100, 116, 139),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_inherits_terminal_background() {
        assert_eq!(ThemeMode::Auto.tui().background, Color::Reset);
    }

    #[test]
    fn fixed_themes_define_backgrounds() {
        assert_ne!(ThemeMode::Dark.tui().background, Color::Reset);
        assert_ne!(ThemeMode::Light.tui().background, Color::Reset);
    }
}
