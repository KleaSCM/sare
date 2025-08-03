/**
 * @file mod.rs
 * @brief Comprehensive shell commands module
 * 
 * This module contains all built-in shell commands organized by category.
 * Provides functionality equivalent to zsh/bash built-in commands.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Complete shell commands implementation with proper error handling,
 * modular structure, and comprehensive command coverage.
 */

pub mod filesystem;
pub mod process;
pub mod text;
pub mod system;
pub mod network;
pub mod development;

use anyhow::Result;
use crate::shell::parser::ParsedCommand;
use crate::shell::Shell;

/**
 * Command execution result
 * 
 * Represents the result of executing a built-in command.
 * Contains output text and exit status.
 */
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Command output text
    pub output: String,
    /// Exit status code (0 = success, non-zero = error)
    pub exit_code: i32,
}

/**
 * Command handler trait
 * 
 * Defines the interface for all built-in command handlers.
 * Ensures consistent error handling and result formatting.
 */
pub trait CommandHandler {
    /**
     * Executes the command
     * 
     * @param command - Parsed command with arguments
     * @param shell - Shell instance for state access
     * @return Result<CommandResult> - Command result or error
     */
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult>;
    
    /**
     * Gets command help text
     * 
     * @return &str - Help text for the command
     */
    fn help(&self) -> &str;
    
    /**
     * Gets command name
     * 
     * @return &str - Command name
     */
    fn name(&self) -> &str;
}

/**
 * Command registry that manages all built-in commands
 * 
 * Provides centralized command registration, lookup, and execution.
 * Follows modular design principles with clear separation of concerns.
 */
pub struct CommandRegistry {
    /// Map of command names to their handlers
    commands: std::collections::HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    /**
     * Creates a new command registry
     * 
     * Initializes the registry with all built-in commands.
     * 
     * @return CommandRegistry - New registry instance
     */
    pub fn new() -> Self {
        let mut registry = Self {
            commands: std::collections::HashMap::new(),
        };
        
        registry.register_commands();
        registry
    }
    
    /**
     * Registers all built-in commands
     * 
     * Adds all available commands to the registry.
     * Organized by category for maintainability.
     */
    fn register_commands(&mut self) {
        // Filesystem commands
        self.register(Box::new(filesystem::CdCommand));
        self.register(Box::new(filesystem::PwdCommand));
        self.register(Box::new(filesystem::LsCommand));
        self.register(Box::new(filesystem::MkdirCommand));
        self.register(Box::new(filesystem::RmCommand));
        self.register(Box::new(filesystem::CpCommand));
        self.register(Box::new(filesystem::MvCommand));
        self.register(Box::new(filesystem::TouchCommand));
        
        // Process commands
        self.register(Box::new(process::JobsCommand));
        self.register(Box::new(process::KillCommand));
        self.register(Box::new(process::BgCommand));
        self.register(Box::new(process::FgCommand));
        self.register(Box::new(process::WaitCommand));
        
        // Text processing commands
        self.register(Box::new(text::EchoCommand));
        self.register(Box::new(text::CatCommand));
        self.register(Box::new(text::GrepCommand));
        self.register(Box::new(text::SedCommand));
        self.register(Box::new(text::AwkCommand));
        self.register(Box::new(text::SortCommand));
        self.register(Box::new(text::UniqCommand));
        self.register(Box::new(text::WcCommand));
        
        // System commands
        self.register(Box::new(system::ExitCommand));
        self.register(Box::new(system::ClearCommand));
        self.register(Box::new(system::HistoryCommand));
        self.register(Box::new(system::HelpCommand));
        self.register(Box::new(system::AliasCommand));
        self.register(Box::new(system::UnaliasCommand));
        self.register(Box::new(system::ExportCommand));
        self.register(Box::new(system::UnsetCommand));
        self.register(Box::new(system::EnvCommand));
        self.register(Box::new(system::SourceCommand));
        
        // Network commands
        self.register(Box::new(network::PingCommand));
        self.register(Box::new(network::CurlCommand));
        self.register(Box::new(network::WgetCommand));
        self.register(Box::new(network::NetstatCommand));
        
        // Development commands
        self.register(Box::new(development::GitCommand));
        self.register(Box::new(development::CargoCommand));
        self.register(Box::new(development::MakeCommand));
        self.register(Box::new(development::NpmCommand));
    }
    
    /**
     * Registers a command handler
     * 
     * @param handler - Command handler to register
     */
    fn register(&mut self, handler: Box<dyn CommandHandler>) {
        self.commands.insert(handler.name().to_string(), handler);
    }
    
    /**
     * Executes a command by name
     * 
     * @param command - Parsed command to execute
     * @param shell - Shell instance
     * @return Result<CommandResult> - Command result or error
     */
    pub fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if let Some(handler) = self.commands.get(&command.command) {
            handler.execute(command, shell)
        } else {
            Err(anyhow::anyhow!("Unknown command: {}", command.command))
        }
    }
    
    /**
     * Gets help for a command
     * 
     * @param command_name - Name of the command
     * @return Option<&str> - Help text if command exists
     */
    pub fn get_help(&self, command_name: &str) -> Option<&str> {
        self.commands.get(command_name).map(|h| h.help())
    }
    
    /**
     * Lists all available commands
     * 
     * @return Vec<&str> - List of command names
     */
    pub fn list_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
    
    /**
     * Checks if a command exists
     * 
     * @param command_name - Name to check
     * @return bool - True if command exists
     */
    pub fn has_command(&self, command_name: &str) -> bool {
        self.commands.contains_key(command_name)
    }
} 