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
	 * コマンド文字列を構造化されたコマンドオブジェクトにパースする関数です
	 * 
	 * 生のコマンド文字列を解析し、コマンド名、引数、リダイレクション、
	 * バックグラウンド実行フラグを適切に分離します。
	 * 
	 * シングルクォート（'）とダブルクォート（"）による文字列リテラル、
	 * バックスラッシュ（\）によるエスケープシーケンス、入力リダイレクション（<）、
	 * 出力リダイレクション（>）、追記リダイレクション（>>）、
	 * バックグラウンド実行（&）をサポートします。
	 * 
	 * 環境変数の展開も行い、クォーティング内でも適切に処理されます。
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
 * コマンドパイプラインを表現する構造体です
 * 
 * 複数のコマンドを演算子（|、&&、||、;）で接続して
 * パイプラインとして実行できるようにします。
 * 
 * commandsフィールドにパースされたコマンドのリストを格納し、
 * operatorsフィールドにコマンド間の演算子を格納します。
 * 
 * パイプ（|）、論理AND（&&）、論理OR（||）、順次実行（;）
 * の各演算子に対応しています。
 */
#[derive(Debug, Clone)]
pub struct CommandPipeline {
    /// Commands in the pipeline
    pub commands: Vec<ParsedCommand>,
    /// Operators between commands
    pub operators: Vec<ChainOperator>,
}

/**
 * コマンドを接続する演算子を定義する列挙型です
 * 
 * パイプライン内でコマンドを接続する様々な方法を定義し、
 * 各演算子の動作を明確に表現します。
 * 
 * Pipe（|）は前のコマンドの出力を次のコマンドの入力に、
 * And（&&）は前のコマンドが成功した場合のみ次のコマンドを実行、
 * Or（||）は前のコマンドが失敗した場合のみ次のコマンドを実行、
 * Sequential（;）は順次実行を表現します。
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
 * コマンド文字列をパイプラインとして解析する関数です
 * 
 * 入力文字列をチェーン演算子（|、&&、||、;）で分割し、
 * 各部分を個別のコマンドとして解析します。
 * 
 * 分割された各部分をCommandParserで解析し、コマンドと
 * 演算子のリストを作成します。演算子の判定は文字列の
 * 包含関係をチェックして決定します。
 * 
 * 空でない部分のみをコマンドとして追加し、適切な演算子を
 * 各コマンド間に設定します。
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