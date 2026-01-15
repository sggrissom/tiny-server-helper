use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

use crate::checker::Status;

/// Theme preset names
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ThemeName {
    #[default]
    Dark,
    Light,
    HighContrast,
}

impl ThemeName {
    /// Cycle to the next theme
    pub fn next(self) -> Self {
        match self {
            ThemeName::Dark => ThemeName::Light,
            ThemeName::Light => ThemeName::HighContrast,
            ThemeName::HighContrast => ThemeName::Dark,
        }
    }
}

impl<'de> Deserialize<'de> for ThemeName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "dark" => Ok(ThemeName::Dark),
            "light" => Ok(ThemeName::Light),
            "high-contrast" | "high_contrast" | "highcontrast" => Ok(ThemeName::HighContrast),
            _ => Err(serde::de::Error::custom(format!("unknown theme: {}", s))),
        }
    }
}

/// Complete theme with all color definitions
#[derive(Debug, Clone)]
pub struct Theme {
    // Status colors
    pub status_up: Color,
    pub status_down: Color,
    pub status_warning: Color,
    pub status_unknown: Color,

    // UI chrome
    pub header_fg: Color,
    pub footer_fg: Color,
    pub border_fg: Color,
    pub selection_bg: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // Alert severity
    pub alert_critical: Color,
    pub alert_warning: Color,
    pub alert_recovery: Color,

    // Chart/graph
    pub chart_line: Color,
    pub chart_axis: Color,

    // Error bar
    pub error_fg: Color,
    pub error_bg: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            status_up: Color::Green,
            status_down: Color::Red,
            status_warning: Color::Yellow,
            status_unknown: Color::Gray,
            header_fg: Color::Cyan,
            footer_fg: Color::DarkGray,
            border_fg: Color::White,
            selection_bg: Color::DarkGray,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            text_muted: Color::DarkGray,
            alert_critical: Color::Red,
            alert_warning: Color::Yellow,
            alert_recovery: Color::Green,
            chart_line: Color::Cyan,
            chart_axis: Color::Gray,
            error_fg: Color::Black,
            error_bg: Color::Yellow,
        }
    }

    pub fn light() -> Self {
        // "Light" theme optimized for dark terminal backgrounds
        // Uses softer, pastel-like colors instead of the dark theme's saturated colors
        Self {
            status_up: Color::Rgb(100, 220, 100),    // Soft green
            status_down: Color::Rgb(255, 100, 100),  // Soft red
            status_warning: Color::Rgb(255, 200, 100), // Soft orange
            status_unknown: Color::Gray,
            header_fg: Color::Rgb(150, 180, 255),    // Soft blue
            footer_fg: Color::Gray,
            border_fg: Color::Gray,
            selection_bg: Color::Rgb(60, 60, 80),   // Muted purple-gray
            text_primary: Color::Rgb(220, 220, 230), // Off-white
            text_secondary: Color::Rgb(160, 160, 180),
            text_muted: Color::Rgb(100, 100, 120),
            alert_critical: Color::Rgb(255, 100, 100),
            alert_warning: Color::Rgb(255, 200, 100),
            alert_recovery: Color::Rgb(100, 220, 100),
            chart_line: Color::Rgb(150, 180, 255),
            chart_axis: Color::Rgb(120, 120, 140),
            error_fg: Color::Black,
            error_bg: Color::Rgb(255, 200, 100),
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            status_up: Color::Rgb(0, 255, 0),        // Bright green
            status_down: Color::Rgb(255, 0, 0),     // Bright red
            status_warning: Color::Rgb(255, 255, 0), // Bright yellow
            status_unknown: Color::White,
            header_fg: Color::Rgb(0, 255, 255),     // Bright cyan
            footer_fg: Color::White,
            border_fg: Color::White,
            selection_bg: Color::Rgb(80, 80, 80),
            text_primary: Color::White,
            text_secondary: Color::White,
            text_muted: Color::Rgb(180, 180, 180),
            alert_critical: Color::Rgb(255, 0, 0),
            alert_warning: Color::Rgb(255, 255, 0),
            alert_recovery: Color::Rgb(0, 255, 0),
            chart_line: Color::Rgb(0, 255, 255),
            chart_axis: Color::White,
            error_fg: Color::Black,
            error_bg: Color::Rgb(255, 255, 0),
        }
    }

    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
            ThemeName::HighContrast => Self::high_contrast(),
        }
    }

    /// Get color for a status value
    pub fn status_color(&self, status: &Status) -> Color {
        match status {
            Status::Up => self.status_up,
            Status::Down => self.status_down,
            Status::Warning => self.status_warning,
        }
    }

    /// Get style for selected items
    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for headers
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for error bar
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.error_fg)
            .bg(self.error_bg)
            .add_modifier(Modifier::BOLD)
    }
}

/// Responsive layout helper based on terminal width
pub struct ResponsiveLayout {
    pub width: u16,
}

impl ResponsiveLayout {
    pub fn new(width: u16) -> Self {
        Self { width }
    }

    /// Get appropriate site name width based on terminal width
    pub fn site_name_width(&self) -> usize {
        if self.width >= 120 {
            40 // Full width
        } else if self.width >= 100 {
            30
        } else if self.width >= 80 {
            25
        } else {
            15 // Narrow terminals
        }
    }

    /// Get appropriate URL truncation length
    pub fn url_max_len(&self) -> usize {
        if self.width >= 120 {
            80
        } else if self.width >= 100 {
            60
        } else if self.width >= 80 {
            40
        } else {
            25
        }
    }

    /// Whether to show sparkline charts
    pub fn show_sparkline(&self) -> bool {
        self.width >= 60
    }

    /// Whether to show detailed metrics
    pub fn show_detailed_metrics(&self) -> bool {
        self.width >= 80
    }

    /// Number of lines per list item (for mouse click calculations)
    pub fn lines_per_site_item(&self) -> u16 {
        if self.show_sparkline() {
            4
        } else if self.show_detailed_metrics() {
            3
        } else {
            2
        }
    }

    /// Truncate string with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.chars().count() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            s.chars().take(max_len).collect()
        } else {
            format!("{}...", s.chars().take(max_len - 3).collect::<String>())
        }
    }
}
