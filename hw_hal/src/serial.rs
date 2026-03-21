//! Serial communication module for microcontroller interaction

use crate::{ConnectionConfig, HardwareInterface, HardwareInfo};
use anyhow::{anyhow, Result};
use serialport::{SerialPort, SerialPortInfo};
use std::io::{Read, Write};
use std::time::Duration;
use tracing::{debug, error, info};

pub struct SerialConnection {
    port: Option<Box<dyn SerialPort>>,
    config: ConnectionConfig,
    connected: bool,
}

impl SerialConnection {
    pub fn new() -> Self {
        Self {
            port: None,
            config: ConnectionConfig::default(),
            connected: false,
        }
    }

    pub fn with_config(config: ConnectionConfig) -> Self {
        Self {
            port: None,
            config,
            connected: false,
        }
    }

    /// List available serial ports
    pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
        Ok(serialport::available_ports()?)
    }

    /// Test if a port is accessible
    pub fn test_port(port_name: &str, baud_rate: u32) -> Result<bool> {
        match serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Create serial port from config
    fn create_port_from_config(config: &ConnectionConfig) -> Result<Box<dyn SerialPort>> {
        let data_bits = match config.data_bits.as_str() {
            "Five" => serialport::DataBits::Five,
            "Six" => serialport::DataBits::Six,
            "Seven" => serialport::DataBits::Seven,
            "Eight" => serialport::DataBits::Eight,
            _ => serialport::DataBits::Eight,
        };

        let flow_control = match config.flow_control.as_str() {
            "None" => serialport::FlowControl::None,
            "Software" => serialport::FlowControl::Software,
            "Hardware" => serialport::FlowControl::Hardware,
            _ => serialport::FlowControl::None,
        };

        let parity = match config.parity.as_str() {
            "None" => serialport::Parity::None,
            "Odd" => serialport::Parity::Odd,
            "Even" => serialport::Parity::Even,
            _ => serialport::Parity::None,
        };

        let stop_bits = match config.stop_bits.as_str() {
            "One" => serialport::StopBits::One,
            "Two" => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        };

        let port = serialport::new(&config.port, config.baud_rate)
            .timeout(Duration::from_millis(config.timeout_ms))
            .data_bits(data_bits)
            .flow_control(flow_control)
            .parity(parity)
            .stop_bits(stop_bits)
            .open()?;

        Ok(port)
    }

    /// Read data from the serial port with timeout
    pub fn read_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(port) = &mut self.port {
            match port.read(buffer) {
                Ok(bytes_read) => Ok(bytes_read),
                Err(e) => {
                    error!("Error reading from serial port: {}", e);
                    Err(anyhow!("Serial read error: {}", e))
                }
            }
        } else {
            Err(anyhow!("Not connected to any serial port"))
        }
    }

    /// Write data to the serial port
    pub fn write_data(&mut self, data: &[u8]) -> Result<()> {
        if let Some(port) = &mut self.port {
            match port.write(data) {
                Ok(_) => {
                    port.flush()?;
                    Ok(())
                }
                Err(e) => {
                    error!("Error writing to serial port: {}", e);
                    Err(anyhow!("Serial write error: {}", e))
                }
            }
        } else {
            Err(anyhow!("Not connected to any serial port"))
        }
    }

    /// Read a line of text from the serial port
    pub fn read_line(&mut self) -> Result<String> {
        let mut buffer = [0u8; 1024];
        let mut line = String::new();
        
        loop {
            match self.read_data(&mut buffer) {
                Ok(0) => break, // No more data
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    line.push_str(&chunk);
                    
                    if line.contains('\n') || line.contains('\r') {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        
        // Clean up line endings
        line = line.trim().to_string();
        Ok(line)
    }

    /// Send a command and wait for response
    pub fn send_command_wait(&mut self, command: &str, timeout_ms: u64) -> Result<String> {
        self.write_data(command.as_bytes())?;
        self.write_data(b"\n")?;
        
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        
        while start_time.elapsed() < timeout {
            match self.read_line() {
                Ok(line) if !line.is_empty() => return Ok(line),
                Ok(_) => continue, // Empty line, keep waiting
                Err(_) => continue,
            }
        }
        
        Err(anyhow!("Timeout waiting for response"))
    }
}

impl HardwareInterface for SerialConnection {
    fn detect(&self) -> Result<Vec<HardwareInfo>> {
        let ports = Self::list_ports()?;
        let mut hardware_list = Vec::new();
        
        for port in ports {
            if let Some(hw_info) = crate::detection::analyze_port(&port) {
                hardware_list.push(hw_info);
            }
        }
        
        Ok(hardware_list)
    }

    fn connect(&mut self, info: &HardwareInfo) -> Result<()> {
        info!("Connecting to {} on {} at {} baud", info.name, info.port, info.baud_rate);
        
        // Create a temporary config from hardware info
        let mut temp_config = self.config.clone();
        temp_config.port = info.port.clone();
        temp_config.baud_rate = info.baud_rate;
        
        self.port = Some(Self::create_port_from_config(&temp_config)?);
        self.config = temp_config;
        self.connected = true;
        
        debug!("Successfully connected to {}", info.port);
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        if self.connected {
            info!("Disconnecting from {}", self.config.port);
            self.port = None;
            self.connected = false;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn send_command(&mut self, command: &str) -> Result<String> {
        if !self.connected {
            return Err(anyhow!("Not connected to any device"));
        }
        
        debug!("Sending command: {}", command);
        self.send_command_wait(command, 2000)
    }

    fn reset(&mut self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to any device"));
        }
        
        // Send reset signal via DTR (common for ESP8266/ESP32)
        if let Some(port) = &mut self.port {
            port.write_request_to_send(false)?;
            std::thread::sleep(Duration::from_millis(100));
            port.write_data_terminal_ready(true)?;
            std::thread::sleep(Duration::from_millis(100));
            port.write_data_terminal_ready(false)?;
            std::thread::sleep(Duration::from_millis(1000));
        }
        
        info!("Device reset signal sent");
        Ok(())
    }
}

impl Default for SerialConnection {
    fn default() -> Self {
        Self::new()
    }
}
