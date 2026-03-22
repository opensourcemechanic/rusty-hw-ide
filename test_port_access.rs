use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("=== Testing Direct Port Access ===");
    
    let ports = vec![
        "/dev/ttyS0", "/dev/ttyS1", "/dev/ttyS2", "/dev/ttyS3", 
        "/dev/ttyS4", "/dev/ttyS5", "/dev/ttyS6", "/dev/ttyS7",
    ];
    
    for port in ports {
        if Path::new(port).exists() {
            println!("\nTesting {}:", port);
            
            // Try direct file access
            match OpenOptions::new()
                .read(true)
                .write(true)
                .open(port) {
                Ok(_) => {
                    println!("  ✓ Can open file directly");
                    
                    // Try to write something
                    match OpenOptions::new()
                        .write(true)
                        .open(port) {
                        Ok(mut file) => {
                            match file.write_all(b"AT\r\n") {
                                Ok(_) => println!("  ✓ Can write to port"),
                                Err(e) => println!("  ✗ Cannot write: {}", e),
                            }
                        }
                        Err(e) => println!("  ✗ Cannot open for writing: {}", e),
                    }
                }
                Err(e) => println!("  ✗ Cannot open file: {}", e),
            }
        } else {
            println!("✗ {} does not exist", port);
        }
    }
    
    // Check Windows COM port forwarding status
    println!("\n=== Checking WSL COM Port Forwarding ===");
    
    // Try to read /proc/tty/driver/serial for active ports
    if Path::new("/proc/tty/driver/serial").exists() {
        match std::fs::read_to_string("/proc/tty/driver/serial") {
            Ok(content) => {
                println!("Active serial ports from /proc/tty/driver/serial:");
                for line in content.lines() {
                    if line.contains("ttyS") {
                        println!("  {}", line);
                    }
                }
            }
            Err(_) => println!("Cannot read /proc/tty/driver/serial"),
        }
    }
}
