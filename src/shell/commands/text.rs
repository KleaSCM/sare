/**
 * @file text.rs
 * @brief Text processing commands
 * 
 * This module implements text processing built-in commands
 * equivalent to standard Unix text utilities.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file text.rs
 * @description Text commands including echo, cat, grep, sed, awk, sort,
 * uniq, wc with proper error handling and text processing capabilities.
 */

use anyhow::Result;
use std::path::PathBuf;
use crate::shell::parser::ParsedCommand;
use crate::shell::Shell;
use crate::shell::commands::{CommandHandler, CommandResult};

/**
 * Echo command
 * 
 * Implements the echo command for printing text to stdout.
 * Supports various options and escape sequences.
 */
pub struct EchoCommand;

impl CommandHandler for EchoCommand {
    fn execute(&self, command: &ParsedCommand, _shell: &mut Shell) -> Result<CommandResult> {
        let no_newline = command.args.iter().any(|arg| arg == "-n" || arg == "--no-newline");
        let interpret_escapes = command.args.iter().any(|arg| arg == "-e" || arg == "--escape");
        
        let mut output = String::new();
        
        for arg in &command.args {
            if arg.starts_with('-') && (arg == "-n" || arg == "-e" || arg == "--no-newline" || arg == "--escape") {
                continue;
            }
            
            if interpret_escapes {
                output.push_str(&interpret_escape_sequences(arg));
            } else {
                output.push_str(arg);
            }
            output.push(' ');
        }
        
        if !no_newline {
            output.push('\n');
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "echo [options] [args...] - Print arguments\n\
         Options:\n\
         -n, --no-newline  Do not output trailing newline\n\
         -e, --escape      Enable interpretation of escape sequences\n\
         -E, --no-escape   Disable interpretation of escape sequences"
    }
    
    fn name(&self) -> &str {
        "echo"
    }
}

/**
 * Cat command
 * 
 * Implements the cat command for concatenating and displaying files.
 * Supports various options for file display.
 */
pub struct CatCommand;

impl CommandHandler for CatCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let show_line_numbers = command.args.iter().any(|arg| arg == "-n" || arg == "--number");
        let show_nonprintable = command.args.iter().any(|arg| arg == "-A" || arg == "--show-all");
        
        let mut output = String::new();
        let mut line_number = 1;
        
        for file_name in &command.args {
            if file_name.starts_with('-') && (file_name == "-n" || file_name == "-A" || 
                file_name == "--number" || file_name == "--show-all") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file_name));
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            
            for line in lines {
                if show_line_numbers {
                    output.push_str(&format!("{:6}  ", line_number));
                    line_number += 1;
                }
                
                if show_nonprintable {
                    output.push_str(&show_nonprintable_chars(line));
                } else {
                    output.push_str(line);
                }
                output.push('\n');
            }
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "cat [options] [files...] - Concatenate and display files\n\
         Options:\n\
         -n, --number      Number all output lines\n\
         -A, --show-all    Show nonprintable characters\n\
         -s, --squeeze-blank  Suppress repeated empty lines"
    }
    
    fn name(&self) -> &str {
        "cat"
    }
}

/**
 * Grep command
 * 
 * Implements the grep command for searching patterns in text.
 * Supports various search options and patterns.
 */
pub struct GrepCommand;

impl CommandHandler for GrepCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: grep <pattern> [files...]"));
        }
        
        let case_insensitive = command.args.iter().any(|arg| arg == "-i" || arg == "--ignore-case");
        let invert_match = command.args.iter().any(|arg| arg == "-v" || arg == "--invert-match");
        let show_line_numbers = command.args.iter().any(|arg| arg == "-n" || arg == "--line-number");
        
        let pattern = &command.args[0];
        let files = &command.args[1..];
        
        let mut output = String::new();
        let mut total_matches = 0;
        
        if files.is_empty() {
            return Err(anyhow::anyhow!("No files specified"));
        }
        
        for file_name in files {
            if file_name.starts_with('-') && (file_name == "-i" || file_name == "-v" || file_name == "-n" ||
                file_name == "--ignore-case" || file_name == "--invert-match" || file_name == "--line-number") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                output.push_str(&format!("grep: {}: No such file or directory\n", file_name));
                continue;
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            
            for (line_num, line) in lines.iter().enumerate() {
                let matches_pattern = if case_insensitive {
                    line.to_lowercase().contains(&pattern.to_lowercase())
                } else {
                    line.contains(pattern)
                };
                
                let should_output = if invert_match {
                    !matches_pattern
                } else {
                    matches_pattern
                };
                
                if should_output {
                    if show_line_numbers {
                        output.push_str(&format!("{}:{}: {}\n", file_name, line_num + 1, line));
                    } else {
                        output.push_str(&format!("{}: {}\n", file_name, line));
                    }
                    total_matches += 1;
                }
            }
        }
        
        if total_matches == 0 {
            return Ok(CommandResult {
                output: "No matches found".to_string(),
                exit_code: 1,
            });
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "grep [options] <pattern> [files...] - Search for patterns\n\
         Options:\n\
         -i, --ignore-case     Ignore case distinctions\n\
         -v, --invert-match    Select non-matching lines\n\
         -n, --line-number     Prefix each line with line number"
    }
    
    fn name(&self) -> &str {
        "grep"
    }
}

/**
 * Sed command
 * 
 * Implements the sed command for stream editing.
 * Supports basic text substitution and editing.
 */
pub struct SedCommand;

impl CommandHandler for SedCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: sed <script> [files...]"));
        }
        
        let script = &command.args[0];
        let files = &command.args[1..];
        
        if !script.contains('s') {
            return Err(anyhow::anyhow!("Unsupported sed script: {}", script));
        }
        
        let mut output = String::new();
        
        if files.is_empty() {
            return Err(anyhow::anyhow!("No files specified"));
        }
        
        for file_name in files {
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                output.push_str(&format!("sed: {}: No such file or directory\n", file_name));
                continue;
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            
            for line in lines {
                let processed_line = apply_sed_script(script, line);
                output.push_str(&processed_line);
                output.push('\n');
            }
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "sed <script> [files...] - Stream editor\n\
         Usage: sed 's/old/new/g' file.txt\n\
         Basic substitution: s/pattern/replacement/flags"
    }
    
    fn name(&self) -> &str {
        "sed"
    }
}

/**
 * Awk command
 * 
 * Implements the awk command for pattern scanning and processing.
 * Supports basic field processing and pattern matching.
 */
pub struct AwkCommand;

impl CommandHandler for AwkCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: awk <script> [files...]"));
        }
        
        let script = &command.args[0];
        let files = &command.args[1..];
        
        let mut output = String::new();
        
        if files.is_empty() {
            return Err(anyhow::anyhow!("No files specified"));
        }
        
        for file_name in files {
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                output.push_str(&format!("awk: {}: No such file or directory\n", file_name));
                continue;
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            
            for line in lines {
                let processed_line = apply_awk_script(script, line);
                if !processed_line.is_empty() {
                    output.push_str(&processed_line);
                    output.push('\n');
                }
            }
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "awk <script> [files...] - Pattern scanning and processing\n\
         Usage: awk '{print $1}' file.txt\n\
         Basic field processing and pattern matching"
    }
    
    fn name(&self) -> &str {
        "awk"
    }
}

/**
 * Sort command
 * 
 * Implements the sort command for sorting lines of text.
 * Supports various sorting options and comparisons.
 */
pub struct SortCommand;

impl CommandHandler for SortCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let reverse = command.args.iter().any(|arg| arg == "-r" || arg == "--reverse");
        let numeric = command.args.iter().any(|arg| arg == "-n" || arg == "--numeric");
        let unique = command.args.iter().any(|arg| arg == "-u" || arg == "--unique");
        
        let mut output = String::new();
        let mut all_lines: Vec<String> = Vec::new();
        
        for file_name in &command.args {
            if file_name.starts_with('-') && (file_name == "-r" || file_name == "-n" || file_name == "-u" ||
                file_name == "--reverse" || file_name == "--numeric" || file_name == "--unique") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file_name));
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            all_lines.extend(lines);
        }
        
        if unique {
            all_lines.sort();
            all_lines.dedup();
        } else {
            if numeric {
                all_lines.sort_by(|a, b| {
                    let a_num = a.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
                    let b_num = b.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
                    a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal)
                });
            } else {
                all_lines.sort();
            }
        }
        
        if reverse {
            all_lines.reverse();
        }
        
        for line in all_lines {
            output.push_str(&line);
            output.push('\n');
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "sort [options] [files...] - Sort lines of text\n\
         Options:\n\
         -r, --reverse    Reverse sort order\n\
         -n, --numeric    Sort numerically\n\
         -u, --unique     Remove duplicate lines"
    }
    
    fn name(&self) -> &str {
        "sort"
    }
}

/**
 * Uniq command
 * 
 * Implements the uniq command for removing duplicate lines.
 * Supports various options for duplicate detection.
 */
pub struct UniqCommand;

impl CommandHandler for UniqCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let count = command.args.iter().any(|arg| arg == "-c" || arg == "--count");
        let show_duplicates = command.args.iter().any(|arg| arg == "-d" || arg == "--repeated");
        
        let mut output = String::new();
        let mut lines: Vec<String> = Vec::new();
        
        for file_name in &command.args {
            if file_name.starts_with('-') && (file_name == "-c" || file_name == "-d" ||
                file_name == "--count" || file_name == "--repeated") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file_name));
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let file_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            lines.extend(file_lines);
        }
        
        if lines.is_empty() {
            return Ok(CommandResult {
                output: String::new(),
                exit_code: 0,
            });
        }
        
        let mut current_line = &lines[0];
        let mut current_count = 1;
        
        for line in &lines[1..] {
            if line == current_line {
                current_count += 1;
            } else {
                if !show_duplicates || current_count > 1 {
                    if count {
                        output.push_str(&format!("{:4} {}\n", current_count, current_line));
                    } else {
                        output.push_str(&format!("{}\n", current_line));
                    }
                }
                current_line = line;
                current_count = 1;
            }
        }
        
        if !show_duplicates || current_count > 1 {
            if count {
                output.push_str(&format!("{:4} {}\n", current_count, current_line));
            } else {
                output.push_str(&format!("{}\n", current_line));
            }
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "uniq [options] [files...] - Remove duplicate lines\n\
         Options:\n\
         -c, --count      Prefix lines with count\n\
         -d, --repeated   Only output duplicate lines"
    }
    
    fn name(&self) -> &str {
        "uniq"
    }
}

/**
 * Word count command
 * 
 * Implements the wc command for counting words, lines, and characters.
 * Supports various counting options.
 */
pub struct WcCommand;

impl CommandHandler for WcCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let count_lines = command.args.iter().any(|arg| arg == "-l" || arg == "--lines");
        let count_words = command.args.iter().any(|arg| arg == "-w" || arg == "--words");
        let count_chars = command.args.iter().any(|arg| arg == "-c" || arg == "--chars");
        
        let mut output = String::new();
        let mut total_lines = 0;
        let mut total_words = 0;
        let mut total_chars = 0;
        
        for file_name in &command.args {
            if file_name.starts_with('-') && (file_name == "-l" || file_name == "-w" || file_name == "-c" ||
                file_name == "--lines" || file_name == "--words" || file_name == "--chars") {
                continue;
            }
            
            let file_path = if PathBuf::from(file_name).is_absolute() {
                PathBuf::from(file_name)
            } else {
                shell.current_path().join(file_name)
            };
            
            if !file_path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file_name));
            }
            
            let content = std::fs::read_to_string(&file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            
            let file_lines = lines.len();
            let file_words = content.split_whitespace().count();
            let file_chars = content.len();
            
            total_lines += file_lines;
            total_words += file_words;
            total_chars += file_chars;
            
            if count_lines {
                output.push_str(&format!("{:6} ", file_lines));
            }
            if count_words {
                output.push_str(&format!("{:6} ", file_words));
            }
            if count_chars {
                output.push_str(&format!("{:6} ", file_chars));
            }
            if !count_lines && !count_words && !count_chars {
                output.push_str(&format!("{:6} {:6} {:6} ", file_lines, file_words, file_chars));
            }
            output.push_str(&format!("{}\n", file_name));
        }
        
        if command.args.len() > 1 {
            if count_lines {
                output.push_str(&format!("{:6} ", total_lines));
            }
            if count_words {
                output.push_str(&format!("{:6} ", total_words));
            }
            if count_chars {
                output.push_str(&format!("{:6} ", total_chars));
            }
            if !count_lines && !count_words && !count_chars {
                output.push_str(&format!("{:6} {:6} {:6} ", total_lines, total_words, total_chars));
            }
            output.push_str("total\n");
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "wc [options] [files...] - Word count\n\
         Options:\n\
         -l, --lines      Count lines\n\
         -w, --words      Count words\n\
         -c, --chars      Count characters"
    }
    
    fn name(&self) -> &str {
        "wc"
    }
}

/**
 * Helper function to interpret escape sequences
 * 
 * @param text - Text with escape sequences
 * @return String - Text with interpreted escapes
 */
fn interpret_escape_sequences(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => {
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/**
 * Helper function to show nonprintable characters
 * 
 * @param text - Text to process
 * @return String - Text with nonprintable characters shown
 */
fn show_nonprintable_chars(text: &str) -> String {
    let mut result = String::new();
    
    for ch in text.chars() {
        match ch {
            '\t' => result.push_str("^I"),
            '\r' => result.push_str("^M"),
            '\n' => result.push_str("$\n"),
            ch if ch.is_control() => result.push_str(&format!("^{}", (ch as u8 + 64) as char)),
            ch => result.push(ch),
        }
    }
    
    result
}

/**
 * Helper function to apply sed script
 * 
 * @param script - Sed script to apply
 * @param line - Line to process
 * @return String - Processed line
 */
fn apply_sed_script(script: &str, line: &str) -> String {
    if script.starts_with("s/") {
        let parts: Vec<&str> = script.split('/').collect();
        if parts.len() >= 3 {
            let pattern = parts[1];
            let replacement = parts[2];
            let flags = if parts.len() > 3 { parts[3] } else { "" };
            
            let mut result = line.to_string();
            if flags.contains('g') {
                result = result.replace(pattern, replacement);
            } else {
                if let Some(pos) = result.find(pattern) {
                    result.replace_range(pos..pos + pattern.len(), replacement);
                }
            }
            return result;
        }
    }
    
    line.to_string()
}

/**
 * Helper function to apply awk script
 * 
 * @param script - Awk script to apply
 * @param line - Line to process
 * @return String - Processed line
 */
fn apply_awk_script(script: &str, line: &str) -> String {
    if script.contains("print $1") {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if !fields.is_empty() {
            return fields[0].to_string();
        }
    } else if script.contains("print $0") {
        return line.to_string();
    }
    
    line.to_string()
} 