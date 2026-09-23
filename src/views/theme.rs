use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;
use ratatui::style::Color;

pub type Rgb = (u8, u8, u8);

/// Theme selection supports Auto (smart detection), Dark (Tokyo Night Night),
/// and Light (Tokyo Night Day).
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

impl FromStr for ThemeMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "auto" => Ok(Self::Auto),
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            other => Err(format!("未知主题: {}", other)),
        }
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
    pub badge_fg: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct CliTheme {
    pub foreground: Rgb,
    pub translation: Rgb,
    pub blue: Rgb,
    pub cyan: Rgb,
    pub green: Rgb,
    pub magenta: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub muted: Rgb,
    pub border: Rgb,
}

impl ThemeMode {
    /// Resolve Auto to either Dark or Light based on environment detection.
    pub fn resolved(self) -> Self {
        match self {
            Self::Auto => detect_terminal_theme(),
            other => other,
        }
    }

    pub fn tui(self) -> Theme {
        match self.resolved() {
            Self::Auto | Self::Dark => Theme {
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
                badge_fg: Color::Rgb(26, 27, 38),
            },
            Self::Light => Theme {
                background: Color::Rgb(245, 245, 247),
                foreground: Color::Rgb(55, 60, 84),
                secondary: Color::Rgb(76, 85, 120),
                blue: Color::Rgb(46, 86, 173),
                cyan: Color::Rgb(0, 113, 143),
                green: Color::Rgb(56, 112, 16),
                magenta: Color::Rgb(142, 60, 202),
                purple: Color::Rgb(115, 60, 190),
                orange: Color::Rgb(180, 77, 24),
                yellow: Color::Rgb(143, 94, 21),
                red: Color::Rgb(199, 44, 72),
                comment: Color::Rgb(132, 142, 179),
                selection: Color::Rgb(210, 214, 224),
                border: Color::Rgb(160, 168, 195),
                badge_fg: Color::Rgb(255, 255, 255),
            },
        }
    }

    pub fn cli(self) -> CliTheme {
        let resolved = self.resolved();
        match resolved {
            Self::Auto | Self::Dark => CliTheme {
                foreground: (230, 237, 243),
                translation: (192, 202, 245),
                blue: (122, 162, 247),
                cyan: (125, 207, 255),
                green: (158, 206, 106),
                magenta: (187, 154, 247),
                orange: (255, 158, 100),
                yellow: (224, 175, 104),
                muted: (169, 177, 214),
                border: (86, 95, 137),
            },
            Self::Light => CliTheme {
                foreground: (55, 60, 84),
                translation: (76, 85, 120),
                blue: (46, 86, 173),
                cyan: (0, 113, 143),
                green: (56, 112, 16),
                magenta: (142, 60, 202),
                orange: (180, 77, 24),
                yellow: (143, 94, 21),
                muted: (132, 142, 179),
                border: (160, 168, 195),
            },
        }
    }
}

pub fn detect_terminal_theme() -> ThemeMode {
    // 1. Check COLORFGBG environment variable (supported by xterm, rxvt, mintty, konsole, etc.)
    if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
        if let Some(bg_str) = colorfgbg.rsplit(';').next() {
            if let Ok(bg_num) = bg_str.trim().parse::<u8>() {
                if bg_num == 7 || bg_num == 15 {
                    return ThemeMode::Light;
                } else if bg_num <= 6 || bg_num == 8 {
                    return ThemeMode::Dark;
                }
            }
        }
    }

    // 2. Check other terminal theme hints
    if let Ok(term_theme) = std::env::var("TERM_THEME") {
        let lower = term_theme.to_lowercase();
        if lower.contains("light") {
            return ThemeMode::Light;
        } else if lower.contains("dark") {
            return ThemeMode::Dark;
        }
    }

    if let Ok(bat_theme) = std::env::var("BAT_THEME") {
        let lower = bat_theme.to_lowercase();
        if lower.contains("light") {
            return ThemeMode::Light;
        }
    }

    // 3. On Windows: check system personalization theme
    #[cfg(windows)]
    {
        if let Some(is_light) = detect_windows_light_theme() {
            if is_light {
                return ThemeMode::Light;
            } else {
                return ThemeMode::Dark;
            }
        }
    }

    // 4. Default fallback: Dark (classic Tokyo Night)
    ThemeMode::Dark
}

#[cfg(windows)]
fn detect_windows_light_theme() -> Option<bool> {
    use std::process::Command;
    let output = Command::new("reg")
        .args([
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("AppsUseLightTheme") {
            if line.contains("0x1") {
                return Some(true);
            } else if line.contains("0x0") {
                return Some(false);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_parsing() {
        assert_eq!("dark".parse::<ThemeMode>().unwrap(), ThemeMode::Dark);
        assert_eq!("light".parse::<ThemeMode>().unwrap(), ThemeMode::Light);
        assert_eq!("auto".parse::<ThemeMode>().unwrap(), ThemeMode::Auto);
        assert!("unknown".parse::<ThemeMode>().is_err());
    }

    #[test]
    fn fixed_themes_define_distinct_palettes() {
        let dark = ThemeMode::Dark.tui();
        let light = ThemeMode::Light.tui();
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.foreground, light.foreground);
    }

    #[test]
    fn auto_theme_resolves_to_valid_theme() {
        let resolved = ThemeMode::Auto.resolved();
        assert!(resolved == ThemeMode::Dark || resolved == ThemeMode::Light);
    }
}
