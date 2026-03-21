//! Status bar component

use crate::{body_text, platform_color, platform_icon, AppTheme};
use eframe::egui;
use hw_hal::{HardwareInfo, Platform};

pub struct StatusBar {
    pub message: String,
    pub message_level: MessageLevel,
    pub hardware_info: Option<HardwareInfo>,
    pub build_status: BuildStatus,
    pub serial_status: SerialStatus,
}

#[derive(Debug, Clone)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum BuildStatus {
    Idle,
    Building,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub enum SerialStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: "Ready".to_string(),
            message_level: MessageLevel::Info,
            hardware_info: None,
            build_status: BuildStatus::Idle,
            serial_status: SerialStatus::Disconnected,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        // Status bar background
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            theme.surface,
        );

        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Main message
            let message_color = match self.message_level {
                MessageLevel::Info => theme.text,
                MessageLevel::Success => crate::SUCCESS_COLOR,
                MessageLevel::Warning => crate::WARNING_COLOR,
                MessageLevel::Error => crate::ERROR_COLOR,
            };

            ui.label(
                egui::RichText::new(&self.message)
                    .color(message_color)
                    .size(12.0)
            );

            ui.separator();

            // Hardware status
            if let Some(ref hardware) = self.hardware_info {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} {}", platform_icon(&hardware.platform), hardware.platform))
                            .color(platform_color(&hardware.platform))
                            .size(12.0)
                    );
                    
                    ui.label(
                        egui::RichText::new(&hardware.port)
                            .color(theme.text_secondary)
                            .size(12.0)
                    );
                });
            } else {
                ui.label(
                    egui::RichText::new("🔴 No Hardware")
                        .color(crate::ERROR_COLOR)
                        .size(12.0)
                );
            }

            ui.separator();

            // Build status
            let build_text = match self.build_status {
                BuildStatus::Idle => "⚪ Idle",
                BuildStatus::Building => "🟡 Building...",
                BuildStatus::Success => "🟢 Build Success",
                BuildStatus::Failed => "🔴 Build Failed",
            };

            ui.label(
                egui::RichText::new(build_text)
                    .size(12.0)
            );

            ui.separator();

            // Serial status
            let serial_text = match self.serial_status {
                SerialStatus::Disconnected => "🔴 Serial Off",
                SerialStatus::Connecting => "🟡 Connecting...",
                SerialStatus::Connected => "🟢 Serial On",
                SerialStatus::Error => "🔴 Serial Error",
            };

            ui.label(
                egui::RichText::new(serial_text)
                    .size(12.0)
            );

            // Right side - time and memory
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Current time
                let now = chrono::Local::now();
                ui.label(
                    egui::RichText::new(now.format("%H:%M:%S").to_string())
                        .color(theme.text_secondary)
                        .size(12.0)
                );

                ui.separator();

                // Memory usage
                if let Some(memory) = self.get_memory_usage() {
                    ui.label(
                        egui::RichText::new(format!("Mem: {}MB", memory))
                            .color(theme.text_secondary)
                            .size(12.0)
                    );
                }
            });
        });
    }

    pub fn set_message(&mut self, message: String, level: MessageLevel) {
        self.message = message;
        self.message_level = level;
    }

    pub fn set_hardware_info(&mut self, hardware: Option<HardwareInfo>) {
        self.hardware_info = hardware;
    }

    pub fn set_build_status(&mut self, status: BuildStatus) {
        self.build_status = status;
    }

    pub fn set_serial_status(&mut self, status: SerialStatus) {
        self.serial_status = status;
    }

    fn get_memory_usage(&self) -> Option<u64> {
        // Simple memory usage calculation
        // In a real implementation, you'd use a proper memory monitoring crate
        Some(std::process::id() as u64 / 1024 / 1024) // Convert to MB (simplified)
    }

    pub fn info(&mut self, message: &str) {
        self.set_message(message.to_string(), MessageLevel::Info);
    }

    pub fn success(&mut self, message: &str) {
        self.set_message(message.to_string(), MessageLevel::Success);
    }

    pub fn warning(&mut self, message: &str) {
        self.set_message(message.to_string(), MessageLevel::Warning);
    }

    pub fn error(&mut self, message: &str) {
        self.set_message(message.to_string(), MessageLevel::Error);
    }

    pub fn clear(&mut self) {
        self.set_message("Ready".to_string(), MessageLevel::Info);
    }
}
