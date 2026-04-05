//! Arduino compilation using official Arduino core libraries from Arduino IDE

use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub output: String,
    pub hex_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub success: bool,
    pub output: String,
}

/// Find Arduino core library paths
fn find_arduino_core() -> Result<(PathBuf, PathBuf), String> {
    // Check LOCALAPPDATA for Arduino15 (Arduino IDE 1.6+)
    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        let arduino15 = PathBuf::from(localappdata).join("Arduino15");
        
        if arduino15.exists() {
            // Find AVR core version
            let avr_base = arduino15.join("packages").join("arduino").join("hardware").join("avr");
            
            if avr_base.exists() {
                if let Ok(entries) = fs::read_dir(&avr_base) {
                    for entry in entries.flatten() {
                        let version_path = entry.path();
                        let core_path = version_path.join("cores").join("arduino");
                        let variant_path = version_path.join("variants").join("standard");
                        
                        if core_path.exists() && variant_path.exists() {
                            return Ok((core_path, variant_path));
                        }
                    }
                }
            }
        }
    }
    
    Err(format!("Arduino core libraries not found.\n\nLOCALAPPDATA: {:?}\n\nPlease install Arduino IDE from: https://www.arduino.cc/en/software\n\nAfter installation, the IDE will automatically download the AVR core libraries.", 
        env::var("LOCALAPPDATA").unwrap_or_else(|_| "Not set".to_string())))
}

/// Compile Arduino code using official Arduino core
pub fn compile_avr(source_code: &str, build_dir: &Path, target_chip: TargetChip, clock_speed: ClockSpeed) -> CompilationResult {
    let mut output = String::new();
    
    // Find Arduino core libraries
    let (core_path, variant_path) = match find_arduino_core() {
        Ok(paths) => paths,
        Err(e) => {
            return CompilationResult {
                success: false,
                output: e,
                hex_file: None,
            };
        }
    };
    
    output.push_str(&format!("Using Arduino core: {}\n", core_path.display()));
    output.push_str(&format!("Using variant: {}\n", variant_path.display()));
    
    // Get MCU-specific settings
    let (mcu, f_cpu, arduino_board) = match target_chip {
        TargetChip::ATmega328P => ("atmega328p", "16000000L", "ARDUINO_AVR_UNO"),
        TargetChip::ATtiny85 => {
            let freq = match clock_speed {
                ClockSpeed::MHz1 => "1000000L",
                ClockSpeed::MHz8 => "8000000L", 
                ClockSpeed::MHz16 => "16000000L",
            };
            ("attiny85", freq, "ARDUINO_AVR_ATTINY85")
        }
    };
    
    output.push_str(&format!("Target MCU: {}\n", mcu));
    output.push_str(&format!("CPU Frequency: {} Hz\n\n", f_cpu));
    
    // Create source file with Arduino.h include
    let source_file = build_dir.join("sketch.cpp");
    let wrapped_code = format!("#include <Arduino.h>\n\n{}", source_code);
    
    if let Err(e) = fs::write(&source_file, wrapped_code) {
        return CompilationResult {
            success: false,
            output: format!("Failed to write source file: {}", e),
            hex_file: None,
        };
    }
    
    let elf_file = build_dir.join("sketch.elf");
    let hex_file = build_dir.join("sketch.hex");
    
    // Collect all Arduino core .c and .cpp files
    let mut core_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&core_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "c" || ext == "cpp" {
                    core_files.push(path);
                }
            }
        }
    }
    
    output.push_str(&format!("=== Compiling {} Arduino core files ===\n", core_files.len()));
    
    // Compile each core file to object file
    let mut object_files = vec![build_dir.join("sketch.o")];
    
    // First compile user sketch
    output.push_str("Compiling sketch.cpp...\n");
    let sketch_mcu_arg = format!("-mmcu={}", mcu);
    let sketch_f_cpu_arg = format!("-DF_CPU={}", f_cpu);
    let sketch_board_arg = format!("-D{}", arduino_board);
    let core_include_arg = format!("-I{}", core_path.to_str().unwrap());
    let variant_include_arg = format!("-I{}", variant_path.to_str().unwrap());
    
    let sketch_compile = Command::new("avr-g++")
        .args(&[
            "-c",
            "-g",
            "-Os",
            "-w",
            "-std=gnu++11",
            "-fpermissive",
            "-fno-exceptions",
            "-ffunction-sections",
            "-fdata-sections",
            "-fno-threadsafe-statics",
            "-MMD",
            "-flto",
            &sketch_mcu_arg,
            &sketch_f_cpu_arg,
            "-DARDUINO=10819",
            &sketch_board_arg,
            "-DARDUINO_ARCH_AVR",
            &core_include_arg,
            &variant_include_arg,
            source_file.to_str().unwrap(),
            "-o",
            object_files[0].to_str().unwrap(),
        ])
        .output();
    
    match sketch_compile {
        Ok(result) => {
            if !result.status.success() {
                output.push_str(&String::from_utf8_lossy(&result.stderr));
                return CompilationResult {
                    success: false,
                    output,
                    hex_file: None,
                };
            }
        }
        Err(e) => {
            return CompilationResult {
                success: false,
                output: format!("Failed to run avr-g++: {}\n\nMake sure avr-g++ is in your PATH.", e),
                hex_file: None,
            };
        }
    }
    
    // Compile core files
    for (i, core_file) in core_files.iter().enumerate() {
        let obj_file = build_dir.join(format!("core_{}.o", i));
        object_files.push(obj_file.clone());
        
        let is_cpp = core_file.extension().and_then(|e| e.to_str()) == Some("cpp");
        let compiler = if is_cpp { "avr-g++" } else { "avr-gcc" };
        
        let mcu_arg = format!("-mmcu={}", mcu);
        let f_cpu_arg = format!("-DF_CPU={}", f_cpu);
        let board_arg = format!("-D{}", arduino_board);
        
        let mut args = vec![
            "-c",
            "-g",
            "-Os",
            "-w",
            "-ffunction-sections",
            "-fdata-sections",
            "-MMD",
            "-flto",
            &mcu_arg,
            &f_cpu_arg,
            "-DARDUINO=10819",
            &board_arg,
            "-DARDUINO_ARCH_AVR",
        ];
        
        if is_cpp {
            args.extend(&[
                "-std=gnu++11",
                "-fpermissive",
                "-fno-exceptions",
                "-fno-threadsafe-statics",
            ]);
        }
        
        let core_include = format!("-I{}", core_path.to_str().unwrap());
        let variant_include = format!("-I{}", variant_path.to_str().unwrap());
        
        args.extend(&[
            core_include.as_str(),
            variant_include.as_str(),
        ]);
        args.push(core_file.to_str().unwrap());
        args.push("-o");
        args.push(obj_file.to_str().unwrap());
        
        if let Err(e) = Command::new(compiler).args(&args).output() {
            return CompilationResult {
                success: false,
                output: format!("{}Failed to compile {}: {}", output, core_file.display(), e),
                hex_file: None,
            };
        }
    }
    
    output.push_str("✓ Core files compiled\n\n");
    
    // Link all object files
    output.push_str("=== Linking ===\n");
    let mcu_link_arg = format!("-mmcu={}", mcu);
    let mut link_args = vec![
        "-w",
        "-Os",
        "-g",
        "-flto",
        "-fuse-linker-plugin",
        "-Wl,--gc-sections",
        &mcu_link_arg,
    ];
    
    let obj_paths: Vec<String> = object_files.iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();
    
    for obj in &obj_paths {
        link_args.push(obj);
    }
    
    link_args.push("-o");
    link_args.push(elf_file.to_str().unwrap());
    link_args.push("-lm");
    
    let link_result = Command::new("avr-gcc").args(&link_args).output();
    
    match link_result {
        Ok(result) => {
            if !result.status.success() {
                output.push_str(&String::from_utf8_lossy(&result.stderr));
                return CompilationResult {
                    success: false,
                    output,
                    hex_file: None,
                };
            }
        }
        Err(e) => {
            return CompilationResult {
                success: false,
                output: format!("{}Failed to link: {}", output, e),
                hex_file: None,
            };
        }
    }
    
    output.push_str("✓ Linking complete\n\n");
    
    // Convert to HEX
    output.push_str("=== Creating HEX file ===\n");
    let objcopy_result = Command::new("avr-objcopy")
        .args(&[
            "-O",
            "ihex",
            "-R",
            ".eeprom",
            elf_file.to_str().unwrap(),
            hex_file.to_str().unwrap(),
        ])
        .output();
    
    match objcopy_result {
        Ok(result) => {
            if !result.status.success() {
                output.push_str(&String::from_utf8_lossy(&result.stderr));
                return CompilationResult {
                    success: false,
                    output,
                    hex_file: None,
                };
            }
        }
        Err(e) => {
            return CompilationResult {
                success: false,
                output: format!("{}Failed to create HEX: {}", output, e),
                hex_file: None,
            };
        }
    }
    
    // Get size info
    output.push_str("\n=== Program Size ===\n");
    let size_mcu_arg = format!("--mcu={}", mcu);
    if let Ok(size_result) = Command::new("avr-size")
        .args(&["-C", &size_mcu_arg, elf_file.to_str().unwrap()])
        .output()
    {
        output.push_str(&String::from_utf8_lossy(&size_result.stdout));
    }
    
    output.push_str("\n✅ Compilation successful!\n");
    output.push_str(&format!("HEX file: {}\n", hex_file.display()));
    
    CompilationResult {
        success: true,
        output,
        hex_file: Some(hex_file),
    }
}

#[derive(Debug, Clone)]
pub enum TargetChip {
    ATmega328P,        // Arduino Uno/Nano target
    ATtiny85,          // ATtiny85 target
}

#[derive(Debug, Clone)]
pub enum ClockSpeed {
    MHz1,              // 1MHz internal
    MHz8,              // 8MHz internal  
    MHz16,             // 16MHz external/ATmega328P
}

#[derive(Debug, Clone)]
pub enum BoardType {
    ArduinoUno,        // Standard Arduino Uno with bootloader
    ArduinoNanoISP,    // Arduino Nano configured as ISP programmer
}

#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub programmer: String,
    pub target_chip: TargetChip,
    pub clock_speed: ClockSpeed,
    pub baud_rate: u32,
    pub port: String,
    pub board_type: BoardType,
}

impl UploadConfig {
    pub fn detect_from_port(port: &str) -> Self {
        // Default detection logic - assume Arduino Uno/Nano for direct upload
        // User can change to ATtiny85 ISP mode via Upload Configuration
        let (programmer, baud_rate) = match port {
            "COM8" => ("arduino".to_string(), 57600),     // Arduino programmer with CH340 baud rate
            _ => ("arduino".to_string(), 115200),         // Standard Arduino
        };
        
        Self {
            programmer,
            target_chip: TargetChip::ATmega328P,
            clock_speed: ClockSpeed::MHz16,  // ATmega328P uses 16MHz
            baud_rate,
            port: port.to_string(),
            board_type: BoardType::ArduinoUno,
        }
    }
    
    pub fn get_mcu_string(&self) -> &'static str {
        match self.target_chip {
            TargetChip::ATmega328P => "atmega328p",
            TargetChip::ATtiny85 => "attiny85",
        }
    }
}

/// Upload HEX file to Arduino
pub fn upload_avr(hex_file: &Path, config: &UploadConfig) -> UploadResult {
    let mut output = String::new();
    
    output.push_str(&format!("=== Uploading to {} ===\n", config.port));
    output.push_str(&format!("Target: {} ({})\n", config.get_mcu_string(), match config.target_chip {
        TargetChip::ATmega328P => "ATmega328P",
        TargetChip::ATtiny85 => "ATtiny85",
    }));
    output.push_str(&format!("Programmer: {}\n", config.programmer));
    output.push_str(&format!("Baud rate: {}\n", config.baud_rate));
    output.push_str(&format!("Board Type: {:?}\n", config.board_type));
    output.push_str(&format!("Clock Speed: {:?}\n", config.clock_speed));
    
    let mut args = vec![
        "-v".to_string(),
        format!("-p{}", config.get_mcu_string()),
        format!("-c{}", config.programmer),
        format!("-P{}", config.port),
        format!("-b{}", config.baud_rate),
    ];
    
    // Add ATtiny85 specific flags
    if matches!(config.target_chip, TargetChip::ATtiny85) {
        // Try without bit clock flag first
    }
    
    args.push(format!("-Uflash:w:{}:i", hex_file.display()));
    
    output.push_str(&format!("Running: avrdude {}\n", args.join(" ")));
    
    let upload_result = Command::new("avrdude").args(&args).output();
    
    match upload_result {
        Ok(result) => {
            output.push_str(&String::from_utf8_lossy(&result.stdout));
            output.push_str(&String::from_utf8_lossy(&result.stderr));
            
            if result.status.success() {
                output.push_str("\n✅ Upload successful!\n");
                UploadResult {
                    success: true,
                    output,
                }
            } else {
                output.push_str("\n❌ Upload failed!\n");
                UploadResult {
                    success: false,
                    output,
                }
            }
        }
        Err(e) => {
            UploadResult {
                success: false,
                output: format!("Failed to run avrdude: {}\n\nMake sure avrdude is in your PATH.\nAlso check that the device is connected to {}.", e, config.port),
            }
        }
    }
}
