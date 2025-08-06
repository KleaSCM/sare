/**
 * Custom widgets module for Sare terminal
 * 
 * This module provides custom terminal widgets and UI components
 * including progress bars, charts, buttons, and interactive elements.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Custom terminal widgets and UI components
 */

pub mod progress_bar;
pub mod charts;
pub mod buttons;
pub mod tables;
pub mod indicators;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Widget position and size
 * 
 * ウィジェットの位置とサイズです。
 * ウィジェットの表示位置と
 * サイズを管理します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidgetRect {
	/// X coordinate
	pub x: u32,
	/// Y coordinate  
	pub y: u32,
	/// Width in characters
	pub width: u32,
	/// Height in characters
	pub height: u32,
}

impl WidgetRect {
	/**
	 * Creates a new widget rectangle
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param width - Width in characters
	 * @param height - Height in characters
	 * @return WidgetRect - New widget rectangle
	 */
	pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
		Self { x, y, width, height }
	}
	
	/**
	 * Checks if a point is inside the rectangle
	 * 
	 * @param px - Point X coordinate
	 * @param py - Point Y coordinate
	 * @return bool - Whether point is inside
	 */
	pub fn contains(&self, px: u32, py: u32) -> bool {
		px >= self.x && px < self.x + self.width &&
		py >= self.y && py < self.y + self.height
	}
	
	/**
	 * Gets the center point of the rectangle
	 * 
	 * @return (u32, u32) - Center coordinates
	 */
	pub fn center(&self) -> (u32, u32) {
		(self.x + self.width / 2, self.y + self.height / 2)
	}
}

/**
 * Widget style configuration
 * 
 * ウィジェットスタイル設定です。
 * ウィジェットの見た目を
 * カスタマイズします。
 */
#[derive(Debug, Clone)]
pub struct WidgetStyle {
	/// Foreground color
	pub fg_color: u32,
	/// Background color
	pub bg_color: u32,
	/// Border color
	pub border_color: u32,
	/// Border style
	pub border_style: BorderStyle,
	/// Padding
	pub padding: u32,
	/// Margin
	pub margin: u32,
	/// Font style
	pub font_style: FontStyle,
}

impl Default for WidgetStyle {
	fn default() -> Self {
		Self {
			fg_color: 0xFFFFFF, // White
			bg_color: 0x000000, // Black
			border_color: 0x666666, // Gray
			border_style: BorderStyle::Single,
			padding: 1,
			margin: 0,
			font_style: FontStyle::Normal,
		}
	}
}

/**
 * Border style for widgets
 * 
 * ウィジェットの境界線スタイルです。
 * 境界線の表示方法を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
	/// No border
	None,
	/// Single line border
	Single,
	/// Double line border
	Double,
	/// Rounded border
	Rounded,
	/// Dashed border
	Dashed,
}

/**
 * Font style for widgets
 * 
 * ウィジェットのフォントスタイルです。
 * テキストの表示スタイルを
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
	/// Normal text
	Normal,
	/// Bold text
	Bold,
	/// Italic text
	Italic,
	/// Underlined text
	Underlined,
}

/**
 * Widget event types
 * 
 * ウィジェットイベントタイプです。
 * ウィジェットの相互作用を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub enum WidgetEvent {
	/// Mouse click event
	Click { x: u32, y: u32, button: MouseButton },
	/// Mouse hover event
	Hover { x: u32, y: u32 },
	/// Mouse leave event
	Leave,
	/// Key press event
	KeyPress { key: String },
	/// Focus gained event
	FocusGained,
	/// Focus lost event
	FocusLost,
	/// Value changed event
	ValueChanged { value: String },
}

/**
 * Mouse button types
 * 
 * マウスボタンタイプです。
 * マウスボタンの種類を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
	/// Left mouse button
	Left,
	/// Right mouse button
	Right,
	/// Middle mouse button
	Middle,
}

/**
 * Base widget trait
 * 
 * ベースウィジェットトレイトです。
 * すべてのウィジェットが
 * 実装する必要があります。
 */
pub trait Widget {
	/**
	 * Gets the widget ID
	 * 
	 * @return &str - Widget ID
	 */
	fn id(&self) -> &str;
	
	/**
	 * Gets the widget position and size
	 * 
	 * @return WidgetRect - Widget rectangle
	 */
	fn rect(&self) -> WidgetRect;
	
	/**
	 * Sets the widget position and size
	 * 
	 * @param rect - New widget rectangle
	 */
	fn set_rect(&mut self, rect: WidgetRect);
	
	/**
	 * Gets the widget style
	 * 
	 * @return &WidgetStyle - Widget style
	 */
	fn style(&self) -> &WidgetStyle;
	
	/**
	 * Sets the widget style
	 * 
	 * @param style - New widget style
	 */
	fn set_style(&mut self, style: WidgetStyle);
	
	/**
	 * Renders the widget
	 * 
	 * @return Result<String> - Rendered widget content
	 */
	fn render(&self) -> Result<String>;
	
	/**
	 * Handles widget events
	 * 
	 * @param event - Widget event
	 * @return Result<bool> - Whether event was handled
	 */
	fn handle_event(&mut self, event: WidgetEvent) -> Result<bool>;
	
	/**
	 * Updates the widget
	 * 
	 * @return Result<bool> - Whether widget needs redraw
	 */
	fn update(&mut self) -> Result<bool>;
	
	/**
	 * Checks if widget is visible
	 * 
	 * @return bool - Whether widget is visible
	 */
	fn is_visible(&self) -> bool;
	
	/**
	 * Sets widget visibility
	 * 
	 * @param visible - Whether widget should be visible
	 */
	fn set_visible(&mut self, visible: bool);
}

/**
 * Widget manager
 * 
 * ウィジェットマネージャーです。
 * すべてのウィジェットを
 * 管理します。
 */
pub struct WidgetManager {
	/// Registered widgets
	widgets: Arc<RwLock<HashMap<String, Box<dyn Widget + Send + Sync>>>>,
	/// Widget event handlers
	event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(WidgetEvent) + Send + Sync>>>>>,
	/// Focused widget ID
	focused_widget: Arc<RwLock<Option<String>>>,
	/// Widget z-order
	z_order: Arc<RwLock<Vec<String>>>,
}

impl WidgetManager {
	/**
	 * Creates a new widget manager
	 * 
	 * @return WidgetManager - New widget manager
	 */
	pub fn new() -> Self {
		Self {
			widgets: Arc::new(RwLock::new(HashMap::new())),
			event_handlers: Arc::new(RwLock::new(HashMap::new())),
			focused_widget: Arc::new(RwLock::new(None)),
			z_order: Arc::new(RwLock::new(Vec::new())),
		}
	}
	
	/**
	 * Registers a widget
	 * 
	 * @param widget - Widget to register
	 * @return Result<()> - Success or error status
	 */
	pub async fn register_widget(&self, widget: Box<dyn Widget + Send + Sync>) -> Result<()> {
		let id = widget.id().to_string();
		let mut widgets = self.widgets.write().await;
		widgets.insert(id.clone(), widget);
		
		let mut z_order = self.z_order.write().await;
		z_order.push(id);
		
		Ok(())
	}
	
	/**
	 * Unregisters a widget
	 * 
	 * @param id - Widget ID to unregister
	 * @return Result<()> - Success or error status
	 */
	pub async fn unregister_widget(&self, id: &str) -> Result<()> {
		let mut widgets = self.widgets.write().await;
		widgets.remove(id);
		
		let mut z_order = self.z_order.write().await;
		z_order.retain(|widget_id| widget_id != id);
		
		Ok(())
	}
	
	/**
	 * Gets a widget by ID
	 * 
	 * @param id - Widget ID
	 * @return Option<Box<dyn Widget + Send + Sync>> - Widget or None
	 */
	pub async fn get_widget(&self, id: &str) -> Option<Box<dyn Widget + Send + Sync>> {
		let widgets = self.widgets.read().await;
		widgets.get(id).cloned()
	}
	
	/**
	 * Renders all widgets
	 * 
	 * @return Result<String> - Rendered widget content
	 */
	pub async fn render_all(&self) -> Result<String> {
		let mut result = String::new();
		let z_order = self.z_order.read().await;
		let widgets = self.widgets.read().await;
		
		for widget_id in z_order.iter() {
			if let Some(widget) = widgets.get(widget_id) {
				if widget.is_visible() {
					result.push_str(&widget.render()?);
					result.push('\n');
				}
			}
		}
		
		Ok(result)
	}
	
	/**
	 * Handles widget events
	 * 
	 * @param event - Widget event
	 * @return Result<bool> - Whether event was handled
	 */
	pub async fn handle_event(&self, event: WidgetEvent) -> Result<bool> {
		let mut widgets = self.widgets.write().await;
		let mut event_handlers = self.event_handlers.write().await;
		
		// Handle click events for widget selection
		if let WidgetEvent::Click { x, y, button: MouseButton::Left } = &event {
			for (id, widget) in widgets.iter_mut() {
				if widget.rect().contains(*x, *y) {
					let mut focused = self.focused_widget.write().await;
					*focused = Some(id.clone());
					
					// Call event handlers
					if let Some(handlers) = event_handlers.get(id) {
						for handler in handlers {
							handler(event.clone());
						}
					}
					
					return Ok(true);
				}
			}
		}
		
		// Handle events for focused widget
		if let Some(focused_id) = &*self.focused_widget.read().await {
			if let Some(widget) = widgets.get_mut(focused_id) {
				if widget.handle_event(event.clone())? {
					// Call event handlers
					if let Some(handlers) = event_handlers.get(focused_id) {
						for handler in handlers {
							handler(event);
						}
					}
					return Ok(true);
				}
			}
		}
		
		Ok(false)
	}
	
	/**
	 * Updates all widgets
	 * 
	 * @return Result<bool> - Whether any widget needs redraw
	 */
	pub async fn update_all(&self) -> Result<bool> {
		let mut widgets = self.widgets.write().await;
		let mut needs_redraw = false;
		
		for widget in widgets.values_mut() {
			if widget.update()? {
				needs_redraw = true;
			}
		}
		
		Ok(needs_redraw)
	}
	
	/**
	 * Adds an event handler for a widget
	 * 
	 * @param widget_id - Widget ID
	 * @param handler - Event handler function
	 */
	pub async fn add_event_handler<F>(&self, widget_id: &str, handler: F)
	where
		F: Fn(WidgetEvent) + Send + Sync + 'static,
	{
		let mut event_handlers = self.event_handlers.write().await;
		event_handlers
			.entry(widget_id.to_string())
			.or_insert_with(Vec::new)
			.push(Box::new(handler));
	}
} 