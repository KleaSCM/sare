/**
 * @file panes.rs
 * @brief TUI pane management
 * 
 * This module handles pane splitting and management
 * for the terminal user interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file panes.rs
 * @description Pane management for split terminal views in the TUI interface.
 */

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
};

/**
 * Pane manager for the TUI
 * 
 * Handles pane splitting and management
 * for the shell interface.
 */
pub struct PaneManager {
    /// Current panes
    panes: Vec<Pane>,
    /// Active pane index
    active_pane: usize,
}

/**
 * Represents a single pane in the terminal
 */
#[derive(Debug, Clone)]
pub struct Pane {
    /// Pane ID
    pub id: usize,
    /// Pane content
    pub content: String,
    /// Pane title
    pub title: String,
    /// Whether pane is active
    pub active: bool,
}

impl PaneManager {
    /**
     * Creates a new pane manager
     * 
     * @return PaneManager - New pane manager instance
     */
    pub fn new() -> Self {
        Self {
            panes: vec![Pane {
                id: 0,
                content: String::new(),
                title: "Main".to_string(),
                active: true,
            }],
            active_pane: 0,
        }
    }
    
    /**
     * Splits the current pane
     * 
     * @param direction - Split direction
     * @return Vec<Rect> - New pane areas
     */
    pub fn split_pane(&mut self, direction: Direction, area: Rect) -> Vec<Rect> {
        let num_panes = self.panes.len() + 1;
        let constraints: Vec<Constraint> = (0..num_panes)
            .map(|_| Constraint::Ratio(1, num_panes as u32))
            .collect();
        
        let new_pane = Pane {
            id: self.panes.len(),
            content: String::new(),
            title: format!("Pane {}", self.panes.len()),
            active: false,
        };
        
        self.panes.push(new_pane);
        
        Layout::default()
            .direction(direction)
            .constraints(&constraints)
            .split(area)
            .to_vec()
    }
    
    /**
     * Sets the active pane
     * 
     * @param pane_id - Pane ID to activate
     */
    pub fn set_active_pane(&mut self, pane_id: usize) {
        if pane_id < self.panes.len() {
            for pane in &mut self.panes {
                pane.active = false;
            }
            self.panes[pane_id].active = true;
            self.active_pane = pane_id;
        }
    }
    
    /**
     * Gets the active pane
     * 
     * @return Option<&Pane> - Active pane if exists
     */
    pub fn get_active_pane(&self) -> Option<&Pane> {
        self.panes.get(self.active_pane)
    }
    
    /**
     * Gets all panes
     * 
     * @return &[Pane] - All panes
     */
    pub fn get_panes(&self) -> &[Pane] {
        &self.panes
    }
    
    /**
     * Renders a pane widget
     * 
     * @param pane - Pane to render
     * @return Paragraph - Rendered pane widget
     */
    pub fn render_pane(&self, pane: &Pane) -> Paragraph {
        let style = if pane.active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        
        let content = pane.content.clone();
        let line = Line::from(vec![Span::styled(content, style)]);
        
        Paragraph::new(vec![line]).block(Block::default().borders(Borders::ALL))
    }
} 