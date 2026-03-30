//! Menu bar component

use crate::AppTheme;
use eframe::egui;
use hw_hal::{HardwareInfo, Platform};

pub struct MenuBar {
    pub show_about: bool,
    pub show_preferences: bool,
    pub show_new_project_dialog: bool,
    pub show_open_project_dialog: bool,
    pub show_hardware_detection: bool,
    pub compile_clicked: bool,
    pub upload_clicked: bool,
    pub upload_config_clicked: bool,
    pub new_file_clicked: bool,
    pub open_file_clicked: bool,
    pub save_file_clicked: bool,
    pub toggle_serial_monitor: bool,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            show_about: false,
            show_preferences: false,
            show_new_project_dialog: false,
            show_open_project_dialog: false,
            show_hardware_detection: false,
            compile_clicked: false,
            upload_clicked: false,
            upload_config_clicked: false,
            new_file_clicked: false,
            open_file_clicked: false,
            save_file_clicked: false,
            toggle_serial_monitor: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project...").clicked() {
                    self.show_new_project_dialog = true;
                    ui.close_menu();
                }
                
                if ui.button("Open Project...").clicked() {
                    self.show_open_project_dialog = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("New File").clicked() {
                    self.new_file_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Open File...").clicked() {
                    self.open_file_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Save").clicked() {
                    self.save_file_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Save As...").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Compile (Ctrl+B)").clicked() {
                    self.compile_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Upload (Ctrl+U)").clicked() {
                    self.upload_clicked = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Exit").clicked() {
                    ui.close_menu();
                    std::process::exit(0);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Redo").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Cut").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Copy").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Paste").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Find...").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Replace...").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Hardware", |ui| {
                if ui.button("Detect Hardware...").clicked() {
                    self.show_hardware_detection = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Connect...").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Disconnect").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Reset Device").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Bootloader Mode").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Build", |ui| {
                if ui.button("Compile").clicked() {
                    self.compile_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Upload").clicked() {
                    self.upload_clicked = true;
                    ui.close_menu();
                }
                
                if ui.button("Compile & Upload").clicked() {
                    self.compile_clicked = true;
                    self.upload_clicked = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Clean Build").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Upload Configuration...").clicked() {
                    self.upload_config_clicked = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Serial Monitor").clicked() {
                    self.toggle_serial_monitor = true;
                    ui.close_menu();
                }
                
                if ui.button("Serial Plotter").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Board Manager").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Library Manager").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Preferences...").clicked() {
                    self.show_preferences = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Documentation").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Examples").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Troubleshooting").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Check for Updates").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Report Issue").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    pub fn show_about_dialog(&mut self, ctx: &egui::Context, theme: &AppTheme) -> bool {
        let mut open = self.show_about;
        let mut result = false;

        let window = egui::Window::new("About Hardware IDE")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .default_height(300.0)
            .open(&mut open);

        window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Title
                ui.label(
                    egui::RichText::new("Hardware-Aware IDE")
                        .size(24.0)
                        .strong()
                        .color(theme.primary)
                );
                
                ui.add_space(10.0);
                
                // Version
                ui.label(
                    egui::RichText::new("Version 0.1.0")
                        .size(16.0)
                        .color(theme.text_secondary)
                );
                
                ui.add_space(20.0);
                
                // Description
                ui.label(
                    egui::RichText::new("A modern IDE for AVR, ESP8266, and ESP32 microcontrollers")
                        .size(14.0)
                        .color(theme.text)
                );
                
                ui.add_space(10.0);
                
                ui.label(
                    egui::RichText::new("Built with Rust and egui")
                        .size(12.0)
                        .color(theme.text_secondary)
                );
                
                ui.add_space(30.0);
                
                // Features
                ui.horizontal(|ui| {
                    ui.label("🔍 Hardware Detection");
                    ui.label("📡 Serial Communication");
                    ui.label("⚡ Fast Compilation");
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("🎨 Modern UI");
                    ui.label("🔧 Cross-Platform");
                    ui.label("🚀 High Performance");
                });
                
                ui.add_space(30.0);
                
                // Close button
                if ui.button("Close").clicked() {
                    result = true;
                }
            });
        });

        self.show_about = open;
        result
    }

    pub fn show_preferences_dialog(&mut self, ctx: &egui::Context, theme: &AppTheme) -> bool {
        let mut open = self.show_preferences;
        let mut result = false;

        let window = egui::Window::new("Preferences")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .open(&mut open);

        window.show(ctx, |ui| {
            ui.heading("General Settings");
            ui.add_space(10.0);
            
            ui.checkbox(&mut false, "Auto-detect hardware on startup");
            ui.checkbox(&mut false, "Auto-connect to known devices");
            ui.checkbox(&mut false, "Check for updates on startup");
            
            ui.add_space(20.0);
            ui.heading("Editor Settings");
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Font size:");
                ui.add(egui::Slider::new(&mut 14.0, 8.0..=24.0));
            });
            
            ui.checkbox(&mut false, "Show line numbers");
            ui.checkbox(&mut false, "Word wrap");
            ui.checkbox(&mut false, "Auto-save files");
            
            ui.add_space(20.0);
            ui.heading("Build Settings");
            ui.add_space(10.0);
            
            ui.checkbox(&mut false, "Verbose build output");
            ui.checkbox(&mut false, "Auto-upload after compilation");
            ui.checkbox(&mut false, "Clean build before compile");
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    result = true;
                }
                
                if ui.button("Cancel").clicked() {
                    result = true;
                }
            });
        });

        self.show_preferences = open;
        result
    }
    
    pub fn reset_action_flags(&mut self) {
        self.compile_clicked = false;
        self.upload_clicked = false;
        self.upload_config_clicked = false;
        self.toggle_serial_monitor = false;
        self.new_file_clicked = false;
        self.open_file_clicked = false;
        self.save_file_clicked = false;
    }
}
