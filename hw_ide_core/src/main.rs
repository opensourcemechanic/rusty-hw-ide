//! Main application entry point for Hardware-Aware IDE

use eframe::egui;
use hw_hal::{HardwareInfo, Platform, HardwareInterface, debug, test};
use hw_hal::serial::SerialConnection;
use hw_ui::{
    AppTheme,
    editor::CodeEditor,
    hardware_panel::HardwarePanel,
    menu_bar::MenuBar,
    serial_monitor::SerialMonitor,
    status_bar::StatusBar,
    card_frame, header_text, platform_color, platform_icon,
};
use tracing::info;

pub struct HardwareIDE {
    // UI Components
    menu_bar: MenuBar,
    hardware_panel: HardwarePanel,
    code_editor: CodeEditor,
    serial_monitor: SerialMonitor,
    status_bar: StatusBar,
    
    // Application State
    theme: AppTheme,
    available_hardware: Vec<HardwareInfo>,
    show_hardware_dialog: bool,
    show_example_dialog: bool,
    
    // Background tasks
    serial_reader_active: bool,
}

impl HardwareIDE {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure tracing
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();

        // Debug serial ports
        debug::debug_serial_ports();
        
        // Test all ports with different baud rates
        test::test_all_ports();

        // Initialize application
        let mut app = Self {
            menu_bar: MenuBar::new(),
            hardware_panel: HardwarePanel::new(),
            code_editor: CodeEditor::new(),
            serial_monitor: SerialMonitor::new(),
            status_bar: StatusBar::new(),
            theme: AppTheme::dark(),
            available_hardware: Vec::new(),
            show_hardware_dialog: false,
            show_example_dialog: false,
            serial_reader_active: false,
        };

        // Load initial example code
        app.load_blink_example();

        // Detect hardware on startup
        if let Ok(hardware) = app.hardware_panel.connection.detect() {
            app.available_hardware = hardware;
            info!("Detected {} hardware devices on startup", app.available_hardware.len());
        }

        app
    }

    fn load_blink_example(&mut self) {
        let blink_code = r#"/*
 * LED Blink Example for WeMos D1 Mini (ESP8266)
 * Blinks the built-in LED (GPIO2) at 1 Hz
 */

#define LED_PIN 2  // Built-in LED on WeMos D1 Mini

void setup() {
  // Initialize serial communication
  Serial.begin(115200);
  Serial.println("LED Blink Example Starting...");
  
  // Set LED pin as output
  pinMode(LED_PIN, OUTPUT);
  
  // Turn LED on initially
  digitalWrite(LED_PIN, HIGH);
  Serial.println("LED turned ON");
}

void loop() {
  // Turn LED off
  digitalWrite(LED_PIN, LOW);
  Serial.println("LED turned OFF");
  delay(500);  // Wait 500ms
  
  // Turn LED on
  digitalWrite(LED_PIN, HIGH);
  Serial.println("LED turned ON");
  delay(500);  // Wait 500ms
}"#;

        self.code_editor = CodeEditor::new_with_code(
            blink_code.to_string(),
            "cpp".to_string(),
        );
        self.code_editor.file_path = Some("examples/blink_wemos_d1.cpp".to_string());
        self.status_bar.info("Loaded LED blink example for WeMos D1 Mini");
    }

    fn show_hardware_selection_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        
        let window = egui::Window::new("Select Hardware")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .open(&mut self.show_hardware_dialog);

        window.show(ctx, |ui| {
            ui.heading("Available Hardware");
            ui.add_space(10.0);

            if self.available_hardware.is_empty() {
                ui.label("No hardware detected. Click 'Refresh' to scan for devices.");
                
                if ui.button("🔄 Refresh").clicked() {
                    if let Ok(hardware) = self.hardware_panel.connection.detect() {
                        self.available_hardware = hardware;
                        self.status_bar.success(&format!("Found {} devices", self.available_hardware.len()));
                    } else {
                        self.status_bar.error("Failed to detect hardware");
                    }
                }
            } else {
                // Hardware list
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, hardware) in self.available_hardware.iter().enumerate() {
                            let frame = card_frame(1.0);
                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Platform icon and name
                                    ui.label(
                                        egui::RichText::new(format!("{} {}", 
                                            platform_icon(&hardware.platform), 
                                            hardware.name))
                                            .size(16.0)
                                            .color(platform_color(&hardware.platform))
                                            .strong()
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Connect").clicked() {
                                            match self.hardware_panel.connect_to_hardware(hardware) {
                                                Ok(_) => {
                                                    self.status_bar.success(&format!("Connected to {}", hardware.name));
                                                    self.status_bar.set_hardware_info(Some(hardware.clone()));
                                                    should_close = true;
                                                }
                                                Err(e) => {
                                                    self.status_bar.error(&format!("Failed to connect: {}", e));
                                                }
                                            }
                                        }
                                    });
                                });

                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(hw_ui::body_text(&format!("Port: {}", hardware.port)));
                                    ui.label(hw_ui::body_text(&format!("Baud: {}", hardware.baud_rate)));
                                });

                                if let Some(ref chip_id) = hardware.chip_id {
                                    ui.label(hw_ui::body_text(&format!("Chip ID: {}", chip_id)));
                                }

                                if let Some(ref description) = hardware.description {
                                    ui.label(hw_ui::body_text(&format!("Description: {}", description)));
                                }
                            });

                            ui.add_space(8.0);
                        }
                    });

                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh").clicked() {
                        if let Ok(hardware) = self.hardware_panel.connection.detect() {
                            self.available_hardware = hardware;
                            self.status_bar.success(&format!("Found {} devices", self.available_hardware.len()));
                        } else {
                            self.status_bar.error("Failed to detect hardware");
                        }
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            }
        });
        
        if should_close {
            self.show_hardware_dialog = false;
        }
    }

    fn show_example_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut load_blink = false;
        
        let window = egui::Window::new("Example Projects")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(300.0)
            .open(&mut self.show_example_dialog);

        window.show(ctx, |ui| {
            ui.heading("Example Projects");
            ui.add_space(10.0);

            let examples = vec![
                ("LED Blink", "Basic LED blinking example", "esp8266"),
                ("Serial Communication", "Serial input/output example", "esp8266"),
                ("WiFi Scanner", "WiFi network scanning", "esp8266"),
                ("Web Server", "Simple HTTP server", "esp8266"),
                ("PWM Control", "PWM output control", "esp8266"),
            ];

            for (name, description, platform) in examples {
                let frame = card_frame(1.0);
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(header_text(name));
                        ui.label(hw_ui::body_text(&format!("({})", platform)));
                    });
                    ui.label(hw_ui::body_text(description));
                    
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            match name {
                                "LED Blink" => load_blink = true,
                                _ => (), // Would need to set a message flag here
                            }
                            should_close = true;
                        }
                    });
                });
                ui.add_space(8.0);
            }

            ui.add_space(10.0);
            if ui.button("Close").clicked() {
                should_close = true;
            }
        });
        
        if should_close {
            self.show_example_dialog = false;
        }
        
        if load_blink {
            self.load_blink_example();
        }
    }

    fn update_serial_monitor(&mut self) {
        if self.serial_monitor.enabled && self.hardware_panel.is_connected() {
            // Read from serial port and add to monitor
            match self.hardware_panel.connection.read_line() {
                Ok(line) if !line.is_empty() => {
                    self.serial_monitor.add_data(&line);
                }
                Ok(_) => {} // Empty line, ignore
                Err(_) => {} // Read error, ignore for now
            }
        }
    }
}

impl eframe::App for HardwareIDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set theme colors
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = self.theme.background;
        visuals.panel_fill = self.theme.surface;
        visuals.widgets.noninteractive.fg_stroke.color = self.theme.text;
        ctx.set_visuals(visuals);

        // Top panel - Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar.show(ui);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Handle menu bar actions and hardware detection
            if self.menu_bar.show_hardware_detection {
                self.show_hardware_dialog = true;
                self.menu_bar.show_hardware_detection = false;
            }

            // Show dialogs
            if self.show_hardware_dialog {
                self.show_hardware_selection_dialog(ctx);
            }

            if self.menu_bar.show_about {
                if self.menu_bar.show_about_dialog(ctx, &self.theme) {
                    self.menu_bar.show_about = false;
                }
            }

            if self.menu_bar.show_preferences {
                if self.menu_bar.show_preferences_dialog(ctx, &self.theme) {
                    self.menu_bar.show_preferences = false;
                }
            }

            // Main layout with tabs
            ui.horizontal(|ui| {
                // Left panel - Hardware
                ui.vertical(|ui| {
                    ui.heading("Hardware");
                    ui.add_space(8.0);
                    
                    if let Some(hardware) = self.hardware_panel.show(ui, &self.theme) {
                        self.available_hardware = hardware;
                        self.status_bar.success(&format!("Found {} devices", self.available_hardware.len()));
                    }
                });

                ui.separator();

                // Center panel - Code Editor
                ui.vertical(|ui| {
                    ui.heading("Code Editor");
                    ui.add_space(8.0);
                    
                    if self.code_editor.show(ui, &self.theme) {
                        self.status_bar.warning("Code modified - remember to save");
                    }
                });

                ui.separator();

                // Right panel - Serial Monitor
                ui.vertical(|ui| {
                    ui.heading("Serial Monitor");
                    ui.add_space(8.0);
                    
                    self.serial_monitor.show(ui, &self.theme, &mut self.hardware_panel.connection);
                });
            });

            // Update serial monitor in background
            self.update_serial_monitor();
        });

        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar.show(ui, &self.theme);
        });

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::R) && i.modifiers.ctrl) {
            self.show_hardware_dialog = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.show_example_dialog = true;
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Configure window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Hardware-Aware IDE"),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Hardware-Aware IDE",
        options,
        Box::new(|cc| Box::new(HardwareIDE::new(cc))),
    )
}
