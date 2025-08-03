/**
 * @file output.rs
 * @brief TUI output handling
 * 
 * This module handles output rendering and management
 * for the terminal user interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file output.rs
 * @description Output handling for displaying command results in the TUI interface.
 */

use ratatui::{
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
};

/**
 * Output manager for the TUI
 * 
 * Handles output rendering and management
 * for the shell interface.
 */
pub struct OutputManager {
    /// Output history
    output_lines: Vec<String>,
    /// Maximum number of lines to keep
    max_lines: usize,
    /// Output style
    output_style: Style,
}

impl OutputManager {
    /**
     * Creates a new output manager
     * 
     * @return OutputManager - New output manager instance
     */
    pub fn new() -> Self {
        Self {
            output_lines: Vec::new(),
            max_lines: 1000,
            output_style: Style::default().fg(Color::White),
        }
    }
    
    /**
     * Renders the output widget
     * 
     * @return Paragraph - Rendered output widget
     */
    pub fn render_output(&self) -> Paragraph {
        let lines: Vec<Line> = self.output_lines
            .iter()
            .map(|line| Line::from(vec![Span::styled(line, self.output_style)]))
            .collect();
        
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Output"))
    }
    
    /**
     * Adds output line
     * 
     * @param line - Output line to add
     */
    pub fn add_output(&mut self, line: String) {
        self.output_lines.push(line);
        
        if self.output_lines.len() > self.max_lines {
            self.output_lines.drain(0..self.output_lines.len() - self.max_lines);
        }
    }
    
    /**
     * Clears all output
     */
    pub fn clear_output(&mut self) {
        self.output_lines.clear();
    }
    
    /**
     * Gets all output lines
     * 
     * @return &[String] - Reference to output lines
     */
    pub fn get_output_lines(&self) -> &[String] {
        &self.output_lines
    }
    
    /**
     * Sets the maximum number of lines to keep
     * 
     * @param max_lines - Maximum number of lines
     */
    pub fn set_max_lines(&mut self, max_lines: usize) {
        self.max_lines = max_lines;
    }
} 