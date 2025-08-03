/**
 * @file mod.rs
 * @brief Command history management module
 * 
 * This module handles command history persistence and retrieval
 * for the shell interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Command history management for persistent
 * command storage and retrieval.
 */

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/**
 * Command history entry
 * 
 * Represents a single command in the history
 * with timestamp and execution status.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    /// Command text
    pub command: String,
    /// Timestamp when executed
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Exit code
    pub exit_code: Option<i32>,
}

/**
 * History manager for command history
 * 
 * Handles loading, saving, and managing command history
 * with persistence to disk.
 */
pub struct HistoryManager {
    /// Command history entries
    history: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// History file path
    history_path: PathBuf,
}

impl HistoryManager {
    /**
     * Creates a new history manager
     * 
     * @return HistoryManager - New history manager instance
     */
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let history_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sare");
        
        fs::create_dir_all(&history_dir)?;
        
        let history_path = history_dir.join("history.json");
        
        let history = if history_path.exists() {
            let history_data = fs::read_to_string(&history_path)?;
            serde_json::from_str(&history_data)?
        } else {
            VecDeque::new()
        };
        
        Ok(Self {
            history,
            max_entries: 1000,
            history_path,
        })
    }
    
    /**
     * Adds a command to history
     * 
     * @param command - Command to add
     * @param exit_code - Exit code of the command
     */
    pub fn add_command(&mut self, command: String, exit_code: Option<i32>) {
        let entry = HistoryEntry {
            command,
            timestamp: chrono::Utc::now(),
            exit_code,
        };
        
        self.history.push_back(entry);
        
        while self.history.len() > self.max_entries {
            self.history.pop_front();
        }
    }
    
    /**
     * Gets all history entries
     * 
     * @return &VecDeque<HistoryEntry> - All history entries
     */
    pub fn get_history(&self) -> &VecDeque<HistoryEntry> {
        &self.history
    }
    
    /**
     * Gets recent history entries
     * 
     * @param count - Number of recent entries to get
     * @return Vec<&HistoryEntry> - Recent history entries
     */
    pub fn get_recent_history(&self, count: usize) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /**
     * Searches history for commands matching a pattern
     * 
     * @param pattern - Search pattern
     * @return Vec<&HistoryEntry> - Matching history entries
     */
    pub fn search_history(&self, pattern: &str) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| entry.command.contains(pattern))
            .collect()
    }
    
    /**
     * Clears all history
     */
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /**
     * Saves history to file
     * 
     * @return Result<()> - Success or error
     */
    pub fn save_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let history_data = serde_json::to_string_pretty(&self.history)?;
        fs::write(&self.history_path, history_data)?;
        Ok(())
    }
    
    /**
     * Sets the maximum number of entries to keep
     * 
     * @param max_entries - Maximum number of entries
     */
    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
        
        while self.history.len() > self.max_entries {
            self.history.pop_front();
        }
    }
} 