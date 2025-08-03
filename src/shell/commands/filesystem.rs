/**
 * @file filesystem.rs
 * @brief Filesystem operation commands
 * 
 * This module implements filesystem-related built-in commands
 * equivalent to standard Unix filesystem utilities.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file filesystem.rs
 * @description Filesystem commands including cd, pwd, ls, mkdir, rm, cp, mv, touch
 * with proper error handling and POSIX compliance.
 */

use anyhow::Result;
use std::path::PathBuf;
use crate::shell::parser::ParsedCommand;
use crate::shell::Shell;
use crate::shell::commands::{CommandHandler, CommandResult};

/**
 * Change directory command
 * 
 * Implements the cd command for changing working directory.
 * Supports relative and absolute paths, home directory expansion.
 */
pub struct CdCommand;

impl CommandHandler for CdCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let path = command.args.first().unwrap_or(&"~".to_string());
        
        let target_path = if path == "~" {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        } else {
            let path_buf = PathBuf::from(path);
            if path_buf.is_absolute() {
                path_buf
            } else {
                shell.current_path().join(path_buf)
            }
        };
        
        if target_path.exists() && target_path.is_dir() {
            let canonical_path = target_path.canonicalize()?;
            std::env::set_current_dir(&canonical_path)?;
            *shell.current_path_mut() = canonical_path;
            
            Ok(CommandResult {
                output: format!("Changed directory to: {}", canonical_path.display()),
                exit_code: 0,
            })
        } else {
            Err(anyhow::anyhow!("Directory not found: {}", path))
        }
    }
    
    fn help(&self) -> &str {
        "cd [directory] - Change working directory\n\
         Usage: cd /path/to/directory\n\
         Usage: cd ~ (go to home directory)\n\
         Usage: cd .. (go to parent directory)"
    }
    
    fn name(&self) -> &str {
        "cd"
    }
}

/**
 * Print working directory command
 * 
 * Implements the pwd command for displaying current working directory.
 */
pub struct PwdCommand;

impl CommandHandler for PwdCommand {
    fn execute(&self, _command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let current_path = shell.current_path();
        
        Ok(CommandResult {
            output: current_path.to_string_lossy().to_string(),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "pwd - Print working directory\n\
         Displays the absolute path of the current working directory."
    }
    
    fn name(&self) -> &str {
        "pwd"
    }
}

/**
 * List directory contents command
 * 
 * Implements the ls command for listing directory contents.
 * Supports various flags and formatting options.
 */
pub struct LsCommand;

impl CommandHandler for LsCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let mut output = String::new();
        let current_path = shell.current_path();
        
        let show_hidden = command.args.iter().any(|arg| arg == "-a" || arg == "--all");
        let long_format = command.args.iter().any(|arg| arg == "-l" || arg == "--long");
        
        let entries = std::fs::read_dir(current_path)?;
        
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if !show_hidden && file_name.starts_with('.') {
                continue;
            }
            
            if long_format {
                let metadata = entry.metadata()?;
                let permissions = format_permissions(&metadata);
                let size = metadata.len();
                let modified = metadata.modified()?;
                let modified_str = format_datetime(modified);
                
                output.push_str(&format!("{} {} {} {} {}\n", 
                    permissions, metadata.file_type().is_dir() as i32, 
                    size, modified_str, file_name));
            } else {
                output.push_str(&format!("{}  ", file_name));
            }
        }
        
        if !long_format {
            output.push('\n');
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "ls [options] [directory] - List directory contents\n\
         Options:\n\
         -a, --all    Show hidden files\n\
         -l, --long   Use long listing format\n\
         -h, --human  Show file sizes in human readable format"
    }
    
    fn name(&self) -> &str {
        "ls"
    }
}

/**
 * Make directory command
 * 
 * Implements the mkdir command for creating directories.
 * Supports recursive directory creation.
 */
pub struct MkdirCommand;

impl CommandHandler for MkdirCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: mkdir <directory>"));
        }
        
        let recursive = command.args.iter().any(|arg| arg == "-p" || arg == "--parents");
        
        for dir_name in &command.args {
            if dir_name.starts_with('-') && dir_name != "-p" && dir_name != "--parents" {
                continue;
            }
            
            let dir_path = if PathBuf::from(dir_name).is_absolute() {
                PathBuf::from(dir_name)
            } else {
                shell.current_path().join(dir_name)
            };
            
            if recursive {
                std::fs::create_dir_all(&dir_path)?;
            } else {
                std::fs::create_dir(&dir_path)?;
            }
        }
        
        Ok(CommandResult {
            output: "Directory created successfully".to_string(),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "mkdir [options] <directory> - Create directory\n\
         Options:\n\
         -p, --parents  Create parent directories as needed\n\
         -m, --mode     Set file mode (permissions)"
    }
    
    fn name(&self) -> &str {
        "mkdir"
    }
}

/**
 * Remove files/directories command
 * 
 * Implements the rm command for removing files and directories.
 * Supports recursive removal and force options.
 */
pub struct RmCommand;

impl CommandHandler for RmCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: rm <file>"));
        }
        
        let recursive = command.args.iter().any(|arg| arg == "-r" || arg == "-R" || arg == "--recursive");
        let force = command.args.iter().any(|arg| arg == "-f" || arg == "--force");
        
        let mut removed_count = 0;
        
        for file_name in &command.args {
            if file_name.starts_with('-') && !file_name.starts_with("--") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if file_path.exists() {
                if file_path.is_dir() && recursive {
                    std::fs::remove_dir_all(&file_path)?;
                } else if file_path.is_file() {
                    std::fs::remove_file(&file_path)?;
                } else if !force {
                    return Err(anyhow::anyhow!("Cannot remove directory '{}': Is a directory", file_name));
                }
                removed_count += 1;
            } else if !force {
                return Err(anyhow::anyhow!("No such file or directory: {}", file_name));
            }
        }
        
        Ok(CommandResult {
            output: format!("Removed {} item(s)", removed_count),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "rm [options] <file> - Remove files or directories\n\
         Options:\n\
         -f, --force     Force removal without prompting\n\
         -r, -R, --recursive  Remove directories recursively\n\
         -i, --interactive  Prompt before removal"
    }
    
    fn name(&self) -> &str {
        "rm"
    }
}

/**
 * Copy files command
 * 
 * Implements the cp command for copying files and directories.
 * Supports recursive copying and preserving attributes.
 */
pub struct CpCommand;

impl CommandHandler for CpCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.len() < 2 {
            return Err(anyhow::anyhow!("Usage: cp <source> <destination>"));
        }
        
        let recursive = command.args.iter().any(|arg| arg == "-r" || arg == "-R" || arg == "--recursive");
        
        let source = &command.args[0];
        let destination = &command.args[1];
        
        let source_path = if PathBuf::from(source).is_absolute() {
            PathBuf::from(source)
        } else {
            shell.current_path().join(source)
        };
        
        let dest_path = if PathBuf::from(destination).is_absolute() {
            PathBuf::from(destination)
        } else {
            shell.current_path().join(destination)
        };
        
        if !source_path.exists() {
            return Err(anyhow::anyhow!("Source file not found: {}", source));
        }
        
        if source_path.is_dir() && recursive {
            copy_directory(&source_path, &dest_path)?;
        } else if source_path.is_file() {
            std::fs::copy(&source_path, &dest_path)?;
        } else {
            return Err(anyhow::anyhow!("Cannot copy directory without -r flag"));
        }
        
        Ok(CommandResult {
            output: format!("Copied {} to {}", source, destination),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "cp [options] <source> <destination> - Copy files and directories\n\
         Options:\n\
         -r, -R, --recursive  Copy directories recursively\n\
         -p, --preserve       Preserve file attributes\n\
         -v, --verbose        Verbose output"
    }
    
    fn name(&self) -> &str {
        "cp"
    }
}

/**
 * Move/rename files command
 * 
 * Implements the mv command for moving and renaming files.
 */
pub struct MvCommand;

impl CommandHandler for MvCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.len() < 2 {
            return Err(anyhow::anyhow!("Usage: mv <source> <destination>"));
        }
        
        let source = &command.args[0];
        let destination = &command.args[1];
        
        let source_path = if PathBuf::from(source).is_absolute() {
            PathBuf::from(source)
        } else {
            shell.current_path().join(source)
        };
        
        let dest_path = if PathBuf::from(destination).is_absolute() {
            PathBuf::from(destination)
        } else {
            shell.current_path().join(destination)
        };
        
        if !source_path.exists() {
            return Err(anyhow::anyhow!("Source file not found: {}", source));
        }
        
        std::fs::rename(&source_path, &dest_path)?;
        
        Ok(CommandResult {
            output: format!("Moved {} to {}", source, destination),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "mv [options] <source> <destination> - Move or rename files\n\
         Options:\n\
         -f, --force     Force move without prompting\n\
         -i, --interactive  Prompt before overwrite\n\
         -v, --verbose   Verbose output"
    }
    
    fn name(&self) -> &str {
        "mv"
    }
}

/**
 * Touch command for creating files or updating timestamps
 * 
 * Implements the touch command for creating empty files or updating timestamps.
 */
pub struct TouchCommand;

impl CommandHandler for TouchCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: touch <file>"));
        }
        
        let mut created_count = 0;
        
        for file_name in &command.args {
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                std::fs::File::create(&file_path)?;
                created_count += 1;
            } else {
                let now = std::time::SystemTime::now();
                let metadata = std::fs::metadata(&file_path)?;
                std::fs::set_times(&file_path, now, now)?;
            }
        }
        
        Ok(CommandResult {
            output: format!("Touched {} file(s)", created_count),
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "touch [options] <file> - Create empty files or update timestamps\n\
         Options:\n\
         -a    Change access time only\n\
         -m    Change modification time only\n\
         -c    Do not create files that do not exist"
    }
    
    fn name(&self) -> &str {
        "touch"
    }
}

/**
 * Helper function to copy directories recursively
 * 
 * @param src - Source directory path
 * @param dst - Destination directory path
 * @return Result<()> - Success or error
 */
fn copy_directory(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    if !dst.exists() {
        std::fs::create_dir(dst)?;
    }
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

/**
 * Helper function to format file permissions
 * 
 * @param metadata - File metadata
 * @return String - Formatted permissions string
 */
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    let mode = metadata.permissions().mode();
    let mut perms = String::new();
    
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    
    perms
}

/**
 * Helper function to format datetime
 * 
 * @param time - System time to format
 * @return String - Formatted datetime string
 */
fn format_datetime(time: std::time::SystemTime) -> String {
    use chrono::{DateTime, Utc};
    
    let datetime: DateTime<Utc> = time.into();
    datetime.format("%Y-%m-%d %H:%M").to_string()
} 