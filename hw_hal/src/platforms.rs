//! Platform-specific implementations and configurations

use crate::{Platform};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub name: String,
    pub default_baud_rate: u32,
    pub upload_speed: u32,
    pub reset_method: ResetMethod,
    pub compiler: CompilerConfig,
    pub programmer: ProgrammerConfig,
    pub board_configs: HashMap<String, BoardConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub name: String,
    pub mcu: String,
    pub f_cpu: u32,
    pub flash_size: u32,
    pub led_pin: Option<u8>,
    pub upload_speed: Option<u32>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResetMethod {
    DTR,        // Use DTR signal (common for ESP8266/ESP32)
    RTS,        // Use RTS signal
    Software,   // Send reset command via serial
    Hardware,   // Use hardware reset pin
    None,       // No reset required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub toolchain: String,
    pub compiler: String,
    pub linker: String,
    pub objcopy: String,
    pub size: String,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgrammerConfig {
    pub protocol: String,
    pub tool: String,
    pub port_option: String,
    pub baud_option: String,
    pub extra_options: Vec<String>,
}

impl PlatformConfig {
    pub fn get_config(platform: &Platform) -> Result<Self> {
        match platform {
            Platform::ESP8266 => Ok(esp8266_config()),
            Platform::ESP32 => Ok(esp32_config()),
            Platform::AVR => Ok(avr_config()),
            Platform::Unknown => Err(anyhow::anyhow!("Unknown platform configuration")),
        }
    }

    pub fn get_board_config(&self, board_name: &str) -> Option<&BoardConfig> {
        self.board_configs.get(board_name)
    }
}

fn esp8266_config() -> PlatformConfig {
    let mut board_configs = HashMap::new();
    
    // WeMos D1 Mini
    board_configs.insert("wemos_d1_mini".to_string(), BoardConfig {
        name: "WeMos D1 Mini".to_string(),
        mcu: "ESP8266".to_string(),
        f_cpu: 80_000_000,
        flash_size: 4_194_304, // 4MB
        led_pin: Some(2), // Built-in LED on GPIO2
        upload_speed: Some(921600),
        vid: Some(0x1A86),
        pid: Some(0x7523),
    });

    // Generic ESP8266
    board_configs.insert("generic_esp8266".to_string(), BoardConfig {
        name: "Generic ESP8266".to_string(),
        mcu: "ESP8266".to_string(),
        f_cpu: 80_000_000,
        flash_size: 1_048_576, // 1MB default
        led_pin: Some(2),
        upload_speed: Some(115200),
        vid: None,
        pid: None,
    });

    PlatformConfig {
        name: "ESP8266".to_string(),
        default_baud_rate: 115200,
        upload_speed: 115200,
        reset_method: ResetMethod::DTR,
        compiler: CompilerConfig {
            toolchain: "xtensa-lx106-elf".to_string(),
            compiler: "xtensa-lx106-elf-gcc".to_string(),
            linker: "xtensa-lx106-elf-gcc".to_string(),
            objcopy: "xtensa-lx106-elf-objcopy".to_string(),
            size: "xtensa-lx106-elf-size".to_string(),
            flags: vec![
                "-Os".to_string(),
                "-g".to_string(),
                "-Wpointer-arith".to_string(),
                "-Wno-write-strings".to_string(),
                "-Wall".to_string(),
                "-Wno-comment".to_string(),
                "-ffunction-sections".to_string(),
                "-fdata-sections".to_string(),
                "-fno-exceptions".to_string(),
                "-fno-rtti".to_string(),
                "-fno-common".to_string(),
                "-mmcu=esp8266".to_string(),
                "-DF_CPU=80000000L".to_string(),
            ],
        },
        programmer: ProgrammerConfig {
            protocol: "esptool".to_string(),
            tool: "esptool.py".to_string(),
            port_option: "--port".to_string(),
            baud_option: "--baud".to_string(),
            extra_options: vec![
                "--chip".to_string(),
                "esp8266".to_string(),
                "write_flash".to_string(),
                "--flash_mode".to_string(),
                "dio".to_string(),
                "--flash_freq".to_string(),
                "40m".to_string(),
            ],
        },
        board_configs,
    }
}

fn esp32_config() -> PlatformConfig {
    let mut board_configs = HashMap::new();
    
    // Generic ESP32
    board_configs.insert("generic_esp32".to_string(), BoardConfig {
        name: "Generic ESP32".to_string(),
        mcu: "ESP32".to_string(),
        f_cpu: 240_000_000,
        flash_size: 4_194_304, // 4MB
        led_pin: Some(2),
        upload_speed: Some(921600),
        vid: None,
        pid: None,
    });

    PlatformConfig {
        name: "ESP32".to_string(),
        default_baud_rate: 115200,
        upload_speed: 115200,
        reset_method: ResetMethod::DTR,
        compiler: CompilerConfig {
            toolchain: "xtensa-esp32-elf".to_string(),
            compiler: "xtensa-esp32-elf-gcc".to_string(),
            linker: "xtensa-esp32-elf-gcc".to_string(),
            objcopy: "xtensa-esp32-elf-objcopy".to_string(),
            size: "xtensa-esp32-elf-size".to_string(),
            flags: vec![
                "-Os".to_string(),
                "-g".to_string(),
                "-Wpointer-arith".to_string(),
                "-Wno-write-strings".to_string(),
                "-Wall".to_string(),
                "-Wno-comment".to_string(),
                "-ffunction-sections".to_string(),
                "-fdata-sections".to_string(),
                "-fno-exceptions".to_string(),
                "-fno-rtti".to_string(),
                "-fno-common".to_string(),
                "-mmcu=esp32".to_string(),
                "-DF_CPU=240000000L".to_string(),
            ],
        },
        programmer: ProgrammerConfig {
            protocol: "esptool".to_string(),
            tool: "esptool.py".to_string(),
            port_option: "--port".to_string(),
            baud_option: "--baud".to_string(),
            extra_options: vec![
                "--chip".to_string(),
                "esp32".to_string(),
                "write_flash".to_string(),
                "--flash_mode".to_string(),
                "dio".to_string(),
                "--flash_freq".to_string(),
                "40m".to_string(),
            ],
        },
        board_configs,
    }
}

fn avr_config() -> PlatformConfig {
    let mut board_configs = HashMap::new();
    
    // Arduino Uno
    board_configs.insert("arduino_uno".to_string(), BoardConfig {
        name: "Arduino Uno".to_string(),
        mcu: "atmega328p".to_string(),
        f_cpu: 16_000_000,
        flash_size: 32_768, // 32KB
        led_pin: Some(13), // Built-in LED on pin 13
        upload_speed: Some(115200),
        vid: Some(0x2341),
        pid: Some(0x0043),
    });

    PlatformConfig {
        name: "AVR".to_string(),
        default_baud_rate: 9600,
        upload_speed: 19200,
        reset_method: ResetMethod::DTR,
        compiler: CompilerConfig {
            toolchain: "avr".to_string(),
            compiler: "avr-gcc".to_string(),
            linker: "avr-gcc".to_string(),
            objcopy: "avr-objcopy".to_string(),
            size: "avr-size".to_string(),
            flags: vec![
                "-Os".to_string(),
                "-g".to_string(),
                "-ffunction-sections".to_string(),
                "-fdata-sections".to_string(),
                "-mmcu=atmega328p".to_string(),
                "-DF_CPU=16000000L".to_string(),
            ],
        },
        programmer: ProgrammerConfig {
            protocol: "avrdude".to_string(),
            tool: "avrdude".to_string(),
            port_option: "-P".to_string(),
            baud_option: "-b".to_string(),
            extra_options: vec![
                "-c".to_string(),
                "arduino".to_string(),
                "-p".to_string(),
                "atmega328p".to_string(),
                "-U".to_string(),
                "flash:w:{binary}:i".to_string(),
            ],
        },
        board_configs,
    }
}
