//! Hardware detection module for identifying connected microcontrollers

use crate::{HardwareInfo, Platform};
use anyhow::Result;
use serialport::SerialPortInfo;

/// Detects all available serial ports and attempts to identify connected hardware
pub fn detect_hardware() -> Result<Vec<HardwareInfo>> {
    let mut hardware_list = Vec::new();
    
    // First try the serialport library detection
    if let Ok(ports) = serialport::available_ports() {
        for port in ports {
            if let Some(hw_info) = analyze_port(&port) {
                hardware_list.push(hw_info);
            }
        }
    }
    
    // Then manually check WSL COM ports (ttyS*) that might not be detected by serialport
    for i in 0..8 {
        let port_name = format!("/dev/ttyS{}", i);
        if std::path::Path::new(&port_name).exists() {
            // Try to test if we can access the port
            if test_port_access(&port_name) {
                if let Some(hw_info) = analyze_wsl_port(&port_name) {
                    hardware_list.push(hw_info);
                }
            }
        }
    }
    
    Ok(hardware_list)
}

/// Tests if we can access a port
fn test_port_access(port_name: &str) -> bool {
    match serialport::new(port_name, 115200)
        .timeout(std::time::Duration::from_millis(100))
        .open() {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Analyzes WSL COM ports specifically
fn analyze_wsl_port(port_name: &str) -> Option<HardwareInfo> {
    // For WSL, we assume ttyS ports are forwarded Windows COM ports
    // Default to AVR/Arduino since that's most common for CH341 chips
    Some(HardwareInfo {
        name: "Arduino/AVR Device (WSL)".to_string(),
        platform: Platform::AVR,
        port: port_name.to_string(),
        baud_rate: 9600, // Arduino default
        chip_id: None,
        description: Some("WSL COM Port Forward".to_string()),
    })
}

/// Analyzes a serial port to determine if it's a supported microcontroller
pub fn analyze_port(port_info: &SerialPortInfo) -> Option<HardwareInfo> {
    let port_name = &port_info.port_name;
    
    // Try to identify by port name patterns (simplified approach)
    // In a real implementation, you might use rusb or other USB libraries
    // to get VID/PID information
    if let Some(platform) = identify_by_port_name(port_name) {
        return Some(HardwareInfo {
            name: format!("Unknown {} Device", platform),
            platform: platform.clone(),
            port: port_name.clone(),
            baud_rate: get_default_baud_rate(&platform),
            chip_id: None,
            description: Some(port_info.port_name.clone()),
        });
    }

    // Try to identify known devices by port name patterns
    if let Some((platform, name)) = identify_known_device(port_name) {
        return Some(HardwareInfo {
            name,
            platform: platform.clone(),
            port: port_name.clone(),
            baud_rate: get_default_baud_rate(&platform),
            chip_id: None,
            description: Some(port_info.port_name.clone()),
        });
    }

    None
}

/// Identifies known devices by port name patterns
fn identify_known_device(port_name: &str) -> Option<(Platform, String)> {
    let port_lower = port_name.to_lowercase();
    
    // WSL COM port forwarding - assume Arduino/AVR for ttyS ports
    if port_lower.contains("ttys") {
        Some((Platform::AVR, "Arduino/AVR Device (WSL)".to_string()))
    }
    // WeMos D1 Mini patterns
    else if port_lower.contains("ch340") || port_lower.contains("wchusb") {
        Some((Platform::ESP8266, "WeMos D1 Mini".to_string()))
    }
    // ESP32 patterns
    else if port_lower.contains("cp2102") || port_lower.contains("silabs") {
        Some((Platform::ESP32, "ESP32 Dev Board".to_string()))
    }
    // Arduino patterns
    else if port_lower.contains("acm") || port_lower.contains("usbmodem") {
        Some((Platform::AVR, "Arduino Board".to_string()))
    }
    else {
        None
    }
}

/// Identifies platform based on port name patterns
fn identify_by_port_name(port_name: &str) -> Option<Platform> {
    let port_lower = port_name.to_lowercase();
    
    // WSL COM port forwarding (Windows COM ports appear as ttyS)
    if port_lower.contains("ttys") {
        // Could be any platform, need further investigation
        None
    }
    // Common patterns for different platforms
    else if port_lower.contains("cu.usbserial") || port_lower.contains("ttyusb") {
        None // Could be any platform, need further investigation
    } else if port_lower.contains("cu.wchusbserial") || port_lower.contains("ttych341usb") {
        Some(Platform::ESP8266) // Common for ESP8266 programming boards
    } else if port_lower.contains("cu.silabs") || port_lower.contains("ttysilab") {
        Some(Platform::ESP32) // Common for ESP32 development boards
    } else if port_lower.contains("cu.usbmodem") || port_lower.contains("ttyacm") {
        Some(Platform::AVR) // Common for Arduino boards
    } else {
        None
    }
}

/// Gets default baud rate for a platform
fn get_default_baud_rate(platform: &Platform) -> u32 {
    match platform {
        Platform::ESP8266 => 115200,
        Platform::ESP32 => 115200,
        Platform::AVR => 9600,
        Platform::Unknown => 115200,
    }
}

/// Attempts to communicate with a device to confirm its platform
pub fn verify_platform(port_name: &str, baud_rate: u32) -> Result<Option<Platform>> {
    use std::time::Duration;
    
    match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open() {
        Ok(_) => {
            // Try to send AT commands (common for ESP8266)
            // and check for typical responses
            
            // This is a simplified verification - in a real implementation,
            // you would send specific commands and parse responses
            
            Ok(Some(Platform::ESP8266)) // Simplified for now
        }
        Err(_) => Ok(None),
    }
}
