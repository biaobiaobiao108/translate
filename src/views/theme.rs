use ratatui::style::Color;

// Tokyo Night Palette.
//
// The original values were tuned for a dark, true-color terminal, but the
// lowest-contrast greys became difficult to read when the terminal did not
// apply the expected background.  Keep the same visual direction while
// making every color used for content readable on both dark and neutral
// terminal themes.
#[allow(dead_code)]
pub const BG: Color = Color::Rgb(26, 27, 38); // #1a1b26
pub const FG: Color = Color::Rgb(230, 237, 243); // #e6edf3 (Primary text)
pub const FG_SUB: Color = Color::Rgb(192, 202, 245); // #c0caf5 (Secondary text / translations)
pub const BLUE: Color = Color::Rgb(122, 162, 247); // #7aa2f7
pub const CYAN: Color = Color::Rgb(125, 207, 255); // #7dcfff
pub const GREEN: Color = Color::Rgb(158, 206, 106); // #9ece6a
pub const MAGENTA: Color = Color::Rgb(187, 154, 247); // #bb9af7
pub const PURPLE: Color = Color::Rgb(157, 124, 216); // #9d7cd8
pub const ORANGE: Color = Color::Rgb(255, 158, 100); // #ff9e64
pub const YELLOW: Color = Color::Rgb(224, 175, 104); // #e0af68
pub const RED: Color = Color::Rgb(247, 118, 142); // #f7768e
pub const COMMENT: Color = Color::Rgb(169, 177, 214); // #a9b1d6 (Readable muted text)
#[allow(dead_code)]
pub const MUTED_TEXT: Color = Color::Rgb(139, 148, 173); // #8b94ad (Low-priority text)
pub const SELECTION: Color = Color::Rgb(51, 65, 90); // #33415a (Badge / selected row)
pub const DARK_BORDER: Color = Color::Rgb(86, 95, 137); // #565f89 (Borders / dividers)
#[allow(dead_code)]
pub const BORDER: Color = Color::Rgb(86, 95, 137); // #565f89
