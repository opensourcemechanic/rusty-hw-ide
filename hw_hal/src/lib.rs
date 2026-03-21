//! Hardware Abstraction Layer for Microcontroller IDE
//! 
//! This module provides a unified interface for detecting and communicating
//! with various microcontroller platforms including ESP8266, ESP32, and AVR.

pub mod detection;
pub mod serial;
pub mod platforms;
pub mod config;
pub mod debug;
pub mod test;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub name: String,
    pub platform: Platform,
    pub port: String,
    pub baud_rate: u32,
    pub chip_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    ESP8266,
    ESP32,
    AVR,
    Unknown,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::ESP8266 => write!(f, "ESP8266"),
            Platform::ESP32 => write!(f, "ESP32"),
            Platform::AVR => write!(f, "AVR"),
            Platform::Unknown => write!(f, "Unknown"),
        }
    }
}

pub trait HardwareInterface {
    fn detect(&self) -> Result<Vec<HardwareInfo>>;
    fn connect(&mut self, info: &HardwareInfo) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn send_command(&mut self, command: &str) -> Result<String>;
    fn reset(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: String,
    pub flow_control: String,
    pub parity: String,
    pub stop_bits: String,
    pub timeout_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            data_bits: "Eight".to_string(),
            flow_control: "None".to_string(),
            parity: "None".to_string(),
            stop_bits: "One".to_string(),
            timeout_ms: 1000,
        }
    }
}
