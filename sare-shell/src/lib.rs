/**
 * Sare Shell Library
 * 
 * This library provides the core shell functionality including
 * command parsing, execution, and shell management. Can be
 * used as a library in other applications or as a standalone shell.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: lib.rs
 * Description: Library interface for the Sare shell implementation
 */

pub mod shell;
pub mod history;
pub mod config;

pub use shell::Shell;
pub use history::HistoryManager;
pub use config::ConfigManager; 