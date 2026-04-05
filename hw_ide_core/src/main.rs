//! Main application entry point for Hardware-Aware IDE

mod compiler;

use eframe::egui;
use hw_hal::{HardwareInfo, Platform, HardwareInterface, debug, test};
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
    
    // Compilation state
    build_dir: std::path::PathBuf,
    last_build_output: String,
    show_build_output_dialog: bool,
    upload_config: compiler::UploadConfig,
    show_upload_config_dialog: bool,
    
    // Async operation state
    compile_in_progress: bool,
    upload_in_progress: bool,
    operation_output: String,
    operation_receiver: Option<std::sync::mpsc::Receiver<String>>,
    
    // Detachable window state
    build_output_detached: bool,
    serial_monitor_detached: bool,
    serial_monitor_visible: bool,
}

impl HardwareIDE {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("=== HardwareIDE::new() called ===");
        
        // Configure tracing
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
        
        // Apply theme once at startup to prevent flickering
        let ctx = cc.egui_ctx.clone();
        let theme = AppTheme::dark();
        ctx.set_visuals({
            let mut visuals = egui::Visuals::dark();
            visuals.window_fill = theme.background;
            visuals.panel_fill = theme.surface;
            visuals.widgets.noninteractive.fg_stroke.color = theme.text;
            visuals
        });

        // Debug serial ports
        debug::debug_serial_ports();
        
        // Test all ports with different baud rates
        test::test_all_ports();

        // Create build directory
        let build_dir = std::env::temp_dir().join("rusty_hw_build");
        if !build_dir.exists() {
            let _ = std::fs::create_dir_all(&build_dir);
        }
        
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
            build_dir,
            last_build_output: String::new(),
            show_build_output_dialog: false,
            upload_config: compiler::UploadConfig::detect_from_port("COM3"), // Default
            show_upload_config_dialog: false,
            
            // Async operation state
            compile_in_progress: false,
            upload_in_progress: false,
            operation_output: String::new(),
            operation_receiver: None,
            
            // Detachable window state
            build_output_detached: false,
            serial_monitor_detached: false,
            serial_monitor_visible: false, // Start hidden by default
        };

        // Load initial example code
        app.load_blink_example();

        // Debug serial port detection
        hw_hal::debug::debug_serial_ports();
        
        // Detect hardware on startup
        tracing::info!("=== About to call detect_hardware() ===");
        if let Ok(hardware) = hw_hal::detection::detect_hardware() {
            app.available_hardware = hardware;
            info!("Detected {} hardware devices on startup", app.available_hardware.len());
        } else {
            tracing::error!("detect_hardware() returned error");
        }

        app
    }

    fn load_blink_example(&mut self) {
        // Load LED blink example based on current board configuration
        let blink_code = match (&self.upload_config.target_chip, &self.upload_config.board_type) {
            (compiler::TargetChip::ATtiny85, _) => {
                // ATtiny85 version - no Serial, use pin 0 (PB0/physical pin 5)
                r#"// LED Blink Example for ATtiny85
// LED connected to Pin 0 (PB0, physical pin 5)

#define LED_PIN 0  // Pin 0 = PB0 = physical pin 5

void setup() {
  pinMode(LED_PIN, OUTPUT);
}

void loop() {
  // Turn LED off
  digitalWrite(LED_PIN, LOW);
  delay(500);  // Wait 500ms
  
  // Turn LED on
  digitalWrite(LED_PIN, HIGH);
  delay(500);  // Wait 500ms
}"#
            }
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoNanoISP) => {
                // Arduino Nano as ISP programmer - don't use this for direct upload
                r#"// Arduino Nano ISP Programmer
// This board is configured as ISP programmer
// Select "Arduino Uno" board type for direct Nano upload

void setup() {
  // Arduino Nano is configured as ISP programmer
}

void loop() {
  // ISP programmer mode - no user code
}"#
            }
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoUno) => {
                // Arduino Nano/Uno direct upload version - no Serial
                r#"// LED Blink Example for Arduino Nano/Uno
// Built-in LED on pin 13 (L LED on Arduino Nano)
// No Serial dependencies - just pure LED blinking

#define LED_PIN 13  // Built-in LED (L) on Arduino Nano

void setup() {
  pinMode(LED_PIN, OUTPUT);
  // Built-in LED should blink after upload
}

void loop() {
  digitalWrite(LED_PIN, HIGH);  // Turn LED ON
  delay(500);                  // Wait 500ms
  
  digitalWrite(LED_PIN, LOW);   // Turn LED OFF
  delay(500);                  // Wait 500ms
}"#
            }
        };

        let filename = match (&self.upload_config.target_chip, &self.upload_config.board_type) {
            (compiler::TargetChip::ATtiny85, _) => "examples/blink_attiny85.cpp",
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoNanoISP) => "examples/arduino_nano_isp.cpp",
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoUno) => "examples/arduino_nano_blink.cpp",
        };

        self.code_editor = CodeEditor::new_with_code(
            blink_code.to_string(),
            "cpp".to_string(),
        );
        self.code_editor.file_path = Some(filename.to_string());
        
        let target_name = match (&self.upload_config.target_chip, &self.upload_config.board_type) {
            (compiler::TargetChip::ATtiny85, _) => "ATtiny85",
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoNanoISP) => "Arduino Nano (ISP)",
            (compiler::TargetChip::ATmega328P, compiler::BoardType::ArduinoUno) => "Arduino Nano (Direct)",
        };
        self.status_bar.info(&format!("Loaded LED blink example for {}", target_name));
    }

    fn show_hardware_selection_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        
        let window = egui::Window::new("Select Hardware")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)  // Even smaller
            .default_height(500.0)  // Increased height for better layout
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
                                // Platform icon and name
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{} {}", 
                                            platform_icon(&hardware.platform), 
                                            hardware.name))
                                            .size(14.0)
                                            .color(platform_color(&hardware.platform))
                                            .strong()
                                    );
                                });

                                ui.add_space(4.0);
                                
                                // Hardware info with better word wrapping
                                ui.label(hw_ui::body_text(&format!("Port: {}", hardware.port)));
                                ui.label(hw_ui::body_text(&format!("Baud: {}", hardware.baud_rate)));

                                if let Some(ref chip_id) = hardware.chip_id {
                                    ui.label(hw_ui::body_text(&format!("Chip: {}", chip_id)));
                                }

                                if let Some(ref description) = hardware.description {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(hw_ui::body_text(description));
                                    });
                                }

                                ui.add_space(6.0);
                                
                                // Connect button on its own line
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

                            ui.add_space(4.0);
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
                ("LED Blink", "Basic LED blinking example", "avr"),
                ("Arduino Nano Blink", "Arduino Nano built-in LED test", "avr"),
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
                                "Arduino Nano Blink" => load_blink = true,
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

    fn compile_code(&mut self) {
        // Don't start if already compiling
        if self.compile_in_progress {
            return;
        }
        
        // Start async compilation
        self.compile_in_progress = true;
        self.operation_output = "Starting compilation...\n".to_string();
        self.show_build_output_dialog = true;
        self.status_bar.info("Compiling...");
        
        // Create a temporary file with the current code
        let sketch_file = self.build_dir.join("sketch.ino");
        
        // Wrap the code with Arduino.h include
        let wrapped_code = format!("#include <Arduino.h>\n\n{}", self.code_editor.code);
        
        if let Err(e) = std::fs::write(&sketch_file, wrapped_code) {
            self.compile_in_progress = false;
            self.operation_output = format!("Failed to write sketch file: {}\n", e);
            self.status_bar.error(&format!("Failed to write sketch file: {}", e));
            return;
        }
        
        // Start compilation in background
        let build_dir = self.build_dir.clone();
        let output_sender = std::sync::mpsc::channel::<String>();
        let receiver = output_sender.1;
        
        // Read the source code for the thread
        let source_code = match std::fs::read_to_string(&sketch_file) {
            Ok(code) => code,
            Err(e) => {
                self.compile_in_progress = false;
                self.operation_output = format!("Failed to read sketch file: {}\n", e);
                self.status_bar.error(&format!("Failed to read sketch file: {}", e));
                return;
            }
        };
        
        // Get current upload config for compilation
        let target_chip = self.upload_config.target_chip.clone();
        let clock_speed = self.upload_config.clock_speed.clone();
        
        std::thread::spawn(move || {
            let result = compiler::compile_avr(&source_code, &build_dir, target_chip, clock_speed);
            let _ = output_sender.0.send(result.output);
            let _ = output_sender.0.send(if result.success { "COMPILATION_SUCCESS".to_string() } else { "COMPILATION_FAILED".to_string() });
        });
        
        // Store receiver for later checking
        self.operation_receiver = Some(receiver);
    }
    
    fn upload_code(&mut self) {
        // Don't start if already uploading
        if self.upload_in_progress || self.compile_in_progress {
            return;
        }
        
        // Check if we have a compiled hex file
        let hex_file = self.build_dir.join("sketch.hex");
        if !hex_file.exists() {
            self.status_bar.error("No compiled code found - compile first!");
            return;
        }
        
        // Get the port from connected hardware
        if self.hardware_panel.is_connected() {
            if let Some(ref hardware) = self.hardware_panel.selected_hardware {
                // Update upload config based on detected port
                self.upload_config = compiler::UploadConfig::detect_from_port(&hardware.port);
            } else {
                self.status_bar.error("No hardware selected");
                return;
            }
        } else {
            self.status_bar.error("No hardware connected - connect first!");
            return;
        };
        
        // Start async upload
        self.upload_in_progress = true;
        self.operation_output = format!("Starting upload to {}...\n", self.upload_config.port);
        self.show_build_output_dialog = true;
        self.status_bar.info(&format!("Uploading to {}...", self.upload_config.port));
        
        // Disconnect serial monitor to release the COM port
        let was_connected = self.hardware_panel.is_connected();
        if was_connected {
            self.operation_output.push_str("Disconnecting serial monitor for upload...\n");
            let _ = self.hardware_panel.connection.disconnect();
        }
        
        // Start upload in background
        let hex_file_clone = hex_file.clone();
        let upload_config = self.upload_config.clone();
        let hardware_info = self.hardware_panel.selected_hardware.clone();
        let output_sender = std::sync::mpsc::channel::<String>();
        let receiver = output_sender.1;
        
        self.operation_output.push_str(&format!("Starting upload thread with config: {:?}\n", upload_config));
        
        std::thread::spawn(move || {
            let result = compiler::upload_avr(&hex_file_clone, &upload_config);
            let _ = output_sender.0.send(result.output);
            let _ = output_sender.0.send(if result.success { "UPLOAD_SUCCESS".to_string() } else { "UPLOAD_FAILED".to_string() });
            
            // Reconnect serial monitor if it was connected before
            if was_connected {
                if let Some(hardware) = hardware_info {
                    // Note: Can't reconnect from background thread, need to handle in main thread
                    let _ = output_sender.0.send("RECONNECT_SERIAL".to_string());
                }
            }
        });
        
        // Store receiver for later checking
        self.operation_receiver = Some(receiver);
    }
    
    fn show_build_output_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_build_output_dialog {
            return;
        }
        
        let mut should_close = false;
        
        // Create window configuration
        let mut window = egui::Window::new("Build Output")
            .collapsible(false)
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0);
        
        if self.build_output_detached {
            window = window.default_pos(egui::pos2(100.0, 100.0));
        }
        
        window.show(ctx, |ui| {
            self.build_output_dialog_contents(ui, &mut should_close);
        });
        
        if should_close {
            self.show_build_output_dialog = false;
        }
    }
    
    fn build_output_dialog_contents(&mut self, ui: &mut egui::Ui, should_close: &mut bool) {
        let title = if self.compile_in_progress {
            "🔄 Compiling..."
        } else if self.upload_in_progress {
            "🔄 Uploading..."
        } else if self.operation_output.contains("✅") {
            "✅ Operation Successful"
        } else {
            "❌ Operation Failed"
        };
        
        ui.heading(title);
        ui.add_space(10.0);
        
        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                ui.monospace(&self.operation_output);
            });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            if ui.button("Copy to Clipboard").clicked() {
                ui.output_mut(|o| o.copied_text = self.operation_output.clone());
                self.status_bar.info("Output copied to clipboard");
            }
            
            if !self.build_output_detached {
                if ui.button("📎 Detach").clicked() {
                    self.build_output_detached = true;
                }
            } else {
                if ui.button("📌 Attach").clicked() {
                    self.build_output_detached = false;
                }
            }
            
            ui.add_space(10.0);
            
            if ui.button("Close").clicked() {
                *should_close = true;
            }
        });
    }

    fn show_upload_config_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_upload_config_dialog {
            return;
        }
        
        let mut should_close = false;
        
        let window = egui::Window::new("Upload Configuration")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .open(&mut self.show_upload_config_dialog);
        
        window.show(ctx, |ui| {
            ui.heading("Upload Configuration");
            ui.add_space(10.0);
            
            // Port
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.label(&self.upload_config.port);
            });
            
            // Board Type
            ui.horizontal(|ui| {
                ui.label("Board Type:");
                let board_names = vec!["Arduino Uno", "Arduino Nano (ISP)"];
                let mut selected_index = match self.upload_config.board_type {
                    compiler::BoardType::ArduinoUno => 0,
                    compiler::BoardType::ArduinoNanoISP => 1,
                };
                
                egui::ComboBox::from_id_source("board_type")
                    .selected_text(board_names[selected_index])
                    .show_ui(ui, |ui| {
                        for (i, board_name) in board_names.iter().enumerate() {
                            ui.selectable_value(&mut selected_index, i, *board_name);
                        }
                    });
                
                self.upload_config.board_type = match selected_index {
                    0 => compiler::BoardType::ArduinoUno,
                    1 => compiler::BoardType::ArduinoNanoISP,
                    _ => compiler::BoardType::ArduinoUno,
                };
                
                // Auto-configure based on board type (but preserve manual overrides)
                match self.upload_config.board_type {
                    compiler::BoardType::ArduinoUno => {
                        self.upload_config.target_chip = compiler::TargetChip::ATmega328P;
                        self.upload_config.clock_speed = compiler::ClockSpeed::MHz16;
                        // Don't overwrite programmer and baud_rate - let user choose
                    }
                    compiler::BoardType::ArduinoNanoISP => {
                        self.upload_config.target_chip = compiler::TargetChip::ATtiny85;
                        self.upload_config.clock_speed = compiler::ClockSpeed::MHz8;
                        self.upload_config.programmer = "stk500v1".to_string();
                        self.upload_config.baud_rate = 19200;
                    }
                }
            });
            
            // Target Chip (read-only, configured by board type)
            ui.horizontal(|ui| {
                ui.label("Target Chip:");
                ui.label(match self.upload_config.target_chip {
                    compiler::TargetChip::ATmega328P => "ATmega328P",
                    compiler::TargetChip::ATtiny85 => "ATtiny85",
                });
            });
            
            // Clock Speed (ATtiny85 only)
            if matches!(self.upload_config.target_chip, compiler::TargetChip::ATtiny85) {
                ui.horizontal(|ui| {
                    ui.label("Clock Speed:");
                    let speed_names = vec!["1MHz Internal", "8MHz Internal", "16MHz External"];
                    let mut selected_index = match self.upload_config.clock_speed {
                        compiler::ClockSpeed::MHz1 => 0,
                        compiler::ClockSpeed::MHz8 => 1,
                        compiler::ClockSpeed::MHz16 => 2,
                    };
                    egui::ComboBox::from_id_source("clock_speed")
                        .selected_text(speed_names[selected_index])
                        .show_ui(ui, |ui| {
                            for (i, name) in speed_names.iter().enumerate() {
                                ui.selectable_value(&mut selected_index, i, *name);
                            }
                        });
                    self.upload_config.clock_speed = match selected_index {
                        0 => compiler::ClockSpeed::MHz1,
                        1 => compiler::ClockSpeed::MHz8,
                        _ => compiler::ClockSpeed::MHz16,
                    };
                });
            }

            // Programmer with presets
            ui.horizontal(|ui| {
                ui.label("Programmer:");
                ui.text_edit_singleline(&mut self.upload_config.programmer);
                
                if ui.button("arduino").clicked() {
                    self.upload_config.programmer = "arduino".to_string();
                }
                if ui.button("stk500v1").clicked() {
                    self.upload_config.programmer = "stk500v1".to_string();
                }
            });
            
            // Baud Rate with presets
            ui.horizontal(|ui| {
                ui.label("Baud Rate:");
                ui.add(egui::DragValue::new(&mut self.upload_config.baud_rate)
                    .speed(1000)
                    .clamp_range(300..=115200));
                
                if ui.button("57600").clicked() {
                    self.upload_config.baud_rate = 57600;
                }
                if ui.button("115200").clicked() {
                    self.upload_config.baud_rate = 115200;
                }
            });
            
            ui.add_space(10.0);
            
            // Detection info
            ui.separator();
            ui.label("Auto-detection:");
            ui.label(format!("Detected: {} -> {} ({})", 
                self.upload_config.port,
                match self.upload_config.target_chip {
                    compiler::TargetChip::ATmega328P => "ATmega328P",
                    compiler::TargetChip::ATtiny85 => "ATtiny85",
                },
                self.upload_config.programmer
            ));
            
            ui.add_space(10.0);
            
            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Reset to Auto-detect").clicked() {
                    if self.hardware_panel.is_connected() {
                        if let Some(ref hardware) = self.hardware_panel.selected_hardware {
                            self.upload_config = compiler::UploadConfig::detect_from_port(&hardware.port);
                        }
                    }
                }
                
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });
        
        if should_close {
            self.show_upload_config_dialog = false;
        }
    }

    fn check_operation_results(&mut self) {
        if let Some(ref mut receiver) = self.operation_receiver {
            // Try to receive results without blocking
            while let Ok(message) = receiver.try_recv() {
                if message == "COMPILATION_SUCCESS" {
                    self.compile_in_progress = false;
                    self.operation_output.push_str("\n✅ Compilation successful!\n");
                    self.status_bar.success("Compilation successful!");
                } else if message == "COMPILATION_FAILED" {
                    self.compile_in_progress = false;
                    self.operation_output.push_str("\n❌ Compilation failed!\n");
                    self.status_bar.error("Compilation failed - see output");
                } else if message == "UPLOAD_SUCCESS" {
                    self.upload_in_progress = false;
                    self.operation_output.push_str("\n✅ Upload successful!\n");
                    self.status_bar.success("Upload successful!");
                } else if message == "UPLOAD_FAILED" {
                    self.upload_in_progress = false;
                    self.operation_output.push_str("\n❌ Upload failed!\n");
                    self.status_bar.error("Upload failed - see output");
                } else if message == "RECONNECT_SERIAL" {
                    // Reconnect serial monitor if it was connected before
                    if let Some(ref hardware) = self.hardware_panel.selected_hardware {
                        let _ = self.hardware_panel.connection.connect(hardware);
                        self.operation_output.push_str("Serial monitor reconnected\n");
                        self.status_bar.info("Serial monitor reconnected");
                    }
                } else {
                    // Regular output message
                    self.operation_output.push_str(&message);
                }
            }
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
        // Only apply theme once at startup to prevent flickering
        // Theme changes are now handled more efficiently
        
        // Optimize repaints - only request continuous repaint when absolutely necessary
        let needs_repaint = self.compile_in_progress 
            || self.upload_in_progress 
            || (self.serial_monitor.enabled && self.hardware_panel.is_connected());
            
        if needs_repaint {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Top panel - Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar.show(ui);
        });

        // Bottom panel - Status bar (must be before side panels and CentralPanel)
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar.show(ui, &self.theme);
        });

        // Handle menu bar actions and hardware detection (OUTSIDE panel closures)
        if self.menu_bar.show_hardware_detection {
            self.show_hardware_dialog = true;
            self.menu_bar.show_hardware_detection = false;
        }
        
        // Handle file menu actions
        if self.menu_bar.new_file_clicked {
            self.code_editor = CodeEditor::new();
            self.status_bar.info("New file created");
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.open_file_clicked {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Source Files", &["ino", "cpp", "c", "h"])
                .add_filter("All Files", &["*"])
                .pick_file()
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    self.code_editor = CodeEditor::new_with_code(content, "cpp".to_string());
                    self.code_editor.file_path = Some(path.to_string_lossy().to_string());
                    self.status_bar.success(&format!("Loaded {}", path.display()));
                } else {
                    self.status_bar.error("Failed to read file");
                }
            }
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.save_file_clicked {
            if let Some(ref path) = self.code_editor.file_path {
                if let Err(e) = std::fs::write(path, &self.code_editor.code) {
                    self.status_bar.error(&format!("Failed to save: {}", e));
                } else {
                    self.code_editor.modified = false;
                    self.status_bar.success("File saved");
                }
            } else {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Source Files", &["ino", "cpp", "c", "h"])
                    .save_file()
                {
                    if let Err(e) = std::fs::write(&path, &self.code_editor.code) {
                        self.status_bar.error(&format!("Failed to save: {}", e));
                    } else {
                        self.code_editor.file_path = Some(path.to_string_lossy().to_string());
                        self.code_editor.modified = false;
                        self.status_bar.success("File saved");
                    }
                }
            }
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.compile_clicked {
            self.compile_code();
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.upload_clicked {
            self.upload_code();
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.upload_config_clicked {
            self.show_upload_config_dialog = true;
            self.menu_bar.reset_action_flags();
        }
        
        if self.menu_bar.toggle_serial_monitor {
            self.serial_monitor_visible = !self.serial_monitor_visible;
            self.menu_bar.reset_action_flags();
        }

        // Handle hardware panel actions
        if self.hardware_panel.disconnect_clicked {
            if self.hardware_panel.connection.is_connected() {
                self.hardware_panel.connection.disconnect().ok();
                self.hardware_panel.selected_hardware = None;
                self.status_bar.info("Hardware disconnected");
            }
            self.hardware_panel.reset_action_flags();
        }
        
        if self.hardware_panel.configure_clicked {
            self.hardware_panel.show_config = !self.hardware_panel.show_config;
            self.hardware_panel.reset_action_flags();
        }

        // Show dialogs
        if self.show_hardware_dialog {
            self.show_hardware_selection_dialog(ctx);
        }
        
        self.show_build_output_dialog(ctx);
        self.show_upload_config_dialog(ctx);

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

        // Left panel - Hardware (fixed 300px width)
        egui::SidePanel::left("hardware_panel")
            .resizable(false)
            .exact_width(300.0)
            .show(ctx, |ui| {
                if let Some(hardware) = self.hardware_panel.show(ui, &self.theme) {
                    self.available_hardware = hardware;
                    self.status_bar.success(&format!("Found {} devices", self.available_hardware.len()));
                    // Open hardware selection dialog when hardware is detected
                    self.show_hardware_dialog = true;
                }
            });
        
        // Right panel - Serial Monitor (fixed 380px width) - only show if visible and not detached
        if self.serial_monitor_visible && !self.serial_monitor_detached {
            egui::SidePanel::right("serial_panel")
                .resizable(false)
                .exact_width(380.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Serial Monitor");
                        ui.add_space(10.0);
                        
                        // Options checkboxes moved to header
                        ui.checkbox(&mut self.serial_monitor.auto_scroll, "Auto");
                        ui.checkbox(&mut self.serial_monitor.show_timestamps, "Time");
                        ui.checkbox(&mut self.serial_monitor.hex_mode, "Hex");
                        
                        ui.add_space(10.0);
                        if ui.button("📎 Detach").clicked() {
                            self.serial_monitor_detached = true;
                        }
                    });
                    ui.separator();
                    self.serial_monitor.show(ui, &self.theme, &mut self.hardware_panel.connection);
                });
        }

        // Show detached serial monitor window if enabled
        let mut should_attach = false;
        if self.serial_monitor_visible && self.serial_monitor_detached {
            let window = egui::Window::new("Serial Monitor")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .default_height(600.0)
                .open(&mut self.serial_monitor_detached)
                .default_pos(egui::pos2(500.0, 100.0));
            
            window.show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Serial Monitor");
                    ui.add_space(10.0);
                    if ui.button("📌 Attach").clicked() {
                        should_attach = true;
                    }
                });
                ui.separator();
                self.serial_monitor.show(ui, &self.theme, &mut self.hardware_panel.connection);
            });
        }
        
        if should_attach {
            self.serial_monitor_detached = false;
        }

        // Center panel - Code Editor (takes remaining space)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate available width and constrain to 90-125 characters
            let available_width = ui.available_width();
            let char_width = 6.0; // Approximate monospace character width
            let min_width = 90.0 * char_width; // 540px
            let max_width = 125.0 * char_width; // 750px
            
            let target_width = if self.serial_monitor_visible && !self.serial_monitor_detached {
                // When Serial Monitor is active, limit to 90 characters
                available_width.min(max_width).min(min_width)
            } else {
                // When Serial Monitor is hidden, allow up to 125 characters
                available_width.min(max_width)
            };
            
            // Set the editor width by constraining the available space
            ui.set_max_width(target_width);
            ui.set_min_width(target_width);
            
            if self.code_editor.show(ui, &self.theme) {
                self.status_bar.warning("Code modified - remember to save");
            }
        });

        // Check for async operation results
        self.check_operation_results();
        
        // Only update serial monitor when visible and connected
        if self.serial_monitor_visible {
            self.update_serial_monitor();
        }

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::R) && i.modifiers.ctrl) {
            self.show_hardware_dialog = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.show_example_dialog = true;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            self.compile_code();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::U) && i.modifiers.ctrl) {
            self.upload_code();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    tracing::info!("=== MAIN FUNCTION CALLED ===");
    
    // Configure window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])  // Reduced default size
            .with_min_inner_size([1200.0, 600.0]) // Reasonable minimum size
            .with_title("Hardware-Aware IDE")
            .with_decorations(true)   // Ensure window decorations are shown
            .with_maximized(false),   // Don't start maximized
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Hardware-Aware IDE",
        options,
        Box::new(|cc| Box::new(HardwareIDE::new(cc))),
    )
}
