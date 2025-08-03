/**
 * @file main.rs
 * @brief Main entry point for the Sare shell application
 * 
 * This file initializes the TUI interface, sets up the shell environment,
 * and manages the main application loop for the Sare shell.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file main.rs
 * @description Main application entry point that orchestrates the shell's TUI interface,
 * command parsing, execution, and job management systems.
 */

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

mod shell;
mod tui;
mod config;
mod history;

use shell::Shell;
use tui::TuiManager;

/**
 * Main application entry point
 * 
 * Initializes the terminal, sets up the shell environment,
 * and runs the main application loop until exit is requested.
 * 
 * @return Result<()> - Success or error status
 */
#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Hide)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut shell = Shell::new()?;
    let mut tui_manager = TuiManager::new();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
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
                .split(f.size());
            
            let top_bar = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("sare@{}: {}", 
                            whoami::username(),
                            shell.current_path().display()
                        ),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(top_bar, chunks[0]);
            
            let output_area = Paragraph::new(shell.get_output())
                .block(Block::default().borders(Borders::ALL).title("Output"));
            f.render_widget(output_area, chunks[1]);
            
            let input_area = Paragraph::new(shell.get_input())
                .block(Block::default().borders(Borders::ALL).title(">>>"));
            f.render_widget(input_area, chunks[2]);
        })?;
        
        if let Event::Key(key) = event::read()? {
            match key {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: _,
                    state: _,
                } => {
                    shell.handle_ctrl_c();
                }
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: _,
                    state: _,
                } => {
                    break;
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    shell.execute_command().await?;
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => {
                    shell.add_char(c);
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    shell.remove_char();
                }
                _ => {}
            }
        }
    }
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), Show)?;
    
    Ok(())
} 