/**
 * @file mod.rs
 * @brief Terminal User Interface module
 * 
 * This module handles the TUI rendering and layout management
 * for the Sare shell interface using ratatui.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description TUI module that manages the terminal interface layout,
 * rendering, and user interaction for the Sare shell.
 */

pub mod layout;
pub mod prompt;
pub mod output;
pub mod panes;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io;

/**
 * TUI manager that handles the terminal interface
 * 
 * Manages the layout, rendering, and user interaction
 * for the shell's terminal user interface.
 */
pub struct TuiManager {
    /// Current layout configuration
    layout_config: LayoutConfig,
    /// Output history for display
    output_history: Vec<String>,
    /// Maximum number of output lines to keep
    max_output_lines: usize,
}

/**
 * Layout configuration for the TUI
 */
#[derive(Debug, Clone)]
struct LayoutConfig {
    /// Top bar height
    top_bar_height: u16,
    /// Input area height
    input_height: u16,
    /// Margin around the layout
    margin: u16,
}

impl TuiManager {
    /**
     * Creates a new TUI manager instance
     * 
     * @return TuiManager - New TUI manager instance
     */
    pub fn new() -> Self {
        Self {
            layout_config: LayoutConfig {
                top_bar_height: 3,
                input_height: 3,
                margin: 1,
            },
            output_history: Vec::new(),
            max_output_lines: 1000,
        }
    }
    
    /**
     * メインシェルインターフェースをレンダリングする関数です (◡‿◡)
     * 
     * この関数は複雑なTUIレイアウト管理を行います。
     * ratatuiのライフタイム管理が難しい部分なので、
     * LineとSpanの適切な使用方法でレンダリングを実装しています (｡◕‿◕｡)
     * 
     * @param frame - レンダリング用のratatuiフレーム
     * @param current_path - 現在の作業ディレクトリ
     * @param input_buffer - 現在の入力テキスト
     * @param output_lines - 表示する出力行
     */
    pub fn render(
        &self,
        frame: &mut Frame,
        current_path: &str,
        input_buffer: &str,
        output_lines: &[String],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(self.layout_config.margin)
            .constraints(
                [
                    Constraint::Length(self.layout_config.top_bar_height),
                    Constraint::Min(0),
                    Constraint::Length(self.layout_config.input_height),
                ]
                .as_ref(),
            )
            .split(frame.size());
        
        let top_bar = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("sare@{}: {}", 
                        whoami::username(),
                        current_path
                    ),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(top_bar, chunks[0]);
        
        let output_lines_vec: Vec<Line> = output_lines
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect();
        
        let output_area = Paragraph::new(output_lines_vec)
            .block(Block::default().borders(Borders::ALL).title("Output"));
        frame.render_widget(output_area, chunks[1]);
        
        let input_area = Paragraph::new(input_buffer)
            .block(Block::default().borders(Borders::ALL).title(">>>"));
        frame.render_widget(input_area, chunks[2]);
    }
    
    /**
     * Adds output to the history
     * 
     * @param output - Output line to add
     */
    pub fn add_output(&mut self, output: String) {
        self.output_history.push(output);
        
        if self.output_history.len() > self.max_output_lines {
            self.output_history.drain(0..self.output_history.len() - self.max_output_lines);
        }
    }
    
    /**
     * Clears the output history
     */
    pub fn clear_output(&mut self) {
        self.output_history.clear();
    }
    
    /**
     * Gets the output history
     * 
     * @return &[String] - Reference to output history
     */
    pub fn get_output_history(&self) -> &[String] {
        &self.output_history
    }
    
    /**
     * Sets the maximum number of output lines to keep
     * 
     * @param max_lines - Maximum number of lines
     */
    pub fn set_max_output_lines(&mut self, max_lines: usize) {
        self.max_output_lines = max_lines;
    }
} 