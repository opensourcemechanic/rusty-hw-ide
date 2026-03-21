//! UI components for the hardware-aware IDE

pub mod editor;
pub mod serial_monitor;
pub mod hardware_panel;
pub mod menu_bar;
pub mod status_bar;

use eframe::egui;
use egui::{Color32, FontId, RichText, Vec2};
use hw_hal::{HardwareInfo, Platform};

pub const PRIMARY_COLOR: Color32 = Color32::from_rgb(41, 128, 185);
pub const SECONDARY_COLOR: Color32 = Color32::from_rgb(52, 152, 219);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(39, 174, 96);
pub const WARNING_COLOR: Color32 = Color32::from_rgb(243, 156, 18);
pub const ERROR_COLOR: Color32 = Color32::from_rgb(231, 76, 60);

pub struct AppTheme {
    pub background: Color32,
    pub surface: Color32,
    pub primary: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub border: Color32,
}

impl AppTheme {
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(30, 30, 30),
            surface: Color32::from_rgb(45, 45, 45),
            primary: PRIMARY_COLOR,
            text: Color32::from_rgb(240, 240, 240),
            text_secondary: Color32::from_rgb(160, 160, 160),
            border: Color32::from_rgb(70, 70, 70),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(250, 250, 250),
            surface: Color32::from_rgb(255, 255, 255),
            primary: PRIMARY_COLOR,
            text: Color32::from_rgb(30, 30, 30),
            text_secondary: Color32::from_rgb(120, 120, 120),
            border: Color32::from_rgb(200, 200, 200),
        }
    }
}

/// UI utility functions
pub fn platform_color(platform: &Platform) -> Color32 {
    match platform {
        Platform::ESP8266 => Color32::from_rgb(0, 122, 204), // Blue
        Platform::ESP32 => Color32::from_rgb(255, 152, 0),   // Orange
        Platform::AVR => Color32::from_rgb(76, 175, 80),    // Green
        Platform::Unknown => Color32::from_rgb(158, 158, 158), // Gray
    }
}

pub fn platform_icon(platform: &Platform) -> &'static str {
    match platform {
        Platform::ESP8266 => "📡",
        Platform::ESP32 => "🔧",
        Platform::AVR => "⚡",
        Platform::Unknown => "❓",
    }
}

pub fn hardware_status_icon(connected: bool) -> &'static str {
    if connected { "🟢" } else { "🔴" }
}

pub fn button_style(_ui: &mut egui::Ui, _primary: bool) -> egui::Style {
    // Simplified style function - in a real implementation you'd customize the style
    egui::Style::default()
}

pub fn card_frame(stroke_width: f32) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::Margin::same(8.0),
        outer_margin: egui::Margin::same(4.0),
        rounding: egui::Rounding::same(4.0),
        shadow: eframe::epaint::Shadow {
            offset: Vec2::new(0.0, 2.0),
            blur: 4.0,
            spread: 0.0,
            color: Color32::from_black_alpha(64),
        },
        fill: Color32::from_rgb(45, 45, 45),
        stroke: egui::Stroke::new(stroke_width, Color32::from_rgb(70, 70, 70)),
    }
}

pub fn header_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(16.0))
        .color(Color32::from_rgb(240, 240, 240))
        .strong()
}

pub fn sub_header_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(14.0))
        .color(Color32::from_rgb(200, 200, 200))
}

pub fn body_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(12.0))
        .color(Color32::from_rgb(180, 180, 180))
}

pub fn success_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(12.0))
        .color(SUCCESS_COLOR)
}

pub fn error_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(12.0))
        .color(ERROR_COLOR)
}

pub fn warning_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(12.0))
        .color(WARNING_COLOR)
}
