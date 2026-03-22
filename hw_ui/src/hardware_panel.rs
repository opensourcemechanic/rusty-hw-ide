//! Hardware connection panel UI component

use crate::{card_frame, header_text, platform_color, platform_icon, success_text, error_text, AppTheme};
use eframe::egui;
use hw_hal::{HardwareInfo, HardwareInterface};
use hw_hal::serial::SerialConnection;

pub struct HardwarePanel {
    pub selected_hardware: Option<HardwareInfo>,
    pub connection: SerialConnection,
    pub show_config: bool,
    pub config_port: String,
    pub config_baud_rate: u32,
    pub auto_detect: bool,
    pub detection_running: bool,
    pub last_detection_time: Option<std::time::Instant>,
}

impl HardwarePanel {
    pub fn new() -> Self {
        Self {
            selected_hardware: None,
            connection: SerialConnection::new(),
            show_config: false,
            config_port: String::new(),
            config_baud_rate: 115200,
            auto_detect: true,
            detection_running: false,
            last_detection_time: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &AppTheme) -> Option<Vec<HardwareInfo>> {
        let mut detect_hardware = None;

        // Header label only
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(header_text("Hardware Connection"));
        });

        ui.add_space(8.0);

        // Controls section (moved down)
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    detect_hardware = Some(());
                }
                
                ui.checkbox(&mut self.auto_detect, "Auto-detect");
            });
        });

        ui.add_space(8.0);

        // Hardware selection content (moved down further)
        if let Some(ref hardware) = self.selected_hardware {
            self.show_connected_hardware(ui, theme);
        } else {
            // Handle detection results from show_hardware_selection
            if let Some(hardware) = self.show_hardware_selection(ui, theme) {
                return Some(hardware);
            }
        }

        // Configuration panel
        if self.show_config {
            self.show_configuration_panel(ui, theme);
        }

        // Handle refresh button detection
        detect_hardware.map(|_| self.detect_hardware())
    }

    fn show_connected_hardware(&self, ui: &mut egui::Ui, theme: &AppTheme) {
        let hardware = self.selected_hardware.as_ref().unwrap();
        
        ui.add_space(4.0);
        
        // Hardware info card
        let frame = card_frame(1.0);
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Status indicator
                let status_icon = if self.connection.is_connected() { "🟢" } else { "🔴" };
                ui.label(crate::hardware_status_icon(self.connection.is_connected()));
                
                // Platform icon and name
                ui.label(egui::RichText::new(format!("{} {}", platform_icon(&hardware.platform), hardware.name))
                    .color(platform_color(&hardware.platform))
                    .strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Disconnect").clicked() {
                        // Disconnect action handled in main app
                    }
                    if ui.button("Configure").clicked() {
                        // Show configuration
                    }
                });
            });
            
            ui.add_space(4.0);
            
            // Hardware details
            ui.horizontal(|ui| {
                ui.label(crate::body_text(&format!("Port: {}", hardware.port)));
                ui.label(crate::body_text(&format!("Baud: {}", hardware.baud_rate)));
            });
            
            if let Some(ref chip_id) = hardware.chip_id {
                ui.label(crate::body_text(&format!("Chip ID: {}", chip_id)));
            }
            
            if let Some(ref description) = hardware.description {
                ui.label(crate::body_text(&format!("Description: {}", description)));
            }
        });
    }

    fn show_hardware_selection(&mut self, ui: &mut egui::Ui, theme: &AppTheme) -> Option<Vec<HardwareInfo>> {
        ui.add_space(4.0);
        
        let mut detection_triggered = false;
        
        let frame = card_frame(1.0);
        frame.show(ui, |ui| {
            ui.label(crate::sub_header_text("No hardware selected"));
            ui.add_space(8.0);
            
            if ui.button("🔍 Detect Hardware").clicked() {
                detection_triggered = true;
            }
            
            ui.add_space(8.0);
            ui.label(crate::body_text("Connect your microcontroller and click detect to find available devices."));
        });
        
        // Return detection results if triggered
        if detection_triggered {
            Some(self.detect_hardware())
        } else {
            None
        }
    }

    fn show_configuration_panel(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        ui.add_space(8.0);
        
        let frame = card_frame(1.0);
        frame.show(ui, |ui| {
            ui.label(header_text("Connection Configuration"));
            ui.add_space(8.0);
            
            // Port configuration
            ui.horizontal(|ui| {
                ui.label(crate::body_text("Port:"));
                ui.text_edit_singleline(&mut self.config_port);
            });
            
            // Baud rate configuration
            ui.horizontal(|ui| {
                ui.label(crate::body_text("Baud Rate:"));
                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", self.config_baud_rate))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config_baud_rate, 9600, "9600");
                        ui.selectable_value(&mut self.config_baud_rate, 19200, "19200");
                        ui.selectable_value(&mut self.config_baud_rate, 38400, "38400");
                        ui.selectable_value(&mut self.config_baud_rate, 57600, "57600");
                        ui.selectable_value(&mut self.config_baud_rate, 115200, "115200");
                        ui.selectable_value(&mut self.config_baud_rate, 230400, "230400");
                        ui.selectable_value(&mut self.config_baud_rate, 460800, "460800");
                        ui.selectable_value(&mut self.config_baud_rate, 921600, "921600");
                    });
            });
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                if ui.button("Test Connection").clicked() {
                    // Test connection
                }
                
                if ui.button("Apply").clicked() {
                    // Apply configuration
                    self.show_config = false;
                }
                
                if ui.button("Cancel").clicked() {
                    self.show_config = false;
                }
            });
        });
    }

    fn detect_hardware(&mut self) -> Vec<HardwareInfo> {
        match hw_hal::detection::detect_hardware() {
            Ok(hardware) => {
                tracing::info!("Detected {} hardware devices", hardware.len());
                hardware
            }
            Err(e) => {
                tracing::error!("Hardware detection failed: {}", e);
                Vec::new()
            }
        }
    }

    pub fn connect_to_hardware(&mut self, hardware: &HardwareInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.connect(hardware)?;
        self.selected_hardware = Some(hardware.clone());
        self.config_port = hardware.port.clone();
        self.config_baud_rate = hardware.baud_rate;
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.disconnect()?;
        self.selected_hardware = None;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    pub fn get_connected_hardware(&self) -> Option<&HardwareInfo> {
        self.selected_hardware.as_ref()
    }
}
