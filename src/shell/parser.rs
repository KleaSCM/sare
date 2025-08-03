/**
 * @file parser.rs
 * @brief Command parsing functionality
 * 
 * This module handles parsing of shell commands, including argument
 * splitting, quoting, and escape sequence handling.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file parser.rs
 * @description Command parser that handles shell command parsing with support
 * for arguments, flags, quoting, and escape sequences.
 */

use anyhow::Result;
use std::collections::VecDeque;

/**
 * Represents a parsed command with its arguments
 * 
 * Contains the command name and a vector of arguments
 * that will be passed to the execution system.
 */
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// The command name to execute
    pub command: String,
    /// Vector of command arguments
    pub args: Vec<String>,
    /// Whether the command should run in background
    pub background: bool,
    /// Input redirection file
    pub input_redirect: Option<String>,
    /// Output redirection file
    pub output_redirect: Option<String>,
    /// Append redirection file
    pub append_redirect: Option<String>,
}

/**
 * Command parser that handles shell command parsing
 * 
 * Provides functionality to parse command strings into
 * structured command objects for execution.
 */
pub struct CommandParser {
    /// Current parsing state
    state: ParseState,
}

/**
 * Internal parsing state for the command parser
 */
#[derive(Debug, Clone)]
enum ParseState {
    /// Parsing command name
    Command,
    /// Parsing arguments
    Arguments,
    /// Inside quoted string
    Quoted(char),
    /// Escaping next character
    Escaping,
}

impl CommandParser {
    /**
     * Creates a new command parser instance
     * 
     * @return CommandParser - New parser instance
     */
    pub fn new() -> Self {
        Self {
            state: ParseState::Command,
        }
    }
    
    /**
     * コマンド文字列を構造化されたコマンドオブジェクトにパースする関数です (◡‿◡)
     * 
     * この関数は複雑な文字列解析を行います。
     * 引数の分割、クォーティング、エスケープ、リダイレクションを処理します。
     * シングルクォート、ダブルクォート、エスケープシーケンス、
     * バックグラウンド実行の'&'をサポートしています (｡◕‿◕｡)
     * 
     * @param input - パースする生のコマンド文字列
     * @return Result<ParsedCommand> - パースされたコマンドまたはエラー
     */
    pub fn parse(&self, input: &str) -> Result<ParsedCommand> {
        // Expand environment variables first
        let expanded_input = self.expand_environment_variables(input);
        
        let mut tokens = VecDeque::new();
        let mut current_token = String::new();
        let mut state = ParseState::Command;
        let mut background = false;
        let mut input_redirect = None;
        let mut output_redirect = None;
        let mut append_redirect: Option<String> = None;
        
        let mut chars = expanded_input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match state {
                ParseState::Command | ParseState::Arguments => {
                    match ch {
                        ' ' | '\t' => {
                            if !current_token.is_empty() {
                                tokens.push_back(current_token.clone());
                                current_token.clear();
                            }
                        }
                        '"' | '\'' => {
                            state = ParseState::Quoted(ch);
                        }
                        '\\' => {
                            state = ParseState::Escaping;
                        }
                        '&' => {
                            if current_token.is_empty() && tokens.is_empty() {
                                return Err(anyhow::anyhow!("Invalid command: empty command with &"));
                            }
                            if !current_token.is_empty() {
                                tokens.push_back(current_token.clone());
                                current_token.clear();
                            }
                            background = true;
                        }
                        '<' => {
                            if !current_token.is_empty() {
                                tokens.push_back(current_token.clone());
                                current_token.clear();
                            }
                            let mut filename = String::new();
                            while let Some(&next_ch) = chars.peek() {
                                if next_ch.is_whitespace() {
                                    break;
                                }
                                filename.push(chars.next().unwrap());
                            }
                            input_redirect = Some(filename);
                        }
                        '>' => {
                            if !current_token.is_empty() {
                                tokens.push_back(current_token.clone());
                                current_token.clear();
                            }
                            
                            // Check for append redirection (>>)
                            if let Some(&'>') = chars.peek() {
                                chars.next(); // consume second '>'
                                let mut filename = String::new();
                                while let Some(&next_ch) = chars.peek() {
                                    if next_ch.is_whitespace() {
                                        break;
                                    }
                                    filename.push(chars.next().unwrap());
                                }
                                append_redirect = Some(filename);
                            } else {
                                let mut filename = String::new();
                                while let Some(&next_ch) = chars.peek() {
                                    if next_ch.is_whitespace() {
                                        break;
                                    }
                                    filename.push(chars.next().unwrap());
                                }
                                output_redirect = Some(filename);
                            }
                        }
                        _ => {
                            current_token.push(ch);
                        }
                    }
                }
                ParseState::Quoted(quote_char) => {
                    if ch == quote_char {
                        state = ParseState::Arguments;
                    } else if ch == '\\' {
                        state = ParseState::Escaping;
                    } else {
                        current_token.push(ch);
                    }
                }
                ParseState::Escaping => {
                    current_token.push(ch);
                    state = ParseState::Arguments;
                }
            }
        }
        
        if !current_token.is_empty() {
            tokens.push_back(current_token);
        }
        
        if tokens.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        let command = tokens.pop_front().unwrap();
        let args = tokens.into_iter().collect();
        
        Ok(ParsedCommand {
            command,
            args,
            background,
            input_redirect,
            output_redirect,
            append_redirect,
        })
    }

    /**
     * Expands environment variables in a string
     * 
     * Replaces variables like $PATH, $HOME with their actual values.
     * Supports both $VAR and ${VAR} syntax.
     * 
     * @param input - String containing environment variables
     * @return String - String with variables expanded
     */
    pub fn expand_environment_variables(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' {
                if let Some(next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '{' {
                        let var_name = if *next_ch == '{' {
                            // ${VAR} syntax
                            chars.next(); // consume '{'
                            let mut var_name = String::new();
                            while let Some(c) = chars.next() {
                                if c == '}' {
                                    break;
                                }
                                var_name.push(c);
                            }
                            var_name
                        } else {
                            // $VAR syntax
                            let mut var_name = String::new();
                            while let Some(c) = chars.peek() {
                                if c.is_alphanumeric() || *c == '_' {
                                    var_name.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            var_name
                        };
                        
                        if let Ok(value) = std::env::var(&var_name) {
                            result.push_str(&value);
                        }
                    } else {
                        result.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

/**
 * コマンドパイプラインの複雑な処理です (｡◕‿◕｡)
 * 
 * この構造体は複数のコマンドを演算子で接続します。
 * パイプ、論理AND/OR、順次実行などの演算子を処理する難しい部分なので、
 * 適切なエラーハンドリングで実装しています (◕‿◕)
 */
#[derive(Debug, Clone)]
pub struct CommandPipeline {
    /// Commands in the pipeline
    pub commands: Vec<ParsedCommand>,
    /// Operators between commands
    pub operators: Vec<ChainOperator>,
}

/**
 * チェーン演算子の複雑な処理です (◕‿◕)
 * 
 * この列挙型はコマンドを接続する様々な方法を定義します。
 * パイプライン内での演算子処理が難しい部分なので、
 * 適切な型安全性で実装しています (｡◕‿◕｡)
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ChainOperator {
    /// Pipe operator (|)
    Pipe,
    /// Logical AND operator (&&)
    And,
    /// Logical OR operator (||)
    Or,
    /// Sequential operator (;)
    Sequential,
}

/**
 * パイプライン解析の複雑な処理です (｡◕‿◕｡)
 * 
 * この関数は複雑なコマンド文字列解析を行います。
 * チェーン演算子による分割と個別コマンド解析が難しい部分なので、
 * 適切なエラーハンドリングで実装しています (◕‿◕)
 * 
 * @param input - 解析するコマンド文字列
 * @return Result<CommandPipeline> - 解析されたパイプラインまたはエラー
 */
pub fn parse_pipeline(input: &str) -> Result<CommandPipeline> {
    let mut commands = Vec::new();
    let mut operators = Vec::new();
    let mut parser = CommandParser::new();
    
    // Split by chain operators
    let parts: Vec<&str> = input
        .split("|")
        .flat_map(|part| part.split("&&"))
        .flat_map(|part| part.split("||"))
        .flat_map(|part| part.split(";"))
        .collect();
    
    for (i, part) in parts.iter().enumerate() {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            let parsed = parser.parse(trimmed)?;
            commands.push(parsed);
            
            // Determine operator between this command and the next
            if i < parts.len() - 1 {
                let next_part = parts[i + 1];
                if input.contains(&format!("{}|{}", trimmed, next_part)) {
                    operators.push(ChainOperator::Pipe);
                } else if input.contains(&format!("{}&&{}", trimmed, next_part)) {
                    operators.push(ChainOperator::And);
                } else if input.contains(&format!("{}||{}", trimmed, next_part)) {
                    operators.push(ChainOperator::Or);
                } else {
                    operators.push(ChainOperator::Sequential);
                }
            }
        }
    }
    
    Ok(CommandPipeline {
        commands,
        operators,
    })
} 