/**
 * @file prompt.rs
 * @brief TUI prompt and input handling
 * 
 * This module handles prompt rendering and input processing
 * for the terminal user interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file prompt.rs
 * @description Prompt handling for user input in the TUI interface.
 */

use ratatui::{
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
};

/**
 * Prompt manager for the TUI
 * 
 * Handles prompt rendering and input processing
 * for the shell interface.
 */
pub struct PromptManager {
    /// Current prompt text
    prompt_text: String,
    /// Prompt style
    prompt_style: Style,
}

impl PromptManager {
    /**
     * Creates a new prompt manager
     * 
     * @return PromptManager - New prompt manager instance
     */
    pub fn new() -> Self {
        Self {
            prompt_text: ">>> ".to_string(),
            prompt_style: Style::default().fg(Color::Green),
        }
    }
    
    /**
     * Renders the prompt widget
     * 
     * @param input_text - Current input text
     * @return Paragraph - Rendered prompt widget
     */
    pub fn render_prompt(&self, input_text: &str) -> Paragraph {
        let prompt_text = self.prompt_text.clone();
        let input_text = input_text.to_string();
        let lines = vec![
            Line::from(vec![
                Span::styled(prompt_text, self.prompt_style),
                Span::raw(input_text),
            ]),
        ];
        
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Input"))
    }
    
    /**
     * Sets the prompt text
     * 
     * @param text - New prompt text
     */
    pub fn set_prompt_text(&mut self, text: String) {
        self.prompt_text = text;
    }
    
    /**
     * Sets the prompt style
     * 
     * @param style - New prompt style
     */
    pub fn set_prompt_style(&mut self, style: Style) {
        self.prompt_style = style;
    }
} 