//! Hardware detection module for identifying connected microcontrollers

use crate::{HardwareInfo, Platform};
use anyhow::Result;
use serialport::SerialPortInfo;

/// Detects all available serial ports and attempts to identify connected hardware
pub fn detect_hardware() -> Result<Vec<HardwareInfo>> {
    let mut hardware_list = Vec::new();
    let mut seen_ports = std::collections::HashSet::new();
    
    println!("=== Hardware Detection Debug ===");
    
    // First try the serialport library detection
    if let Ok(ports) = serialport::available_ports() {
        println!("Found {} ports from serialport library", ports.len());
        for (i, port) in ports.iter().enumerate() {
            println!("Analyzing port {}: {}", i + 1, port.port_name);
            
            // Skip if we've already seen this port
            if seen_ports.contains(&port.port_name) {
                println!("⚠ Skipping duplicate port: {}", port.port_name);
                continue;
            }
            
            if let Some(hw_info) = analyze_port(&port) {
                println!("✓ Detected: {} on {}", hw_info.name, hw_info.port);
                hardware_list.push(hw_info);
                seen_ports.insert(port.port_name.clone());
            } else {
                println!("✗ No hardware detected for {}", port.port_name);
            }
        }
    } else {
        println!("Error: Could not get serial ports");
    }
    
    println!("Total hardware detected: {}", hardware_list.len());
    
    // On Windows, manually check common COM ports that might not be detected
    #[cfg(target_os = "windows")]
    {
        for i in 1..=20 {
            let port_name = format!("COM{}", i);
            if test_port_access(&port_name) {
                println!("✓ Found accessible COM port: {}", port_name);
                if let Some(hw_info) = analyze_windows_com_port(&port_name) {
                    hardware_list.push(hw_info);
                }
            } else {
                println!("✗ COM{} does not exist or is inaccessible", i);
            }
        }
    }
    
    // On Linux, check for USB serial devices and udev information
    #[cfg(target_os = "linux")]
    {
        println!("=== Checking Linux USB Serial Devices ===");
        
        // Check for common USB serial device paths
        check_native_usb_devices(&mut hardware_list, &mut seen_ports);
        
        // Also check udev for Arduino devices
        check_udev_devices(&mut hardware_list);
    }
    
    Ok(hardware_list)
}

/// Check for native USB serial devices (not COM port forwarded)
#[cfg(target_os = "linux")]
fn check_native_usb_devices(hardware_list: &mut Vec<HardwareInfo>, seen_ports: &mut std::collections::HashSet<String>) {
    println!("=== Checking Native USB Serial Devices ===");
    
    // Common USB serial device paths in Linux
    let usb_paths = vec![
        "/dev/ttyUSB0", "/dev/ttyUSB1", "/dev/ttyUSB2", "/dev/ttyUSB3",
        "/dev/ttyACM0", "/dev/ttyACM1", "/dev/ttyACM2", "/dev/ttyACM3",
    ];
    
    for path in usb_paths {
        // Skip if we've already seen this port
        if seen_ports.contains(path) {
            println!("  ⚠ Skipping already detected port: {}", path);
            continue;
        }
        
        if std::path::Path::new(path).exists() {
            println!("  Found USB device: {}", path);
            
            if test_port_access(path) {
                println!("  ✓ Can access {}", path);
                
                // Try to get more info about the device
                if let Ok(ports) = serialport::available_ports() {
                    if let Some(port_info) = ports.iter().find(|p| p.port_name == path) {
                        if let Some(hw_info) = analyze_port(port_info) {
                            hardware_list.push(hw_info);
                            seen_ports.insert(path.to_string());
                        }
                    } else {
                        // Fallback for ports not detected by serialport
                        let hw_info = HardwareInfo {
                            name: "USB Serial Device".to_string(),
                            platform: Platform::AVR,
                            port: path.to_string(),
                            baud_rate: 9600,
                            chip_id: None,
                            description: Some("Native USB Serial".to_string()),
                        };
                        hardware_list.push(hw_info);
                        seen_ports.insert(path.to_string());
                    }
                }
            } else {
                println!("  ✗ Cannot access {}", path);
            }
        }
    }
}

/// Check udev for Arduino devices on Linux
#[cfg(target_os = "linux")]
fn check_udev_devices(hardware_list: &mut Vec<HardwareInfo>) {
    println!("=== Checking udev for Arduino Devices ===");
    
    // Check for CH340/CH341 devices specifically
    if let Ok(output) = std::process::Command::new("sh")
        .args(&["-c", "dmesg | grep -i 'ch340\\|ch341\\|arduino'"])
        .output()
    {
        let dmesg_info = String::from_utf8_lossy(&output.stdout);
        if !dmesg_info.trim().is_empty() {
            println!("Arduino-related kernel messages:");
            println!("{}", dmesg_info);
        }
    }
    
    // Check if ch340 driver is loaded
    if let Ok(output) = std::process::Command::new("sh")
        .args(&["-c", "lsmod | grep ch340"])
        .output()
    {
        let lsmod_info = String::from_utf8_lossy(&output.stdout);
        if !lsmod_info.trim().is_empty() {
            println!("CH340 driver loaded: {}", lsmod_info);
        } else {
            println!("CH340 driver not loaded - this may be the issue");
        }
    }
    
    // Check for USB devices with Arduino-related VID/PID
    if let Ok(output) = std::process::Command::new("sh")
        .args(&["-c", "lsusb | grep -i '1a86\\|0403\\|10c4'"])
        .output()
    {
        let usb_info = String::from_utf8_lossy(&output.stdout);
        if !usb_info.trim().is_empty() {
            println!("Found potential Arduino USB devices:");
            println!("{}", usb_info);
        } else {
            println!("No Arduino USB devices found with lsusb");
        }
    }
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

/// Analyzes Windows COM ports specifically
#[cfg(target_os = "windows")]
fn analyze_windows_com_port(port_name: &str) -> Option<HardwareInfo> {
    println!("  analyze_windows_com_port: {}", port_name);
    
    // For Windows COM ports, default to Arduino/AVR since that's most common
    // But try to be more specific based on port number
    let (name, baud_rate) = match port_name {
        "COM8" => ("Arduino Nano CH340 (COM8)".to_string(), 57600), // CH340 specific settings
        "COM3" => ("Arduino Uno (COM3)".to_string(), 115200),
        _ => ("Arduino/AVR Device".to_string(), 115200),
    };
    
    Some(HardwareInfo {
        name,
        platform: Platform::AVR,
        port: port_name.to_string(),
        baud_rate,
        chip_id: None,
        description: Some(format!("Windows COM Port - {}", port_name)),
    })
}

/// Analyzes a serial port to determine if it's a supported microcontroller
pub fn analyze_port(port_info: &SerialPortInfo) -> Option<HardwareInfo> {
    let port_name = &port_info.port_name;
    println!("  Analyzing port: {}", port_name);
    
    // Try to identify by USB VID/PID and port name patterns
    if let serialport::SerialPortType::UsbPort(usb_info) = &port_info.port_type {
        println!("  USB Port detected: VID={:04X}, PID={:04X}", usb_info.vid, usb_info.pid);
        if let Some((platform, device_name)) = identify_known_device(port_name, usb_info.vid, usb_info.pid) {
            println!("  ✓ Identified as: {} ({})", device_name, platform);
            return Some(HardwareInfo {
                name: device_name,
                platform: platform.clone(),
                port: port_name.clone(),
                baud_rate: get_default_baud_rate(&platform),
                chip_id: Some(format!("{:04X}:{:04X}", usb_info.vid, usb_info.pid)),
                description: Some(format!("{} - {}", usb_info.product.as_ref().unwrap_or(&"Unknown".to_string()), 
                                         usb_info.manufacturer.as_ref().unwrap_or(&"Unknown".to_string()))),
            });
        } else {
            println!("  ✗ identify_known_device returned None");
        }
    } else {
        println!("  Not a USB port: {:?}", port_info.port_type);
    }
    
    // Fallback to port name pattern detection
    if let Some(platform) = identify_by_port_name(port_name) {
        println!("  ✓ Identified by port name: {}", platform);
        return Some(HardwareInfo {
            name: format!("Unknown {} Device", platform),
            platform: platform.clone(),
            port: port_name.clone(),
            baud_rate: get_default_baud_rate(&platform),
            chip_id: None,
            description: Some(port_info.port_name.clone()),
        });
    } else {
        println!("  ✗ identify_by_port_name returned None");
    }

    println!("  ✗ No identification possible for {}", port_name);
    None
}

/// Identifies known devices by port name patterns and USB VID/PID
fn identify_known_device(port_name: &str, vid: u16, pid: u16) -> Option<(Platform, String)> {
    let port_lower = port_name.to_lowercase();
    println!("    identify_known_device: port={}, VID={:04X}, PID={:04X}", port_name, vid, pid);
    
    // Windows COM ports - identify by VID/PID
    if port_lower.contains("com") {
        println!("    Windows COM port detected");
        match (vid, pid) {
            // FTDI devices (common for Arduino clones)
            (0x0403, 0x6001) => {
                println!("    ✓ Matched FTDI Arduino");
                Some((Platform::AVR, "Arduino (FTDI)".to_string()))
            },
            // Silicon Labs CP210x (common for ESP32/ESP8266)
            (0x10c4, 0xea60) => {
                println!("    ✓ Matched WeMos D1 Mini Pro");
                Some((Platform::ESP8266, "WeMos D1 Mini Pro".to_string()))
            },
            // CH340/CH341 (common Arduino clones)
            (0x1a86, 0x7523) => {
                println!("    ✓ Matched CH340 Arduino");
                Some((Platform::AVR, "Arduino (CH340)".to_string()))
            },
            // Other common patterns
            (0x10c4, _) => {
                println!("    ✓ Matched Silicon Labs (ESP32)");
                Some((Platform::ESP32, "ESP32 Dev Board".to_string()))
            },
            (0x0403, _) => {
                println!("    ✓ Matched FTDI Device");
                Some((Platform::AVR, "FTDI Device".to_string()))
            },
            _ => {
                println!("    ✗ No VID/PID match found");
                Some((Platform::AVR, "Unknown USB Device".to_string()))
            },
        }
    }
    // Linux USB serial devices
    else if port_lower.contains("ttyusb") || port_lower.contains("ttyacm") {
        println!("    Linux USB serial port detected");
        match (vid, pid) {
            // FTDI devices (common for Arduino clones)
            (0x0403, 0x6001) => {
                println!("    ✓ Matched FTDI Arduino");
                Some((Platform::AVR, "Arduino (FTDI)".to_string()))
            },
            // Silicon Labs CP210x (common for ESP32/ESP8266)
            (0x10c4, 0xea60) => {
                println!("    ✓ Matched WeMos D1 Mini Pro");
                Some((Platform::ESP8266, "WeMos D1 Mini Pro".to_string()))
            },
            // CH340/CH341 (common Arduino clones)
            (0x1a86, 0x7523) => {
                println!("    ✓ Matched CH340 Arduino");
                Some((Platform::AVR, "Arduino (CH340)".to_string()))
            },
            // Other common patterns
            (0x10c4, _) => {
                println!("    ✓ Matched Silicon Labs (ESP32)");
                Some((Platform::ESP32, "ESP32 Dev Board".to_string()))
            },
            (0x0403, _) => {
                println!("    ✓ Matched FTDI Device");
                Some((Platform::AVR, "FTDI Device".to_string()))
            },
            _ => {
                println!("    ✗ No VID/PID match found");
                Some((Platform::AVR, "Unknown USB Device".to_string()))
            },
        }
    }
    // Linux/macOS patterns
    else if port_lower.contains("ttys") {
        println!("    Linux/WSL ttyS port detected");
        Some((Platform::AVR, "Arduino/AVR Device (WSL)".to_string()))
    }
    // WeMos D1 Mini patterns
    else if port_lower.contains("ch340") || port_lower.contains("wchusb") {
        println!("    CH340/WCHUSB device detected");
        Some((Platform::ESP8266, "WeMos D1 Mini".to_string()))
    }
    // ESP32 patterns
    else if port_lower.contains("cp2102") || port_lower.contains("silabs") {
        println!("    CP2102/Silabs device detected");
        Some((Platform::ESP32, "ESP32 Dev Board".to_string()))
    }
    // Arduino patterns
    else if port_lower.contains("acm") || port_lower.contains("usbmodem") {
        println!("    ACM/USB modem device detected");
        Some((Platform::AVR, "Arduino Board".to_string()))
    }
    else {
        println!("    ✗ No port pattern match");
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
