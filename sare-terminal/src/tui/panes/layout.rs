/**
 * @file layout.rs
 * @brief Pane layout management for Sare terminal
 * 
 * This module provides layout management capabilities for the multi-pane
 * terminal interface, including pane positioning, sizing algorithms,
 * and layout tree management for developer workflows.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file layout.rs
 * @description Layout module that provides pane positioning and sizing
 * algorithms for the multi-pane terminal interface.
 */

use anyhow::Result;
use std::collections::HashMap;

use super::{PaneLayout, LayoutNode, NodeType, SplitDirection, LayoutAlgorithm};

/**
 * Layout manager for pane positioning
 * 
 * Manages pane layout algorithms, positioning, and sizing
 * for optimal developer workflow experience.
 */
pub struct LayoutManager {
	/// Current layout algorithm
	algorithm: LayoutAlgorithm,
	/// Layout tree
	layout_tree: HashMap<String, LayoutNode>,
	/// Layout constraints
	constraints: LayoutConstraints,
}

/**
 * Layout constraints
 * 
 * Defines constraints for layout calculations including
 * minimum sizes, spacing, and aspect ratios.
 */
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
	/// Minimum pane width
	pub min_width: u16,
	/// Minimum pane height
	pub min_height: u16,
	/// Pane spacing
	pub spacing: u16,
	/// Aspect ratio constraints
	pub aspect_ratio: Option<f32>,
	/// Maximum panes per row/column
	pub max_panes_per_dimension: usize,
}

impl Default for LayoutConstraints {
	fn default() -> Self {
		Self {
			min_width: 20,
			min_height: 10,
			spacing: 1,
			aspect_ratio: None,
			max_panes_per_dimension: 4,
		}
	}
}

/**
 * Layout calculation result
 * 
 * Contains the calculated layout information for a pane
 * including position, size, and constraints.
 */
#[derive(Debug, Clone)]
pub struct LayoutResult {
	/// Pane position (x, y)
	pub position: (u16, u16),
	/// Pane size (width, height)
	pub size: (u16, u16),
	/// Layout constraints met
	pub constraints_met: bool,
	/// Layout algorithm used
	pub algorithm: LayoutAlgorithm,
}

impl LayoutManager {
	/**
	 * Creates a new layout manager
	 * 
	 * @param algorithm - Layout algorithm to use
	 * @param constraints - Layout constraints
	 * @return LayoutManager - New layout manager instance
	 */
	pub fn new(algorithm: LayoutAlgorithm, constraints: LayoutConstraints) -> Self {
		/**
		 * レイアウトマネージャーを初期化する関数です
		 * 
		 * 指定されたレイアウトアルゴリズムと制約設定を使用して
		 * レイアウトマネージャーを初期化し、空のレイアウトツリーを
		 * 作成します。
		 * 
		 * アルゴリズム（BinaryTree、Grid、Manual）と制約設定
		 * （最小サイズ、間隔、アスペクト比など）を設定し、
		 * ペイン配置計算の準備を整えます。
		 */
		
		Self {
			algorithm,
			layout_tree: HashMap::new(),
			constraints,
		}
	}
	
	/**
	 * Calculates layout for all panes
	 * 
	 * Calculates the optimal layout for all panes based on
	 * the current layout algorithm and constraints.
	 * 
	 * @param pane_ids - List of pane IDs to layout
	 * @param total_size - Total available size (width, height)
	 * @return Result<HashMap<String, LayoutResult>> - Layout results for each pane
	 */
	pub fn calculate_layout(&self, pane_ids: &[String], total_size: (u16, u16)) -> Result<HashMap<String, LayoutResult>> {
		/**
		 * 全ペインの最適レイアウトを計算する関数です
		 * 
		 * 現在のレイアウトアルゴリズムと制約設定に基づいて、
		 * 指定されたペインIDリストの最適な配置を計算します。
		 * 
		 * BinaryTree、Grid、Manualの各アルゴリズムに対応し、
		 * 制約設定（最小サイズ、間隔、アスペクト比）を適用して
		 * 各ペインの位置とサイズを決定します。
		 * 
		 * 計算結果はLayoutResultとして返され、制約が満たされているか
		 * どうかの情報も含まれます。
		 */
		
		match self.algorithm {
			LayoutAlgorithm::BinaryTree => self.calculate_binary_tree_layout(pane_ids, total_size),
			LayoutAlgorithm::Grid => self.calculate_grid_layout(pane_ids, total_size),
			LayoutAlgorithm::Manual => self.calculate_manual_layout(pane_ids, total_size),
		}
	}
	
	/**
	 * Calculates binary tree layout
	 * 
	 * Calculates layout using a binary tree algorithm where
	 * panes are split recursively in alternating directions.
	 * 
	 * @param pane_ids - List of pane IDs
	 * @param total_size - Total available size
	 * @return Result<HashMap<String, LayoutResult>> - Layout results
	 */
	fn calculate_binary_tree_layout(&self, pane_ids: &[String], total_size: (u16, u16)) -> Result<HashMap<String, LayoutResult>> {
		let mut results = HashMap::new();
		
		if pane_ids.is_empty() {
			return Ok(results);
		}
		
		let (width, height) = total_size;
		let num_panes = pane_ids.len();
		
		// Calculate initial pane size
		let mut pane_width = width;
		let mut pane_height = height;
		
		// Apply binary tree splitting
		for (i, pane_id) in pane_ids.iter().enumerate() {
			let split_level = (i as f32).log2().floor() as u32;
			let direction = if split_level % 2 == 0 {
				SplitDirection::Vertical
			} else {
				SplitDirection::Horizontal
			};
			
			// Calculate position and size based on split level
			let position = self.calculate_binary_position(i, split_level, total_size);
			let size = self.calculate_binary_size(split_level, total_size);
			
			// Apply constraints
			let constrained_size = self.apply_constraints(size);
			
			results.insert(pane_id.clone(), LayoutResult {
				position,
				size: constrained_size,
				constraints_met: self.check_constraints(constrained_size),
				algorithm: LayoutAlgorithm::BinaryTree,
			});
		}
		
		Ok(results)
	}
	
	/**
	 * Calculates grid layout
	 * 
	 * Calculates layout using a grid algorithm where
	 * panes are arranged in a rectangular grid.
	 * 
	 * @param pane_ids - List of pane IDs
	 * @param total_size - Total available size
	 * @return Result<HashMap<String, LayoutResult>> - Layout results
	 */
	fn calculate_grid_layout(&self, pane_ids: &[String], total_size: (u16, u16)) -> Result<HashMap<String, LayoutResult>> {
		let mut results = HashMap::new();
		
		if pane_ids.is_empty() {
			return Ok(results);
		}
		
		let (width, height) = total_size;
		let num_panes = pane_ids.len();
		
		// Calculate grid dimensions
		let grid_cols = (num_panes as f32).sqrt().ceil() as u16;
		let grid_rows = ((num_panes as f32) / grid_cols as f32).ceil() as u16;
		
		// Calculate cell size
		let cell_width = (width - (grid_cols - 1) * self.constraints.spacing) / grid_cols;
		let cell_height = (height - (grid_rows - 1) * self.constraints.spacing) / grid_rows;
		
		for (i, pane_id) in pane_ids.iter().enumerate() {
			let row = (i as u16) / grid_cols;
			let col = (i as u16) % grid_cols;
			
			let position = (
				col * (cell_width + self.constraints.spacing),
				row * (cell_height + self.constraints.spacing),
			);
			
			let size = (cell_width, cell_height);
			let constrained_size = self.apply_constraints(size);
			
			results.insert(pane_id.clone(), LayoutResult {
				position,
				size: constrained_size,
				constraints_met: self.check_constraints(constrained_size),
				algorithm: LayoutAlgorithm::Grid,
			});
		}
		
		Ok(results)
	}
	
	/**
	 * Calculates manual layout
	 * 
	 * Calculates layout using manual positioning where
	 * panes maintain their current positions and sizes.
	 * 
	 * @param pane_ids - List of pane IDs
	 * @param total_size - Total available size
	 * @return Result<HashMap<String, LayoutResult>> - Layout results
	 */
	fn calculate_manual_layout(&self, pane_ids: &[String], total_size: (u16, u16)) -> Result<HashMap<String, LayoutResult>> {
		let mut results = HashMap::new();
		let (total_width, total_height) = total_size;
		
		// Get existing pane positions from layout tree
		for (i, pane_id) in pane_ids.iter().enumerate() {
			let position = if let Some(node) = self.layout_tree.get(pane_id) {
				// Use existing position if available
				(0, 0) // Placeholder position
			} else {
				// Calculate default position based on pane index
				let cols = (pane_ids.len() as f32).sqrt().ceil() as u16;
				let rows = ((pane_ids.len() as f32) / cols as f32).ceil() as u16;
				
				let pane_width = total_width / cols;
				let pane_height = total_height / rows;
				
				let col = (i as u16) % cols;
				let row = (i as u16) / cols;
				
				(col * pane_width, row * pane_height)
			};
			
			let size = if let Some(node) = self.layout_tree.get(pane_id) {
				// Use existing size if available
				(80, 24) // Placeholder size
			} else {
				// Calculate default size
				let cols = (pane_ids.len() as f32).sqrt().ceil() as u16;
				let rows = ((pane_ids.len() as f32) / cols as f32).ceil() as u16;
				
				let pane_width = total_width / cols;
				let pane_height = total_height / rows;
				
				(pane_width, pane_height)
			};
			
			// Apply constraints to size
			let constrained_size = self.apply_constraints(size);
			
			// Ensure position is within bounds
			let constrained_position = (
				position.0.min(total_width.saturating_sub(constrained_size.0)),
				position.1.min(total_height.saturating_sub(constrained_size.1))
			);
			
			// Check if constraints are met
			let constraints_met = self.check_constraints(constrained_size);
			
			results.insert(pane_id.clone(), LayoutResult {
				position: constrained_position,
				size: constrained_size,
				constraints_met,
				algorithm: LayoutAlgorithm::Manual,
			});
		}
		
		Ok(results)
	}
	
	/**
	 * Calculates binary tree position
	 * 
	 * Calculates the position of a pane in a binary tree layout
	 * based on its index and split level.
	 * 
	 * @param index - Pane index
	 * @param split_level - Current split level
	 * @param total_size - Total available size
	 * @return (u16, u16) - Calculated position
	 */
	fn calculate_binary_position(&self, index: usize, split_level: u32, total_size: (u16, u16)) -> (u16, u16) {
		let (width, height) = total_size;
		
		// Calculate position based on binary tree structure
		let x_offset = (index % (1 << split_level)) as u16 * (width / (1 << split_level));
		let y_offset = (index / (1 << split_level)) as u16 * (height / (1 << split_level));
		
		(x_offset, y_offset)
	}
	
	/**
	 * Calculates binary tree size
	 * 
	 * Calculates the size of a pane in a binary tree layout
	 * based on its split level.
	 * 
	 * @param split_level - Current split level
	 * @param total_size - Total available size
	 * @return (u16, u16) - Calculated size
	 */
	fn calculate_binary_size(&self, split_level: u32, total_size: (u16, u16)) -> (u16, u16) {
		let (width, height) = total_size;
		
		// Calculate size based on split level
		let pane_width = width / (1 << split_level);
		let pane_height = height / (1 << split_level);
		
		(pane_width, pane_height)
	}
	
	/**
	 * Applies layout constraints
	 * 
	 * Applies layout constraints to ensure panes meet
	 * minimum size requirements and other constraints.
	 * 
	 * @param size - Original size
	 * @return (u16, u16) - Constrained size
	 */
	fn apply_constraints(&self, size: (u16, u16)) -> (u16, u16) {
		let (width, height) = size;
		
		// Apply minimum size constraints
		let constrained_width = width.max(self.constraints.min_width);
		let constrained_height = height.max(self.constraints.min_height);
		
		// Apply aspect ratio constraint if specified
		if let Some(aspect_ratio) = self.constraints.aspect_ratio {
			let current_ratio = constrained_width as f32 / constrained_height as f32;
			
			if current_ratio > aspect_ratio {
				// Too wide, reduce width
				let new_width = (constrained_height as f32 * aspect_ratio) as u16;
				(new_width, constrained_height)
			} else {
				// Too tall, reduce height
				let new_height = (constrained_width as f32 / aspect_ratio) as u16;
				(constrained_width, new_height)
			}
		} else {
			(constrained_width, constrained_height)
		}
	}
	
	/**
	 * Checks if layout meets constraints
	 * 
	 * @param size - Pane size to check
	 * @return bool - True if constraints are met
	 */
	fn check_constraints(&self, size: (u16, u16)) -> bool {
		let (width, height) = size;
		
		width >= self.constraints.min_width && height >= self.constraints.min_height
	}
	
	/**
	 * Updates layout tree
	 * 
	 * Updates the layout tree with new node information
	 * for tracking pane relationships and splits.
	 * 
	 * @param node_id - Node ID
	 * @param node - Node information
	 */
	pub fn update_layout_tree(&mut self, node_id: String, node: LayoutNode) {
		self.layout_tree.insert(node_id, node);
	}
	
	/**
	 * Gets layout tree
	 * 
	 * @return &HashMap<String, LayoutNode> - Layout tree
	 */
	pub fn get_layout_tree(&self) -> &HashMap<String, LayoutNode> {
		&self.layout_tree
	}
	
	/**
	 * Sets layout algorithm
	 * 
	 * @param algorithm - New layout algorithm
	 */
	pub fn set_algorithm(&mut self, algorithm: LayoutAlgorithm) {
		self.algorithm = algorithm;
	}
	
	/**
	 * Gets current algorithm
	 * 
	 * @return LayoutAlgorithm - Current layout algorithm
	 */
	pub fn get_algorithm(&self) -> LayoutAlgorithm {
		self.algorithm.clone()
	}
	
	/**
	 * Updates layout constraints
	 * 
	 * @param constraints - New layout constraints
	 */
	pub fn update_constraints(&mut self, constraints: LayoutConstraints) {
		self.constraints = constraints;
	}
	
	/**
	 * Gets layout constraints
	 * 
	 * @return &LayoutConstraints - Current layout constraints
	 */
	pub fn get_constraints(&self) -> &LayoutConstraints {
		&self.constraints
	}
}

/**
 * Layout utilities
 * 
 * Provides utility functions for layout calculations
 * and layout tree management.
 */
pub struct LayoutUtils;

	/**
	 * Calculates optimal grid dimensions
	 * 
	 * Calculates the optimal number of rows and columns
	 * for a grid layout based on the number of panes.
	 * 
	 * @param num_panes - Number of panes
	 * @return (u16, u16) - Optimal (rows, columns)
	 */
impl LayoutUtils {

	pub fn calculate_optimal_grid(num_panes: usize) -> (u16, u16) {
		if num_panes == 0 {
			return (0, 0);
		}
		
		let sqrt = (num_panes as f32).sqrt();
		let cols = sqrt.ceil() as u16;
		let rows = ((num_panes as f32) / cols as f32).ceil() as u16;
		
		(rows, cols)
	}
	
	/**
	 * Calculates split ratio
	 * 
	 * Calculates the optimal split ratio for dividing
	 * a pane based on content and usage patterns.
	 * 
	 * @param total_size - Total available size
	 * @param num_children - Number of child panes
	 * @return f32 - Split ratio (0.0 to 1.0)
	 */
	pub fn calculate_split_ratio(total_size: (u16, u16), num_children: usize) -> f32 {
		if num_children == 0 {
			return 0.5; // Default 50/50 split
		}
		
		// Calculate based on golden ratio for aesthetic splits
		let golden_ratio = 1.618;
		let ratio: f32 = 1.0 / golden_ratio;
		
		ratio.max(0.2).min(0.8) // Clamp between 20% and 80%
	}
	
	/**
	 * Validates layout constraints
	 * 
	 * Validates that layout constraints are reasonable
	 * and can be satisfied with the given total size.
	 * 
	 * @param constraints - Layout constraints
	 * @param total_size - Total available size
	 * @return bool - True if constraints are valid
	 */
	pub fn validate_constraints(constraints: &LayoutConstraints, total_size: (u16, u16)) -> bool {
		let (width, height) = total_size;
		
		// Check minimum size constraints
		if constraints.min_width > width || constraints.min_height > height {
			return false;
		}
		
		// Check spacing constraints
		if constraints.spacing > width || constraints.spacing > height {
			return false;
		}
		
		// Check aspect ratio constraints
		if let Some(aspect_ratio) = constraints.aspect_ratio {
			if aspect_ratio <= 0.0 {
				return false;
			}
		}
		
		true
	}
} 