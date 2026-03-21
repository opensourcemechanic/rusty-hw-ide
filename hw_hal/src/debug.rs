//! Debug module for testing hardware detection

use serialport;

pub fn debug_serial_ports() {
    println!("=== Serial Port Debug Information ===");
    
    match serialport::available_ports() {
        Ok(ports) => {
            println!("Found {} serial ports:", ports.len());
            for (i, port) in ports.iter().enumerate() {
                println!("{}. Port: {}", i + 1, port.port_name);
                println!("   Port type: {:?}", port.port_type);
                
                // Try to get more detailed info
                match &port.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        println!("   USB Info:");
                        println!("     VID: {:04X}", info.vid);
                        println!("     PID: {:04X}", info.pid);
                        println!("     Product: {:?}", info.product);
                        println!("     Manufacturer: {:?}", info.manufacturer);
                        println!("     Serial Number: {:?}", info.serial_number);
                    }
                    serialport::SerialPortType::PciPort => {
                        println!("   PCI Port");
                    }
                    serialport::SerialPortType::BluetoothPort => {
                        println!("   Bluetooth Port");
                    }
                    serialport::SerialPortType::Unknown => {
                        println!("   Unknown Port Type");
                    }
                }
                println!();
            }
        }
        Err(e) => {
            println!("Error scanning ports: {}", e);
        }
    }
    
    // Test some common port names manually
    let common_ports = vec![
        "/dev/ttyUSB0", "/dev/ttyUSB1", "/dev/ttyUSB2",
        "/dev/ttyACM0", "/dev/ttyACM1", "/dev/ttyACM2",
        "/dev/ttyS0", "/dev/ttyS1", "/dev/ttyS2", "/dev/ttyS3", "/dev/ttyS4", "/dev/ttyS5", "/dev/ttyS6", "/dev/ttyS7",
    ];
    
    println!("Testing common port names:");
    for port_name in common_ports {
        if std::path::Path::new(port_name).exists() {
            println!("✓ {} exists", port_name);
            
            // Try to open it briefly
            match serialport::new(port_name, 115200).timeout(std::time::Duration::from_millis(100)).open() {
                Ok(_) => println!("  ✓ Can open port"),
                Err(e) => println!("  ✗ Cannot open port: {}", e),
            }
        } else {
            println!("✗ {} does not exist", port_name);
        }
    }
}
