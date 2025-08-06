/**
 * Tables widget for Sare terminal
 * 
 * This module provides table widgets for data display
 * including sortable tables and data grids.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: tables.rs
 * Description: Table widgets for data display
 */

use anyhow::Result;
use super::{Widget, WidgetRect, WidgetStyle, WidgetEvent};

/**
 * Table column
 * 
 * テーブルカラムです。
 * テーブルの個別カラムを
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct TableColumn {
	/// Column header
	pub header: String,
	/// Column width
	pub width: u32,
	/// Column sortable
	pub sortable: bool,
}

/**
 * Table row
 * 
 * テーブル行です。
 * テーブルの個別行を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct TableRow {
	/// Row data
	pub data: Vec<String>,
	/// Row selected state
	pub selected: bool,
}

/**
 * Table widget
 * 
 * テーブルウィジェットです。
 * データ表示のための
 * テーブルを提供します。
 */
pub struct Table {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Table columns
	columns: Vec<TableColumn>,
	/// Table rows
	rows: Vec<TableRow>,
	/// Selected row index
	selected_row: Option<usize>,
	/// Sort column index
	sort_column: Option<usize>,
	/// Sort ascending
	sort_ascending: bool,
}

impl Table {
	/**
	 * Creates a new table
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @return Table - New table
	 */
	pub fn new(id: String, rect: WidgetRect) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			columns: Vec::new(),
			rows: Vec::new(),
			selected_row: None,
			sort_column: None,
			sort_ascending: true,
		}
	}
	
	/**
	 * Adds a column to the table
	 * 
	 * @param column - Table column
	 */
	pub fn add_column(&mut self, column: TableColumn) {
		self.columns.push(column);
	}
	
	/**
	 * Adds a row to the table
	 * 
	 * @param row - Table row
	 */
	pub fn add_row(&mut self, row: TableRow) {
		self.rows.push(row);
	}
	
	/**
	 * Gets the columns
	 * 
	 * @return &Vec<TableColumn> - Table columns
	 */
	pub fn get_columns(&self) -> &Vec<TableColumn> {
		&self.columns
	}
	
	/**
	 * Gets the rows
	 * 
	 * @return &Vec<TableRow> - Table rows
	 */
	pub fn get_rows(&self) -> &Vec<TableRow> {
		&self.rows
	}
	
	/**
	 * Sorts the table by column
	 * 
	 * @param column_index - Column index to sort by
	 */
	pub fn sort_by_column(&mut self, column_index: usize) {
		if column_index >= self.columns.len() {
			return;
		}
		
		if self.sort_column == Some(column_index) {
			self.sort_ascending = !self.sort_ascending;
		} else {
			self.sort_column = Some(column_index);
			self.sort_ascending = true;
		}
		
		// Sort rows
		self.rows.sort_by(|a, b| {
			if column_index >= a.data.len() || column_index >= b.data.len() {
				return std::cmp::Ordering::Equal;
			}
			
			let a_val = &a.data[column_index];
			let b_val = &b.data[column_index];
			
			let result = a_val.cmp(b_val);
			if self.sort_ascending {
				result
			} else {
				result.reverse()
			}
		});
	}
	
	/**
	 * Renders the table
	 * 
	 * @return String - Rendered table
	 */
	fn render_table(&self) -> String {
		let mut result = String::new();
		
		if self.columns.is_empty() {
			result.push_str("No columns defined\n");
			return result;
		}
		
		// Render header
		result.push_str(&self.render_header());
		
		// Render separator
		result.push_str(&self.render_separator());
		
		// Render rows
		for (i, row) in self.rows.iter().enumerate() {
			result.push_str(&self.render_row(i, row));
		}
		
		result
	}
	
	/**
	 * Renders the table header
	 * 
	 * @return String - Rendered header
	 */
	fn render_header(&self) -> String {
		let mut result = String::new();
		
		result.push('│');
		for (i, column) in self.columns.iter().enumerate() {
			let header = if self.sort_column == Some(i) {
				if self.sort_ascending {
					format!("{} ▲", column.header)
				} else {
					format!("{} ▼", column.header)
				}
			} else {
				column.header.clone()
			};
			
			result.push_str(&self.pad_cell(&header, column.width));
			result.push('│');
		}
		result.push('\n');
		
		result
	}
	
	/**
	 * Renders the table separator
	 * 
	 * @return String - Rendered separator
	 */
	fn render_separator(&self) -> String {
		let mut result = String::new();
		
		result.push('├');
		for column in &self.columns {
			for _ in 0..column.width {
				result.push('─');
			}
			result.push('┼');
		}
		result.pop(); // Remove last ┼
		result.push('┤');
		result.push('\n');
		
		result
	}
	
	/**
	 * Renders a table row
	 * 
	 * @param row_index - Row index
	 * @param row - Table row
	 * @return String - Rendered row
	 */
	fn render_row(&self, row_index: usize, row: &TableRow) -> String {
		let mut result = String::new();
		
		// Add selection indicator
		if Some(row_index) == self.selected_row {
			result.push('▶');
		} else {
			result.push('│');
		}
		
		for (i, cell) in row.data.iter().enumerate() {
			if i < self.columns.len() {
				result.push_str(&self.pad_cell(cell, self.columns[i].width));
				result.push('│');
			}
		}
		
		// Pad remaining columns
		for i in row.data.len()..self.columns.len() {
			result.push_str(&self.pad_cell("", self.columns[i].width));
			result.push('│');
		}
		
		result.push('\n');
		result
	}
	
	/**
	 * Pads a cell to the specified width
	 * 
	 * @param content - Cell content
	 * @param width - Cell width
	 * @return String - Padded cell
	 */
	fn pad_cell(&self, content: &str, width: u32) -> String {
		let mut result = String::new();
		let content_len = content.chars().count();
		let width = width as usize;
		
		if content_len >= width {
			// Truncate content
			result.push_str(&content.chars().take(width).collect::<String>());
		} else {
			// Pad content
			result.push_str(content);
			for _ in content_len..width {
				result.push(' ');
			}
		}
		
		result
	}
}

impl Widget for Table {
	fn id(&self) -> &str {
		&self.id
	}
	
	fn rect(&self) -> WidgetRect {
		self.rect
	}
	
	fn set_rect(&mut self, rect: WidgetRect) {
		self.rect = rect;
	}
	
	fn style(&self) -> &WidgetStyle {
		&self.style
	}
	
	fn set_style(&mut self, style: WidgetStyle) {
		self.style = style;
	}
	
	fn render(&self) -> Result<String> {
		if !self.visible {
			return Ok(String::new());
		}
		
		Ok(self.render_table())
	}
	
	fn handle_event(&mut self, event: WidgetEvent) -> Result<bool> {
		match event {
			WidgetEvent::Click { x, y, button: super::MouseButton::Left } => {
				// Handle row selection
				if y > 2 && y < 2 + self.rows.len() as u32 {
					let row_index = (y - 2) as usize;
					if row_index < self.rows.len() {
						self.selected_row = Some(row_index);
						return Ok(true);
					}
				}
				
				// Handle column header clicks for sorting
				if y == 1 {
					let mut current_x = 1;
					for (i, column) in self.columns.iter().enumerate() {
						if x >= current_x && x < current_x + column.width {
							self.sort_by_column(i);
							return Ok(true);
						}
						current_x += column.width + 1;
					}
				}
			}
			_ => {}
		}
		
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Tables don't need regular updates
		Ok(false)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
} 