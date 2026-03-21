//! Configuration management for hardware settings

use crate::{ConnectionConfig, Platform};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hardware: HardwareConfig,
    ui: UIConfig,
    compiler: CompilerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub auto_detect: bool,
    pub auto_connect: bool,
    pub default_platform: Platform,
    pub connection_configs: Vec<ConnectionConfig>,
    pub preferred_ports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub font_size: f32,
    pub window_width: u32,
    pub window_height: u32,
    pub show_serial_monitor: bool,
    pub auto_scroll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub toolchain_paths: std::collections::HashMap<String, String>,
    pub optimization_level: String,
    pub debug_symbols: bool,
    pub verbose_output: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hardware: HardwareConfig::default(),
            ui: UIConfig::default(),
            compiler: CompilerConfig::default(),
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            auto_connect: false,
            default_platform: Platform::ESP8266,
            connection_configs: vec![ConnectionConfig::default()],
            preferred_ports: Vec::new(),
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14.0,
            window_width: 1200,
            window_height: 800,
            show_serial_monitor: true,
            auto_scroll: true,
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        let mut toolchain_paths = std::collections::HashMap::new();
        toolchain_paths.insert("esp8266".to_string(), "".to_string());
        toolchain_paths.insert("esp32".to_string(), "".to_string());
        toolchain_paths.insert("avr".to_string(), "".to_string());

        Self {
            toolchain_paths,
            optimization_level: "Os".to_string(),
            debug_symbols: true,
            verbose_output: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Add a connection configuration
    pub fn add_connection_config(&mut self, config: ConnectionConfig) {
        self.hardware.connection_configs.push(config);
    }

    /// Get connection configuration for a port
    pub fn get_connection_config(&self, port: &str) -> Option<&ConnectionConfig> {
        self.hardware.connection_configs
            .iter()
            .find(|config| config.port == port)
    }

    /// Update connection configuration
    pub fn update_connection_config(&mut self, port: &str, new_config: ConnectionConfig) {
        if let Some(config) = self.hardware.connection_configs
            .iter_mut()
            .find(|config| config.port == port) {
            *config = new_config;
        } else {
            self.hardware.connection_configs.push(new_config);
        }
    }

    /// Add preferred port
    pub fn add_preferred_port(&mut self, port: String) {
        if !self.hardware.preferred_ports.contains(&port) {
            self.hardware.preferred_ports.push(port);
        }
    }

    /// Remove preferred port
    pub fn remove_preferred_port(&mut self, port: &str) {
        self.hardware.preferred_ports.retain(|p| p != port);
    }
}

/// Get the configuration file path
fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("hw_ide");
    path.push("config.toml");
    path
}

/// Project-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub platform: Platform,
    pub board: String,
    pub source_files: Vec<String>,
    pub include_paths: Vec<String>,
    pub libraries: Vec<String>,
    pub build_flags: Vec<String>,
    pub upload_port: Option<String>,
    pub monitor_speed: u32,
}

impl ProjectConfig {
    pub fn new(name: String, platform: Platform) -> Self {
        Self {
            name,
            platform,
            board: "generic".to_string(),
            source_files: vec!["src/main.cpp".to_string()],
            include_paths: vec!["src".to_string()],
            libraries: Vec::new(),
            build_flags: Vec::new(),
            upload_port: None,
            monitor_speed: 115200,
        }
    }

    /// Load project configuration from file
    pub fn load(project_path: &PathBuf) -> Result<Self> {
        let config_path = project_path.join("hw_ide.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: ProjectConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Err(anyhow::anyhow!("Project configuration file not found"))
        }
    }

    /// Save project configuration to file
    pub fn save(&self, project_path: &PathBuf) -> Result<()> {
        let config_path = project_path.join("hw_ide.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
