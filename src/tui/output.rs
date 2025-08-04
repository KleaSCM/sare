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
 * for the shell interface with scrollable output and copy/paste support.
 */
pub struct OutputManager {
    /// Output history
    output_lines: Vec<String>,
    /// Maximum number of lines to keep
    max_lines: usize,
    /// Output style
    output_style: Style,
    /// Current scroll position
    scroll_position: usize,
    /// Selected text for copy/paste
    selected_text: Option<String>,
    /// Copy/paste buffer
    clipboard_buffer: String,
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
            scroll_position: 0,
            selected_text: None,
            clipboard_buffer: String::new(),
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
    
    /**
     * Scrolls output up
     * 
     * Scrolls the output view up by the specified number of lines.
     * 
     * @param lines - Number of lines to scroll up
     */
    pub fn scroll_up(&mut self, lines: usize) {
        if self.scroll_position > 0 {
            self.scroll_position = self.scroll_position.saturating_sub(lines);
        }
    }
    
    /**
     * Scrolls output down
     * 
     * Scrolls the output view down by the specified number of lines.
     * 
     * @param lines - Number of lines to scroll down
     */
    pub fn scroll_down(&mut self, lines: usize) {
        let max_scroll = self.output_lines.len().saturating_sub(1);
        if self.scroll_position < max_scroll {
            self.scroll_position = (self.scroll_position + lines).min(max_scroll);
        }
    }
    
    /**
     * Gets current scroll position
     * 
     * @return usize - Current scroll position
     */
    pub fn get_scroll_position(&self) -> usize {
        self.scroll_position
    }
    
    /**
     * Sets scroll position
     * 
     * @param position - New scroll position
     */
    pub fn set_scroll_position(&mut self, position: usize) {
        let max_scroll = self.output_lines.len().saturating_sub(1);
        self.scroll_position = position.min(max_scroll);
    }
    
    /**
     * Selects text for copy/paste
     * 
     * @param start_line - Starting line number
     * @param end_line - Ending line number
     * @return bool - True if selection was successful
     */
    pub fn select_text(&mut self, start_line: usize, end_line: usize) -> bool {
        if start_line < self.output_lines.len() && end_line < self.output_lines.len() && start_line <= end_line {
            let selected_lines: Vec<String> = self.output_lines[start_line..=end_line].to_vec();
            self.selected_text = Some(selected_lines.join("\n"));
            true
        } else {
            false
        }
    }
    
    /**
     * Copies selected text to clipboard
     * 
     * @return Option<String> - Selected text if available
     */
    pub fn copy_selected_text(&mut self) -> Option<String> {
        if let Some(text) = &self.selected_text {
            self.clipboard_buffer = text.clone();
            Some(text.clone())
        } else {
            None
        }
    }
    
    /**
     * Gets clipboard content
     * 
     * @return &str - Clipboard content
     */
    pub fn get_clipboard(&self) -> &str {
        &self.clipboard_buffer
    }
    
    /**
     * Sets clipboard content
     * 
     * @param text - Text to set in clipboard
     */
    pub fn set_clipboard(&mut self, text: String) {
        self.clipboard_buffer = text;
    }
    
    /**
     * Clears text selection
     */
    pub fn clear_selection(&mut self) {
        self.selected_text = None;
    }
    
    /**
     * Gets selected text
     * 
     * @return Option<&str> - Selected text if available
     */
    pub fn get_selected_text(&self) -> Option<&str> {
        self.selected_text.as_deref()
    }
    
    /**
     * Renders output with scroll position
     * 
     * @return Paragraph - Rendered output widget with scroll
     */
    pub fn render_scrollable_output(&self) -> Paragraph {
        let visible_lines: Vec<Line> = self.output_lines
            .iter()
            .skip(self.scroll_position)
            .take(50) // Show 50 lines at a time
            .map(|line| Line::from(vec![Span::styled(line, self.output_style)]))
            .collect();
        
        Paragraph::new(visible_lines)
            .block(Block::default().borders(Borders::ALL).title("Output (Scrollable)"))
            .scroll((self.scroll_position as u16, 0))
    }
} 