use std::env;
use std::path::{Path, PathBuf};
use std::fs;

fn find_arduino_core() -> Result<(PathBuf, PathBuf), String> {
    // First try Linux paths
    let linux_paths = vec![
        PathBuf::from("/home").join(env::var("USER").unwrap_or_else(|_| "user".to_string())).join(".arduino15"),
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| "".to_string())).join(".arduino15"),
    ];
    
    for arduino15 in linux_paths {
        if arduino15.exists() {
            println!("Checking Arduino IDE path: {}", arduino15.display());
            
            // Find AVR core version
            let avr_base = arduino15.join("packages").join("arduino").join("hardware").join("avr");
            
            if avr_base.exists() {
                if let Ok(entries) = fs::read_dir(&avr_base) {
                    for entry in entries.flatten() {
                        let version_path = entry.path();
                        let core_path = version_path.join("cores").join("arduino");
                        let variant_path = version_path.join("variants").join("standard");
                        
                        if core_path.exists() && variant_path.exists() {
                            println!("Found Arduino core: {}", core_path.display());
                            return Ok((core_path, variant_path));
                        }
                    }
                }
            }
        }
    }
    
    Err("Arduino core not found".to_string())
}

fn main() {
    println!("Testing Arduino core detection...");
    
    match find_arduino_core() {
        Ok((core_path, variant_path)) => {
            println!("✓ Arduino core found!");
            println!("  Core path: {}", core_path.display());
            println!("  Variant path: {}", variant_path.display());
            
            // Check for Arduino.h
            let arduino_h = core_path.join("Arduino.h");
            if arduino_h.exists() {
                println!("✓ Arduino.h found at: {}", arduino_h.display());
            } else {
                println!("✗ Arduino.h not found");
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }
}
