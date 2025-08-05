/**
 * @file mod.rs
 * @brief Command history management
 * 
 * This module handles persistent command history storage
 * and retrieval for the shell.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: History management with file-based persistence
 * for command history across shell sessions.
 */

pub mod navigation;
pub mod completion;

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

pub use navigation::{HistoryNavigator, HistoryNavigationState};
pub use completion::{TabCompleter, CompletionResult, CompletionContext};

/**
 * Represents a single history entry
 * 
 * Contains command information, timestamp, and exit status.
 */
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The command that was executed
    pub command: String,
    /// When the command was executed
    pub timestamp: DateTime<Utc>,
    /// Exit code of the command (if available)
    pub exit_code: Option<i32>,
}

/**
 * History manager that handles command history
 * 
 * Provides functionality to store, retrieve, and persist
 * command history across shell sessions.
 */
#[derive(Debug)]
pub struct HistoryManager {
    /// In-memory history storage
    pub history: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep
    pub max_entries: usize,
    /// Path to history file
    pub history_file: PathBuf,
}

impl HistoryManager {
    /**
     * Creates a new history manager instance
     * 
     * @return Result<HistoryManager> - New history manager or error
     */
    pub fn new() -> Result<Self> {
        Self::with_config(1000, PathBuf::from(".sare_history"))
    }
    
    /**
     * Creates a history manager with custom configuration
     * 
     * @param max_entries - Maximum number of entries to keep
     * @param history_file - Path to history file
     * @return Result<HistoryManager> - New history manager or error
     */
    pub fn with_config(max_entries: usize, history_file: PathBuf) -> Result<Self> {
        let history_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".sare_history");
        
        let mut manager = Self {
            history: VecDeque::new(),
            max_entries: 1000,
            history_file,
        };
        
        manager.load_history()?;
        
        Ok(manager)
    }
    
    /**
     * 履歴ファイルからコマンド履歴を読み込む関数です
     * 
     * 履歴ファイルを解析して、タイムスタンプ、コマンド、終了コードを
     * 含む履歴エントリを復元します。ファイルが存在しない場合は
     * 空の履歴で開始します。
     * 
     * 履歴エントリは最大数に制限され、古いエントリは自動的に削除されます
     * 
     * @return Result<()> - Success or error
     */
    pub fn load_history(&mut self) -> Result<()> {
        if !self.history_file.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.history_file)?;
        
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 2 {
                let timestamp = parts[0].parse::<i64>()
                    .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()))
                    .unwrap_or_else(|_| Utc::now());
                
                let command = parts[1].to_string();
                let exit_code = parts.get(2).and_then(|s| s.parse::<i32>().ok());
                
                self.history.push_back(HistoryEntry {
                    command,
                    timestamp,
                    exit_code,
                });
            }
        }
        
        while self.history.len() > self.max_entries {
            self.history.pop_front();
        }
        
        Ok(())
    }
    
    /**
     * Saves history to file
     * 
     * @return Result<()> - Success or error
     */
    pub fn save_history(&self) -> Result<()> {
        let mut content = String::new();
        
        for entry in &self.history {
            content.push_str(&format!("{}|{}|{}\n",
                entry.timestamp.timestamp(),
                entry.command,
                entry.exit_code.unwrap_or(-1)
            ));
        }
        
        fs::write(&self.history_file, content)?;
        Ok(())
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
            timestamp: Utc::now(),
            exit_code,
        };
        
        self.history.push_back(entry);
        
        while self.history.len() > self.max_entries {
            self.history.pop_front();
        }
        
        if let Err(e) = self.save_history() {
            eprintln!("Failed to save history: {}", e);
        }
    }
    
    /**
     * Gets all history entries
     * 
     * @return Vec<&HistoryEntry> - List of history entries
     */
    pub fn get_history(&self) -> Vec<&HistoryEntry> {
        self.history.iter().collect()
    }
    
    /**
     * Clears all history
     */
    pub fn clear_history(&mut self) -> Result<()> {
        self.history.clear();
        self.save_history()
    }
    
    /**
     * Searches history for commands matching a pattern
     * 
     * @param pattern - Pattern to search for
     * @return Vec<&HistoryEntry> - Matching history entries
     */
    pub fn search_history(&self, pattern: &str) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| entry.command.contains(pattern))
            .collect()
    }
} 