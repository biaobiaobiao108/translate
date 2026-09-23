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
                background: Color::Reset,
                foreground: Color::Rgb(230, 237, 243),
                secondary: Color::Rgb(192, 202, 245),
                blue: Color::Rgb(122, 162, 247),
                cyan: Color::Rgb(125, 207, 255),
                green: Color::Rgb(115, 218, 202),
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
                background: Color::Reset,
                foreground: Color::Rgb(40, 44, 60),
                secondary: Color::Rgb(76, 85, 120),
                blue: Color::Rgb(36, 76, 160),
                cyan: Color::Rgb(0, 113, 143),
                green: Color::Rgb(22, 110, 60),
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
                translation: (115, 218, 202), // Tokyo Night mint green #73daca: crisp and highly legible on dark
                blue: (122, 162, 247),
                cyan: (125, 207, 255),
                green: (115, 218, 202),
                magenta: (187, 154, 247),
                orange: (255, 158, 100),
                yellow: (224, 175, 104),
                muted: (169, 177, 214),
                border: (86, 95, 137),
            },
            Self::Light => CliTheme {
                foreground: (40, 44, 60),
                translation: (22, 110, 60), // High contrast deep emerald green for light backgrounds
                blue: (36, 76, 160),
                cyan: (0, 113, 143),
                green: (22, 110, 60),
                magenta: (142, 60, 202),
                orange: (180, 77, 24),
                yellow: (143, 94, 21),
                muted: (100, 110, 140),
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

    // 3. On Windows: check Windows Terminal settings if inside WT_SESSION
    #[cfg(windows)]
    {
        if let Some(mode) = detect_windows_terminal_theme() {
            return mode;
        }
    }

    // 4. Default fallback: Dark (classic Tokyo Night for terminal environments)
    ThemeMode::Dark
}

#[cfg(windows)]
fn detect_windows_terminal_theme() -> Option<ThemeMode> {
    if std::env::var("WT_SESSION").is_err() {
        return None;
    }

    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let candidate_paths = [
        format!(
            "{}\\Packages\\Microsoft.WindowsTerminal_8wekyb3d8bbwe\\LocalState\\settings.json",
            local_app_data
        ),
        format!(
            "{}\\Microsoft\\Windows Terminal\\settings.json",
            local_app_data
        ),
    ];

    for path in candidate_paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // If user configured root theme as "light" or "dark"
                if let Some(theme_str) = json.get("theme").and_then(|v| v.as_str()) {
                    if theme_str.eq_ignore_ascii_case("light") {
                        return Some(ThemeMode::Light);
                    } else if theme_str.eq_ignore_ascii_case("dark") {
                        return Some(ThemeMode::Dark);
                    }
                }

                // Check default profile's colorScheme
                let default_scheme_name = json
                    .pointer("/profiles/defaults/colorScheme")
                    .and_then(|v| v.as_str());

                if let Some(scheme_name) = default_scheme_name {
                    if let Some(schemes) = json.get("schemes").and_then(|v| v.as_array()) {
                        for s in schemes {
                            if s.get("name").and_then(|v| v.as_str()) == Some(scheme_name) {
                                if let Some(bg) = s.get("background").and_then(|v| v.as_str()) {
                                    if is_hex_color_light(bg) {
                                        return Some(ThemeMode::Light);
                                    } else {
                                        return Some(ThemeMode::Dark);
                                    }
                                }
                            }
                        }
                    }
                }

                // Windows Terminal defaults to Campbell (dark background #0C0C0C)
                return Some(ThemeMode::Dark);
            }
        }
    }

    Some(ThemeMode::Dark)
}

#[cfg(windows)]
fn is_hex_color_light(hex: &str) -> bool {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            let lum = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
            return lum > 140.0;
        }
    }
    false
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
        assert_eq!(dark.background, Color::Reset);
        assert_eq!(light.background, Color::Reset);
        assert_ne!(dark.foreground, light.foreground);
    }

    #[test]
    fn auto_theme_resolves_to_valid_theme() {
        let resolved = ThemeMode::Auto.resolved();
        assert!(resolved == ThemeMode::Dark || resolved == ThemeMode::Light);
    }
}
