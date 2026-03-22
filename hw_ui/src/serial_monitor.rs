//! Serial monitor UI component

use crate::{card_frame, header_text, success_text, error_text, AppTheme};
use eframe::egui;
use hw_hal::{HardwareInterface};
use hw_hal::serial::SerialConnection;
use std::sync::{Arc, Mutex};

pub struct SerialMonitor {
    pub enabled: bool,
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub baud_rate: u32,
    pub buffer: Arc<Mutex<Vec<String>>>,
    pub input_buffer: String,
    pub max_lines: usize,
    pub filter_text: String,
    pub hex_mode: bool,
}

impl SerialMonitor {
    pub fn new() -> Self {
        Self {
            enabled: false,
            auto_scroll: true,
            show_timestamps: true,
            baud_rate: 115200,
            buffer: Arc::new(Mutex::new(Vec::new())),
            input_buffer: String::new(),
            max_lines: 1000,
            filter_text: String::new(),
            hex_mode: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &AppTheme, connection: &mut SerialConnection) {
        // Header label only
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(header_text("Serial Monitor"));
        });

        ui.add_space(8.0);

        // Controls section (moved down)
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Baud rate selector
                ui.label("Baud:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", self.baud_rate))
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.baud_rate, 9600, "9600");
                        ui.selectable_value(&mut self.baud_rate, 19200, "19200");
                        ui.selectable_value(&mut self.baud_rate, 38400, "38400");
                        ui.selectable_value(&mut self.baud_rate, 57600, "57600");
                        ui.selectable_value(&mut self.baud_rate, 115200, "115200");
                        ui.selectable_value(&mut self.baud_rate, 230400, "230400");
                        ui.selectable_value(&mut self.baud_rate, 460800, "460800");
                        ui.selectable_value(&mut self.baud_rate, 921600, "921600");
                    });
                
                ui.separator();
                
                // Options checkboxes
                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                ui.checkbox(&mut self.show_timestamps, "Timestamps");
                ui.checkbox(&mut self.hex_mode, "Hex");
                
                ui.separator();
                
                // Clear button
                if ui.button("Clear").clicked() {
                    self.clear_buffer();
                }
                
                // Connect/Disconnect button
                let button_text = if self.enabled && connection.is_connected() {
                    "⏹ Disconnect"
                } else {
                    "▶ Connect"
                };
                
                if ui.button(button_text).clicked() {
                    if connection.is_connected() {
                        connection.disconnect().ok();
                        self.enabled = false;
                    } else {
                        self.enabled = true;
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Serial output area (moved down further)
        let frame = card_frame(1.0);
        frame.show(ui, |ui| {
            // Output area
            let buffer_data = self.buffer.lock().unwrap();
            let output_text = if self.hex_mode {
                let text = buffer_data.join("\n");
                self.format_as_hex(&text)
            } else {
                buffer_data.join("\n")
            };
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    ui.label(output_text);
                });
        });

        ui.add_space(8.0);

        // Send input area
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label("Send:");
            let response = ui.text_edit_singleline(&mut self.input_buffer);
            
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let text = self.input_buffer.trim();
                if !text.is_empty() {
                    if let Err(e) = connection.write_data(text.as_bytes()) {
                        eprintln!("Failed to send data: {}", e);
                    }
                    self.input_buffer.clear();
                }
            }
            
            if ui.button("Send").clicked() {
                let text = self.input_buffer.trim();
                if !text.is_empty() {
                    if let Err(e) = connection.write_data(text.as_bytes()) {
                        eprintln!("Failed to send data: {}", e);
                    }
                    self.input_buffer.clear();
                }
            }
        });
    }

    pub fn add_data(&self, data: &str) {
        let mut buffer = self.buffer.lock().unwrap();
        
        // Split data into lines
        for line in data.lines() {
            if !line.trim().is_empty() {
                buffer.push(line.to_string());
            }
        }
        
        // Limit buffer size
        if buffer.len() > self.max_lines * 2 {
            let drain_count = buffer.len() - self.max_lines;
            buffer.drain(0..drain_count);
        }
    }

    pub fn add_raw_data(&self, data: &[u8]) {
        let data_str = String::from_utf8_lossy(data);
        self.add_data(&data_str);
    }

    fn send_command(&self, connection: &mut SerialConnection, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !connection.is_connected() {
            return Err("Not connected to hardware".into());
        }
        
        let full_command = format!("{}\n", command);
        connection.send_command(&full_command)?;
        
        // Echo the sent command
        self.add_data(&format!("> {}", command));
        
        Ok(())
    }

    fn clear_buffer(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
    }

    fn get_timestamp(&self) -> String {
        use chrono::Local;
        Local::now().format("%H:%M:%S.%3f").to_string()
    }

    fn format_as_hex(&self, data: &str) -> String {
        let bytes = data.as_bytes();
        let mut hex_string = String::new();
        
        for (i, &byte) in bytes.iter().enumerate() {
            if i > 0 && i % 16 == 0 {
                hex_string.push('\n');
            } else if i > 0 {
                hex_string.push(' ');
            }
            
            hex_string.push_str(&format!("{:02X}", byte));
        }
        
        hex_string
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
