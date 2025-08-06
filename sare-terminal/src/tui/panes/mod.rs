/**
 * @file mod.rs
 * @brief Multi-pane terminal interface for Sare
 * 
 * This module provides multi-pane terminal capabilities for the Sare terminal,
 * enabling developers to work with multiple terminal sessions simultaneously
 * in an IDE-like interface with split panes, independent shell sessions,
 * and advanced pane management features.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Multi-pane module that provides split pane functionality
 * and independent shell sessions for the Sare terminal.
 */

pub mod layout;
pub mod session;
pub mod navigation;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::terminal::{TerminalEmulator, TerminalConfig};

/**
 * Pane manager for multi-pane terminal
 * 
 * Manages multiple terminal panes with independent shell sessions,
 * layout management, and pane navigation for developer workflows.
 */
pub struct PaneManager {
	/// Active panes
	panes: Arc<RwLock<HashMap<String, Pane>>>,
	/// Pane layout
	layout: PaneLayout,
	/// Currently focused pane
	focused_pane: Option<String>,
	/// Pane configuration
	config: PaneConfig,
}

/**
 * Individual pane information
 * 
 * Contains information about a single terminal pane
 * including its session, layout, and state.
 */
#[derive(Clone)]
pub struct Pane {
	/// Pane ID
	pub pane_id: String,
	/// Pane title
	pub title: String,
	/// Terminal emulator for this pane
	pub terminal: Arc<RwLock<TerminalEmulator>>,
	/// Pane layout information
	pub layout: PaneLayoutInfo,
	/// Pane state
	pub state: PaneState,
	/// Pane history
	pub history: Vec<String>,
}

/**
 * Pane layout information
 * 
 * Contains layout information for a single pane
 * including position, size, and split information.
 */
#[derive(Debug, Clone)]
pub struct PaneLayoutInfo {
	/// Pane position (x, y)
	pub position: (u16, u16),
	/// Pane size (width, height)
	pub size: (u16, u16),
	/// Parent pane ID (if split)
	pub parent_id: Option<String>,
	/// Child pane IDs (if split)
	pub child_ids: Vec<String>,
	/// Split direction
	pub split_direction: Option<SplitDirection>,
}

/**
 * Split direction enumeration
 * 
 * Defines the different ways a pane can be split.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
	/// Split horizontally (top/bottom)
	Horizontal,
	/// Split vertically (left/right)
	Vertical,
}

/**
 * Pane state information
 * 
 * Contains the current state of a pane including
 * focus, activity, and session status.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum PaneState {
	/// Pane is active and ready
	Active,
	/// Pane is focused
	Focused,
	/// Pane is inactive
	Inactive,
	/// Pane has an error
	Error(String),
}

/**
 * Pane layout manager
 * 
 * Manages the overall layout of all panes including
 * split management, resizing, and layout algorithms.
 */
#[derive(Debug, Clone)]
pub struct PaneLayout {
	/// Root pane ID
	root_pane_id: Option<String>,
	/// Layout tree
	layout_tree: HashMap<String, LayoutNode>,
	/// Layout algorithm
	algorithm: LayoutAlgorithm,
}

/**
 * Layout node information
 * 
 * Contains information about a node in the layout tree
 * including its children and split information.
 */
#[derive(Debug, Clone)]
pub struct LayoutNode {
	/// Node ID
	pub node_id: String,
	/// Node type
	pub node_type: NodeType,
	/// Child node IDs
	pub children: Vec<String>,
	/// Split ratio (0.0 to 1.0)
	pub split_ratio: f32,
	/// Split direction
	pub split_direction: Option<SplitDirection>,
}

/**
 * Node type enumeration
 * 
 * Defines the different types of layout nodes.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
	/// Leaf node (terminal pane)
	Leaf,
	/// Split node (container)
	Split,
}

/**
 * Layout algorithm enumeration
 * 
 * Defines the different layout algorithms available.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutAlgorithm {
	/// Binary tree layout
	BinaryTree,
	/// Grid layout
	Grid,
	/// Manual layout
	Manual,
}

/**
 * Pane configuration
 * 
 * Contains configuration options for pane management
 * including default settings and behavior options.
 */
#[derive(Debug, Clone)]
pub struct PaneConfig {
	/// Default shell for new panes
	pub default_shell: String,
	/// Default pane size
	pub default_size: (u16, u16),
	/// Enable pane synchronization
	pub sync_panes: bool,
	/// Enable pane history
	pub enable_history: bool,
	/// Maximum panes allowed
	pub max_panes: usize,
	/// Pane title format
	pub title_format: String,
}

impl Default for PaneConfig {
	fn default() -> Self {
		Self {
			default_shell: "/bin/bash".to_string(),
			default_size: (80, 24),
			sync_panes: false,
			enable_history: true,
			max_panes: 10,
			title_format: "{shell} - {cwd}".to_string(),
		}
	}
}

impl PaneManager {
	/**
	 * Creates a new pane manager
	 * 
	 * @param config - Pane configuration
	 * @return PaneManager - New pane manager instance
	 */
	pub fn new(config: PaneConfig) -> Self {
		/**
		 * ペインマネージャーを初期化する関数です
		 * 
		 * 複数の独立したターミナルペインを管理するための
		 * ペインマネージャーを作成し、レイアウト管理と
		 * フォーカス管理を初期化します。
		 * 
		 * 各ペインは独立したターミナルエミュレーターを持ち、
		 * 適切な分離と状態管理を提供します。レイアウトは
		 * BinaryTreeアルゴリズムで初期化されます。
		 */
		
		Self {
			panes: Arc::new(RwLock::new(HashMap::new())),
			layout: PaneLayout {
				root_pane_id: None,
				layout_tree: HashMap::new(),
				algorithm: LayoutAlgorithm::BinaryTree,
			},
			focused_pane: None,
			config,
		}
	}
	
	/**
	 * Creates a new pane
	 * 
	 * Creates a new terminal pane with an independent shell session
	 * and adds it to the pane manager.
	 * 
	 * @param title - Pane title
	 * @param shell - Shell to use
	 * @return Result<String> - Pane ID or error
	 */
	pub async fn create_pane(&mut self, title: &str, shell: &str) -> Result<String> {
		/**
		 * 新しいペインを作成する関数です
		 * 
		 * 指定されたタイトルとシェルを使用して新しいターミナルペインを
		 * 作成し、独立したシェルセッションを初期化します。
		 * 
		 * ターミナルエミュレーターを設定で初期化し、ペインレイアウト情報を
		 * 作成してペインマネージャーに追加します。最初のペインの場合は
		 * ルートペインとして設定し、フォーカスも設定します。
		 * 
		 * 各ペインは独立したターミナルセッションを持ち、適切な
		 * 分離と状態管理を提供します。
		 */
		
		let pane_id = uuid::Uuid::new_v4().to_string();
		
		// Create terminal configuration
		let terminal_config = TerminalConfig {
			default_shell: shell.to_string(),
			size: self.config.default_size,
			..Default::default()
		};
		
		// Create terminal emulator
		let terminal = TerminalEmulator::new(terminal_config)?;
		
		// Create pane layout info
		let layout_info = PaneLayoutInfo {
			position: (0, 0),
			size: self.config.default_size,
			parent_id: None,
			child_ids: Vec::new(),
			split_direction: None,
		};
		
		// Create pane
		let pane = Pane {
			pane_id: pane_id.clone(),
			title: title.to_string(),
			terminal: Arc::new(RwLock::new(terminal)),
			layout: layout_info,
			state: PaneState::Active,
			history: Vec::new(),
		};
		
		// Add pane to manager
		let mut panes = self.panes.write().await;
		panes.insert(pane_id.clone(), pane);
		
		// Set as root if first pane
		if self.layout.root_pane_id.is_none() {
			self.layout.root_pane_id = Some(pane_id.clone());
		}
		
		// Set as focused
		self.focused_pane = Some(pane_id.clone());
		
		Ok(pane_id)
	}
	
	/**
	 * Splits a pane
	 * 
	 * Splits an existing pane into two panes with the specified
	 * direction and creates a new shell session.
	 * 
	 * @param pane_id - Pane ID to split
	 * @param direction - Split direction
	 * @param title - Title for new pane
	 * @return Result<String> - New pane ID or error
	 */
	pub async fn split_pane(&mut self, pane_id: &str, direction: SplitDirection, title: &str) -> Result<String> {
		/**
		 * ペイン分割の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なレイアウト管理を行います。
		 * ペイン分割とレイアウト再計算が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let new_pane_id = uuid::Uuid::new_v4().to_string();
		
		// Create new pane
		let shell = self.config.default_shell.clone();
		let new_pane_id = self.create_pane(title, &shell).await?;
		
		// Update layout
		let mut panes = self.panes.write().await;
		
				// Get parent pane info first
		let parent_info = if let Some(parent_pane) = panes.get(pane_id) {
			(parent_pane.layout.size, parent_pane.layout.position)
		} else {
			return Err(anyhow::anyhow!("Parent pane not found"));
		};
		
		// Update parent pane layout
		if let Some(parent_pane) = panes.get_mut(pane_id) {
			parent_pane.layout.child_ids.push(new_pane_id.clone());
			parent_pane.layout.split_direction = Some(direction.clone());
		}
		
		// Update new pane layout and calculate sizes
		if let Some(new_pane) = panes.get_mut(&new_pane_id) {
			new_pane.layout.parent_id = Some(pane_id.to_string());
			
			// Use stored parent info for calculations
			let (parent_size, parent_position) = parent_info;
			
			// Calculate new sizes based on split direction
			match direction {
				SplitDirection::Horizontal => {
					// Split vertically (left/right)
					let parent_width = parent_size.0;
					let new_width = parent_width / 2;
					
					new_pane.layout.size.0 = new_width;
					new_pane.layout.position.0 = parent_position.0 + new_width;
				}
				SplitDirection::Vertical => {
					// Split horizontally (top/bottom)
					let parent_height = parent_size.1;
					let new_height = parent_height / 2;
					
					new_pane.layout.size.1 = new_height;
					new_pane.layout.position.1 = parent_position.1 + new_height;
				}
			}
			
			// Update parent pane size after new pane is configured
			if let Some(parent_pane) = panes.get_mut(pane_id) {
				match direction {
					SplitDirection::Horizontal => {
						parent_pane.layout.size.0 = parent_size.0 / 2;
					}
					SplitDirection::Vertical => {
						parent_pane.layout.size.1 = parent_size.1 / 2;
					}
				}
			}
		}
		
		// Set new pane as focused
		self.focused_pane = Some(new_pane_id.clone());
		
		Ok(new_pane_id)
	}
	
	/**
	 * Focuses a pane
	 * 
	 * Sets the specified pane as the focused pane
	 * and updates the pane states accordingly.
	 * 
	 * @param pane_id - Pane ID to focus
	 * @return Result<()> - Success or error status
	 */
	pub async fn focus_pane(&mut self, pane_id: &str) -> Result<()> {
		let mut panes = self.panes.write().await;
		
		// Update all pane states
		for pane in panes.values_mut() {
			if pane.pane_id == pane_id {
				pane.state = PaneState::Focused;
			} else {
				pane.state = PaneState::Active;
			}
		}
		
		self.focused_pane = Some(pane_id.to_string());
		
		Ok(())
	}
	
	/**
	 * Closes a pane
	 * 
	 * Closes the specified pane and cleans up its resources.
	 * If it was a split pane, the layout is adjusted accordingly.
	 * 
	 * @param pane_id - Pane ID to close
	 * @return Result<()> - Success or error status
	 */
	pub async fn close_pane(&mut self, pane_id: &str) -> Result<()> {
		/**
		 * ペイン終了の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なリソース管理を行います。
		 * ペイン終了とレイアウト調整が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut panes = self.panes.write().await;
		
		// Get pane info first
		let pane_info = if let Some(pane) = panes.get(pane_id) {
			(pane.layout.parent_id.clone(), pane.terminal.clone())
		} else {
			return Ok(());
		};
		
		// Stop terminal session
		let mut terminal = pane_info.1.write().await;
		terminal.stop_session().await?;
		
		// Remove pane
		panes.remove(pane_id);
		
		// Update layout if needed
		if let Some(parent_id) = &pane_info.0 {
			if let Some(parent_pane) = panes.get_mut(parent_id) {
					// Remove from parent's children
					parent_pane.layout.child_ids.retain(|id| id != pane_id);
					
					// If no more children, remove split
					if parent_pane.layout.child_ids.is_empty() {
						parent_pane.layout.split_direction = None;
					}
			}
			
			// Update focused pane if needed
			if self.focused_pane.as_ref() == Some(&pane_id.to_string()) {
				// Focus first available pane
				if let Some((first_id, _)) = panes.iter().next() {
					self.focused_pane = Some(first_id.clone());
				} else {
					self.focused_pane = None;
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Resizes a pane
	 * 
	 * Resizes the specified pane and adjusts the layout
	 * of neighboring panes accordingly.
	 * 
	 * @param pane_id - Pane ID to resize
	 * @param new_size - New size (width, height)
	 * @return Result<()> - Success or error status
	 */
	pub async fn resize_pane(&mut self, pane_id: &str, new_size: (u16, u16)) -> Result<()> {
		let mut panes = self.panes.write().await;
		
		if let Some(pane) = panes.get_mut(pane_id) {
			pane.layout.size = new_size;
			
			// Implement layout recalculation
			use crate::tui::panes::layout::LayoutManager;
			
			// Create layout manager for recalculation
			let layout_manager = LayoutManager::new(
				self.layout.algorithm.clone(),
				crate::tui::panes::layout::LayoutConstraints::default()
			);
			
			// Get all pane IDs for layout calculation
			let pane_ids: Vec<String> = panes.keys().cloned().collect();
			let total_size = (80, 24); // Default terminal size
			
			// Calculate new layout for all panes
			let layout_results = layout_manager.calculate_layout(&pane_ids, total_size)?;
			
			// Update pane sizes and positions
			for (pane_id, layout_result) in layout_results {
				if let Some(pane) = panes.get_mut(&pane_id) {
					pane.layout.position = layout_result.position;
					pane.layout.size = layout_result.size;
					
					// Notify terminal of size change
					let mut terminal = pane.terminal.write().await;
					terminal.resize(layout_result.size.0, layout_result.size.1).await?;
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets a pane
	 * 
	 * @param pane_id - Pane ID
	 * @return Option<Pane> - Pane if found
	 */
	pub async fn get_pane(&self, pane_id: &str) -> Option<Pane> {
		let panes = self.panes.read().await;
		panes.get(pane_id).cloned()
	}
	
	/**
	 * Lists all panes
	 * 
	 * @return Vec<Pane> - List of all panes
	 */
	pub async fn list_panes(&self) -> Vec<Pane> {
		let panes = self.panes.read().await;
		panes.values().cloned().collect()
	}
	
	/**
	 * Gets the focused pane
	 * 
	 * @return Option<String> - Focused pane ID
	 */
	pub fn focused_pane(&self) -> Option<String> {
		self.focused_pane.clone()
	}
	
	/**
	 * Gets pane configuration
	 * 
	 * @return &PaneConfig - Pane configuration
	 */
	pub fn config(&self) -> &PaneConfig {
		&self.config
	}
	
	/**
	 * Synchronizes panes
	 * 
	 * Sends the same input to all panes for synchronized
	 * operation across multiple terminal sessions.
	 * 
	 * @param input - Input to send to all panes
	 * @return Result<()> - Success or error status
	 */
	pub async fn sync_panes(&self, input: &[u8]) -> Result<()> {
		if !self.config.sync_panes {
			return Ok(());
		}
		
		let panes = self.panes.read().await;
		
		// Implement session synchronization
		use crate::tui::panes::session::SessionManager;
		
		// Create session manager for synchronization
		let mut session_manager = SessionManager::new();
		
		// Get all session IDs from panes
		let session_ids: Vec<String> = panes.values()
			.map(|pane| pane.pane_id.clone())
			.collect();
		
		// Synchronize all sessions
		session_manager.synchronize_sessions(&session_ids).await?;
		
		// Send input to all panes
		for pane in panes.values() {
			let terminal = pane.terminal.read().await;
			terminal.send_input(input).await?;
		}
		
		Ok(())
	}
} 