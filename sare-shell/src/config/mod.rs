/**
 * @file mod.rs
 * @brief Configuration management module
 * 
 * This module handles shell configuration including themes,
 * shortcuts, and user preferences.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Configuration management for shell settings,
 * themes, and user preferences.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/**
 * Shell configuration structure
 * 
 * Contains all user-configurable settings for the shell
 * including themes, shortcuts, and preferences.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Keyboard shortcuts
    pub shortcuts: HashMap<String, String>,
    /// Shell preferences
    pub preferences: PreferencesConfig,
}

/**
 * Theme configuration
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary color
    pub primary_color: String,
    /// Secondary color
    pub secondary_color: String,
    /// Background color
    pub background_color: String,
    /// Text color
    pub text_color: String,
}

/**
 * Preferences configuration
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencesConfig {
    /// Maximum history size
    pub max_history_size: usize,
    /// Auto-completion enabled
    pub auto_completion: bool,
    /// Show line numbers
    pub show_line_numbers: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                primary_color: "#87CEEB".to_string(),
                secondary_color: "#98FB98".to_string(),
                background_color: "#1E1E1E".to_string(),
                text_color: "#FFFFFF".to_string(),
            },
            shortcuts: HashMap::new(),
            preferences: PreferencesConfig {
                max_history_size: 1000,
                auto_completion: true,
                show_line_numbers: false,
            },
        }
    }
}

/**
 * Configuration manager
 * 
 * Handles loading, saving, and managing shell configuration.
 */
pub struct ConfigManager {
    /// Current configuration
    config: ShellConfig,
    /// Configuration file path
    config_path: PathBuf,
}

impl ConfigManager {
    /**
     * Creates a new configuration manager
     * 
     * @return ConfigManager - New config manager instance
     */
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sare");
        
        fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.json");
        
        let config = if config_path.exists() {
            let config_data = fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_data)?
        } else {
            ShellConfig::default()
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    /**
     * Gets the current configuration
     * 
     * @return &ShellConfig - Current configuration
     */
    pub fn get_config(&self) -> &ShellConfig {
        &self.config
    }
    
    /**
     * Gets mutable reference to configuration
     * 
     * @return &mut ShellConfig - Mutable configuration reference
     */
    pub fn get_config_mut(&mut self) -> &mut ShellConfig {
        &mut self.config
    }
    
    /**
     * Saves the current configuration to file
     * 
     * @return Result<()> - Success or error
     */
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_data = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, config_data)?;
        Ok(())
    }
    
    /**
     * Reloads configuration from file
     * 
     * @return Result<()> - Success or error
     */
    pub fn reload_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            let config_data = fs::read_to_string(&self.config_path)?;
            self.config = serde_json::from_str(&config_data)?;
        }
        Ok(())
    }
} 