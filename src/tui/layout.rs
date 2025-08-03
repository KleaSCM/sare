/**
 * @file layout.rs
 * @brief TUI layout management
 * 
 * This module handles layout calculations and management
 * for the terminal user interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file layout.rs
 * @description Layout management for the TUI interface.
 */

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/**
 * Layout manager for the TUI
 * 
 * Handles layout calculations and constraints for
 * the terminal user interface.
 */
pub struct LayoutManager;

impl LayoutManager {
    /**
     * Creates the main shell layout
     * 
     * @param area - Available terminal area
     * @return Vec<Rect> - Layout chunks
     */
    pub fn create_main_layout(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area)
            .to_vec()
    }
    
    /**
     * Creates a pane layout for split views
     * 
     * @param area - Available terminal area
     * @param direction - Split direction
     * @param ratios - Split ratios
     * @return Vec<Rect> - Layout chunks
     */
    pub fn create_pane_layout(
        area: Rect,
        direction: Direction,
        ratios: &[u32],
    ) -> Vec<Rect> {
        let constraints: Vec<Constraint> = ratios
            .iter()
            .map(|&ratio| Constraint::Ratio(ratio, ratios.iter().sum()))
            .collect();
        
        Layout::default()
            .direction(direction)
            .constraints(&constraints)
            .split(area)
            .to_vec()
    }
} 