//! ANSI color support for terminal output.

use std::io::IsTerminal;

/// Color mode setting
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    /// Automatically detect terminal support
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

impl ColorMode {
    /// Parse from string (for CLI argument)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }

    /// Check if colors should be used
    pub fn should_use_color(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => std::io::stdout().is_terminal(),
        }
    }
}

/// ANSI color codes
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Reset,
    Bold,
    Dim,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    /// Get the ANSI escape code for this color
    pub fn code(&self) -> &'static str {
        match self {
            Self::Reset => "\x1b[0m",
            Self::Bold => "\x1b[1m",
            Self::Dim => "\x1b[2m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
            Self::BrightBlack => "\x1b[90m",
            Self::BrightRed => "\x1b[91m",
            Self::BrightGreen => "\x1b[92m",
            Self::BrightYellow => "\x1b[93m",
            Self::BrightBlue => "\x1b[94m",
            Self::BrightMagenta => "\x1b[95m",
            Self::BrightCyan => "\x1b[96m",
            Self::BrightWhite => "\x1b[97m",
        }
    }
}

/// Colorizer utility
pub struct Colorizer {
    enabled: bool,
}

impl Colorizer {
    pub fn new(mode: ColorMode) -> Self {
        Self {
            enabled: mode.should_use_color(),
        }
    }

    /// Wrap text with color if enabled
    pub fn paint(&self, color: Color, text: &str) -> String {
        if self.enabled {
            format!("{}{}{}", color.code(), text, Color::Reset.code())
        } else {
            text.to_string()
        }
    }

    /// Check if colors are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_from_str() {
        assert_eq!(ColorMode::parse("auto"), Some(ColorMode::Auto));
        assert_eq!(ColorMode::parse("always"), Some(ColorMode::Always));
        assert_eq!(ColorMode::parse("never"), Some(ColorMode::Never));
        assert_eq!(ColorMode::parse("invalid"), None);
    }

    #[test]
    fn test_colorizer_disabled() {
        let c = Colorizer::new(ColorMode::Never);
        assert_eq!(c.paint(Color::Red, "test"), "test");
    }

    #[test]
    fn test_colorizer_enabled() {
        let c = Colorizer::new(ColorMode::Always);
        let result = c.paint(Color::Red, "test");
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("\x1b[0m"));
    }
}
