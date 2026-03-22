//! Test module for serial port detection

use std::path::Path;
use std::time::Duration;

pub fn test_all_ports() {
    println!("=== Testing All Serial Ports ===");
    
    // Test some common port names manually
    let common_ports = vec![
        // Windows COM ports
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "COM10",
    ];
    
    let baud_rates = vec![9600, 19200, 38400, 57600, 115200];
    
    for port in common_ports {
        if Path::new(port).exists() {
            println!("\nTesting {}:", port);
            
            for baud in &baud_rates {
                match serialport::new(port, *baud)
                    .timeout(Duration::from_millis(100))
                    .open() {
                    Ok(mut port_handle) => {
                        println!("  ✓ Can open at {} baud", baud);
                        
                        // Try to send a simple command and see if we get a response
                        match port_handle.write(b"AT\r\n") {
                            Ok(_) => {
                                let mut buffer = [0u8; 64];
                                match port_handle.read(&mut buffer) {
                                    Ok(bytes_read) => {
                                        if bytes_read > 0 {
                                            let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                                            println!("    Got response: {:?}", response);
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                        
                        // Close the port
                        drop(port_handle);
                        break; // Found a working baud rate
                    }
                    Err(e) => {
                        println!("  ✗ Cannot open at {} baud: {}", baud, e);
                    }
                }
            }
        } else {
            println!("✗ {} does not exist", port);
        }
    }
}
