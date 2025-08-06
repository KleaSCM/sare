/**
 * Charts widget for Sare terminal
 * 
 * This module provides chart widgets for data visualization
 * including line charts, bar charts, and pie charts.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: charts.rs
 * Description: Chart widgets for data visualization
 */

use anyhow::Result;
use super::{Widget, WidgetRect, WidgetStyle, WidgetEvent};

/**
 * Chart data point
 * 
 * チャートデータポイントです。
 * チャートの個別データを
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct DataPoint {
	/// Point label
	pub label: String,
	/// Point value
	pub value: f64,
	/// Point color
	pub color: u32,
}

/**
 * Chart type
 * 
 * チャートタイプです。
 * チャートの種類を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
	/// Line chart
	Line,
	/// Bar chart
	Bar,
	/// Pie chart
	Pie,
	/// Scatter plot
	Scatter,
}

/**
 * Chart widget
 * 
 * チャートウィジェットです。
 * データ可視化のための
 * チャートを提供します。
 */
pub struct Chart {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Chart type
	chart_type: ChartType,
	/// Chart data
	data: Vec<DataPoint>,
	/// Chart title
	title: String,
	/// Show legend
	show_legend: bool,
	/// Show grid
	show_grid: bool,
}

impl Chart {
	/**
	 * Creates a new chart
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @param chart_type - Chart type
	 * @return Chart - New chart
	 */
	pub fn new(id: String, rect: WidgetRect, chart_type: ChartType) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			chart_type,
			data: Vec::new(),
			title: String::new(),
			show_legend: true,
			show_grid: true,
		}
	}
	
	/**
	 * Sets the chart data
	 * 
	 * @param data - Chart data points
	 */
	pub fn set_data(&mut self, data: Vec<DataPoint>) {
		self.data = data;
	}
	
	/**
	 * Gets the chart data
	 * 
	 * @return &Vec<DataPoint> - Chart data
	 */
	pub fn get_data(&self) -> &Vec<DataPoint> {
		&self.data
	}
	
	/**
	 * Sets the chart title
	 * 
	 * @param title - Chart title
	 */
	pub fn set_title(&mut self, title: String) {
		self.title = title;
	}
	
	/**
	 * Gets the chart title
	 * 
	 * @return &str - Chart title
	 */
	pub fn get_title(&self) -> &str {
		&self.title
	}
	
	/**
	 * Sets the chart type
	 * 
	 * @param chart_type - Chart type
	 */
	pub fn set_chart_type(&mut self, chart_type: ChartType) {
		self.chart_type = chart_type;
	}
	
	/**
	 * Gets the chart type
	 * 
	 * @return ChartType - Chart type
	 */
	pub fn get_chart_type(&self) -> ChartType {
		self.chart_type
	}
	
	/**
	 * Renders the chart
	 * 
	 * @return String - Rendered chart
	 */
	fn render_chart(&self) -> String {
		let mut result = String::new();
		
		// Add title if available
		if !self.title.is_empty() {
			result.push_str(&format!("{}\n", self.title));
		}
		
		// Render based on chart type
		match self.chart_type {
			ChartType::Line => self.render_line_chart(&mut result),
			ChartType::Bar => self.render_bar_chart(&mut result),
			ChartType::Pie => self.render_pie_chart(&mut result),
			ChartType::Scatter => self.render_scatter_chart(&mut result),
		}
		
		// Add legend if enabled
		if self.show_legend {
			result.push_str(&self.render_legend());
		}
		
		result
	}
	
	/**
	 * Renders a line chart
	 * 
	 * @param result - Output string
	 */
	fn render_line_chart(&self, result: &mut String) {
		if self.data.is_empty() {
			result.push_str("No data available\n");
			return;
		}
		
		let height = self.rect.height as usize;
		let width = self.rect.width as usize;
		
		// Find min and max values
		let min_value = self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
		let max_value = self.data.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
		let value_range = max_value - min_value;
		
		if value_range == 0.0 {
			result.push_str("No value range\n");
			return;
		}
		
		// Render chart
		for y in 0..height {
			let normalized_y = height - 1 - y;
			let value = min_value + (normalized_y as f64 / (height - 1) as f64) * value_range;
			
			result.push_str(&format!("{:8.2} │", value));
			
			// Find data points at this Y level
			for (i, point) in self.data.iter().enumerate() {
				let x = (i * (width - 10) / (self.data.len() - 1).max(1)) + 10;
				let point_y = ((point.value - min_value) / value_range * (height - 1) as f64) as usize;
				
				if point_y == normalized_y {
					result.push_str("●");
				} else if point_y < normalized_y {
					result.push_str("│");
				} else {
					result.push_str(" ");
				}
			}
			result.push('\n');
		}
		
		// Render X axis
		result.push_str("        └");
		for _ in 0..width - 10 {
			result.push('─');
		}
		result.push('\n');
		
		// Render X labels
		result.push_str("          ");
		for (i, point) in self.data.iter().enumerate() {
			if i < width - 10 {
				result.push_str(&format!("{}", point.label.chars().next().unwrap_or(' ')));
			}
		}
		result.push('\n');
	}
	
	/**
	 * Renders a bar chart
	 * 
	 * @param result - Output string
	 */
	fn render_bar_chart(&self, result: &mut String) {
		if self.data.is_empty() {
			result.push_str("No data available\n");
			return;
		}
		
		let height = self.rect.height as usize;
		let width = self.rect.width as usize;
		
		// Find max value
		let max_value = self.data.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
		
		if max_value <= 0.0 {
			result.push_str("No positive values\n");
			return;
		}
		
		// Render bars
		for (i, point) in self.data.iter().enumerate() {
			if i >= height {
				break;
			}
			
			let bar_height = ((point.value / max_value) * (width - 10) as f64) as usize;
			let label = if point.label.len() > 8 {
				&point.label[..8]
			} else {
				&point.label
			};
			
			result.push_str(&format!("{:8} │", label));
			
			for j in 0..width - 10 {
				if j < bar_height {
					result.push('█');
				} else {
					result.push(' ');
				}
			}
			result.push_str(&format!(" {}", point.value));
			result.push('\n');
		}
	}
	
	/**
	 * Renders a pie chart
	 * 
	 * @param result - Output string
	 */
	fn render_pie_chart(&self, result: &mut String) {
		if self.data.is_empty() {
			result.push_str("No data available\n");
			return;
		}
		
		let total = self.data.iter().map(|p| p.value).sum::<f64>();
		
		if total <= 0.0 {
			result.push_str("No positive values\n");
			return;
		}
		
		// Simple pie chart representation
		result.push_str("Pie Chart:\n");
		for point in &self.data {
			let percentage = (point.value / total) * 100.0;
			let bar_length = (percentage / 10.0) as usize;
			let bar = "█".repeat(bar_length);
			result.push_str(&format!("{}: {} ({:.1}%)\n", point.label, bar, percentage));
		}
	}
	
	/**
	 * Renders a scatter chart
	 * 
	 * @param result - Output string
	 */
	fn render_scatter_chart(&self, result: &mut String) {
		if self.data.is_empty() {
			result.push_str("No data available\n");
			return;
		}
		
		let height = self.rect.height as usize;
		let width = self.rect.width as usize;
		
		// Find min and max values
		let min_value = self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
		let max_value = self.data.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
		let value_range = max_value - min_value;
		
		if value_range == 0.0 {
			result.push_str("No value range\n");
			return;
		}
		
		// Create scatter plot grid
		let mut grid = vec![vec![' '; width]; height];
		
		// Plot points
		for point in &self.data {
			let x = ((point.value - min_value) / value_range * (width - 1) as f64) as usize;
			let y = height - 1 - (x % height);
			
			if x < width && y < height {
				grid[y][x] = '●';
			}
		}
		
		// Render grid
		for row in grid {
			result.push_str("│");
			for cell in row {
				result.push(cell);
			}
			result.push_str("│\n");
		}
		
		// Render X axis
		result.push_str("└");
		for _ in 0..width {
			result.push('─');
		}
		result.push_str("┘\n");
	}
	
	/**
	 * Renders the legend
	 * 
	 * @return String - Rendered legend
	 */
	fn render_legend(&self) -> String {
		let mut result = String::new();
		
		if !self.data.is_empty() {
			result.push_str("Legend:\n");
			for point in &self.data {
				result.push_str(&format!("● {}: {}\n", point.label, point.value));
			}
		}
		
		result
	}
}

impl Widget for Chart {
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
		
		Ok(self.render_chart())
	}
	
	fn handle_event(&mut self, _event: WidgetEvent) -> Result<bool> {
		// Charts don't handle events
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Charts don't need regular updates
		Ok(false)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
} 