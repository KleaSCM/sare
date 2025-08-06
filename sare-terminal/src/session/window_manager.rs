/**
 * Window manager for Sare terminal
 * 
 * This module provides window management capabilities including multiple windows,
 * window splitting, and window management for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: window_manager.rs
 * Description: Window manager for multiple windows and window splitting
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{
	WindowMetadata, WindowType, WindowGeometry, SessionMetadata,
};

/**
 * Window manager
 * 
 * ウィンドウ管理の中心的なコンポーネントです。
 * 複数のウィンドウの作成、分割、管理を担当します。
 * 
 * ウィンドウの作成、削除、分割、リサイズの各機能を提供し、
 * セッションごとのウィンドウ管理を実現します
 */
#[derive(Debug)]
pub struct WindowManager {
	/// ウィンドウのメタデータ
	windows: Arc<RwLock<HashMap<Uuid, WindowMetadata>>>,
	/// セッションごとのウィンドウリスト
	session_windows: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
	/// アクティブなウィンドウ
	active_windows: Arc<RwLock<HashMap<Uuid, Uuid>>>, // session_id -> window_id
	/// ウィンドウの階層構造
	window_hierarchy: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>, // parent_window_id -> child_window_ids
}

impl WindowManager {
	/**
	 * Creates a new window manager
	 * 
	 * @return Result<WindowManager> - New window manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいウィンドウマネージャーを作成する関数です
		 * 
		 * ウィンドウのメタデータ、セッションごとのウィンドウリスト、
		 * アクティブなウィンドウ、ウィンドウの階層構造を初期化します。
		 * 
		 * 各セッションに対してウィンドウ管理を提供します
		 */
		
		Ok(Self {
			windows: Arc::new(RwLock::new(HashMap::new())),
			session_windows: Arc::new(RwLock::new(HashMap::new())),
			active_windows: Arc::new(RwLock::new(HashMap::new())),
			window_hierarchy: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Initializes the window manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * ウィンドウマネージャーを初期化する関数です
		 * 
		 * 既存のウィンドウデータを読み込み、ウィンドウマネージャーを
		 * 準備状態にします。
		 * 
		 * アプリケーション起動時に呼び出され、ウィンドウの
		 * 復旧準備を行います
		 */
		
		// Load persistent window data from storage
		self.load_persistent_window_data().await?;
		
		Ok(())
	}
	
	/**
	 * Loads persistent window data from storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn load_persistent_window_data(&self) -> Result<()> {
		/**
		 * 永続化されたウィンドウデータをストレージから読み込む関数です
		 * 
		 * ウィンドウのメタデータ、セッションごとのウィンドウリスト、
		 * アクティブなウィンドウ、ウィンドウの階層構造を復元します。
		 * 
		 * アプリケーション起動時に既存のウィンドウ状態を
		 * 復旧するために使用されます
		 */
		
		// Get user's home directory for window data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let window_data_file = home_dir.join(".sare_windows.json");
		
		// Try to load existing window data
		if let Ok(data) = tokio::fs::read_to_string(&window_data_file).await {
			if let Ok(window_data) = serde_json::from_str::<serde_json::Value>(&data) {
				// Load windows metadata
				if let Some(windows_data) = window_data.get("windows") {
					if let Some(windows_object) = windows_data.as_object() {
						let mut windows = self.windows.write().await;
						for (window_id_str, window_metadata) in windows_object {
							if let Ok(window_id) = Uuid::parse_str(window_id_str) {
								if let Ok(metadata) = serde_json::from_value::<WindowMetadata>(window_metadata.clone()) {
									windows.insert(window_id, metadata);
								}
							}
						}
					}
				}
				
				// Load session windows mapping
				if let Some(session_windows_data) = window_data.get("session_windows") {
					if let Some(session_windows_object) = session_windows_data.as_object() {
						let mut session_windows = self.session_windows.write().await;
						for (session_id_str, window_ids_array) in session_windows_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Some(window_ids) = window_ids_array.as_array() {
									let mut window_ids_vec = Vec::new();
									for window_id_value in window_ids {
										if let Some(window_id_str) = window_id_value.as_str() {
											if let Ok(window_id) = Uuid::parse_str(window_id_str) {
												window_ids_vec.push(window_id);
											}
										}
									}
									session_windows.insert(session_id, window_ids_vec);
								}
							}
						}
					}
				}
				
				// Load active windows mapping
				if let Some(active_windows_data) = window_data.get("active_windows") {
					if let Some(active_windows_object) = active_windows_data.as_object() {
						let mut active_windows = self.active_windows.write().await;
						for (session_id_str, window_id_str) in active_windows_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Ok(window_id) = Uuid::parse_str(window_id_str.as_str().unwrap_or("")) {
									active_windows.insert(session_id, window_id);
								}
							}
						}
					}
				}
				
				// Load window hierarchy mapping
				if let Some(window_hierarchy_data) = window_data.get("window_hierarchy") {
					if let Some(window_hierarchy_object) = window_hierarchy_data.as_object() {
						let mut window_hierarchy = self.window_hierarchy.write().await;
						for (parent_window_id_str, child_window_ids_array) in window_hierarchy_object {
							if let Ok(parent_window_id) = Uuid::parse_str(parent_window_id_str) {
								if let Some(child_window_ids) = child_window_ids_array.as_array() {
									let mut child_window_ids_vec = Vec::new();
									for child_window_id_value in child_window_ids {
										if let Some(child_window_id_str) = child_window_id_value.as_str() {
											if let Ok(child_window_id) = Uuid::parse_str(child_window_id_str) {
												child_window_ids_vec.push(child_window_id);
											}
										}
									}
									window_hierarchy.insert(parent_window_id, child_window_ids_vec);
								}
							}
						}
					}
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Saves persistent window data to storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn save_persistent_window_data(&self) -> Result<()> {
		/**
		 * ウィンドウデータを永続化ストレージに保存する関数です
		 * 
		 * ウィンドウのメタデータ、セッションごとのウィンドウリスト、
		 * アクティブなウィンドウ、ウィンドウの階層構造を保存します。
		 * 
		 * アプリケーション終了時にウィンドウ状態を
		 * 永続化するために使用されます
		 */
		
		// Get user's home directory for window data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let window_data_file = home_dir.join(".sare_windows.json");
		
		// Prepare window data for serialization
		let mut window_data = serde_json::Map::new();
		
		// Serialize windows metadata
		let windows = self.windows.read().await;
		let mut windows_object = serde_json::Map::new();
		for (window_id, metadata) in windows.iter() {
			windows_object.insert(window_id.to_string(), serde_json::to_value(metadata)?);
		}
		window_data.insert("windows".to_string(), serde_json::Value::Object(windows_object));
		
		// Serialize session windows mapping
		let session_windows = self.session_windows.read().await;
		let mut session_windows_object = serde_json::Map::new();
		for (session_id, window_ids) in session_windows.iter() {
			let window_ids_array: Vec<serde_json::Value> = window_ids.iter()
				.map(|id| serde_json::Value::String(id.to_string()))
				.collect();
			session_windows_object.insert(session_id.to_string(), serde_json::Value::Array(window_ids_array));
		}
		window_data.insert("session_windows".to_string(), serde_json::Value::Object(session_windows_object));
		
		// Serialize active windows mapping
		let active_windows = self.active_windows.read().await;
		let mut active_windows_object = serde_json::Map::new();
		for (session_id, window_id) in active_windows.iter() {
			active_windows_object.insert(session_id.to_string(), serde_json::Value::String(window_id.to_string()));
		}
		window_data.insert("active_windows".to_string(), serde_json::Value::Object(active_windows_object));
		
		// Serialize window hierarchy mapping
		let window_hierarchy = self.window_hierarchy.read().await;
		let mut window_hierarchy_object = serde_json::Map::new();
		for (parent_window_id, child_window_ids) in window_hierarchy.iter() {
			let child_window_ids_array: Vec<serde_json::Value> = child_window_ids.iter()
				.map(|id| serde_json::Value::String(id.to_string()))
				.collect();
			window_hierarchy_object.insert(parent_window_id.to_string(), serde_json::Value::Array(child_window_ids_array));
		}
		window_data.insert("window_hierarchy".to_string(), serde_json::Value::Object(window_hierarchy_object));
		
		// Write window data to file
		let window_data_json = serde_json::to_string_pretty(&serde_json::Value::Object(window_data))?;
		tokio::fs::write(&window_data_file, window_data_json).await?;
		
		Ok(())
	}
	
	/**
	 * Creates a new window
	 * 
	 * @param name - Window name
	 * @param window_type - Window type
	 * @param session_id - Parent session ID
	 * @param tab_id - Parent tab ID (optional)
	 * @param geometry - Window geometry
	 * @return Result<WindowMetadata> - Created window metadata
	 */
	pub async fn create_window(
		&self,
		name: String,
		window_type: WindowType,
		session_id: Uuid,
		tab_id: Option<Uuid>,
		geometry: WindowGeometry,
	) -> Result<WindowMetadata> {
		/**
		 * 新しいウィンドウを作成する関数です
		 * 
		 * 指定された名前、タイプ、親セッションID、タブID、ジオメトリで
		 * ウィンドウを作成し、セッションのウィンドウリストに追加します。
		 * 
		 * ウィンドウIDは自動生成され、作成日時と更新日時が
		 * 自動的に設定されます
		 */
		
		let window_id = Uuid::new_v4();
		let now = Utc::now();
		
		let metadata = WindowMetadata {
			id: window_id,
			name: name.clone(),
			window_type,
			session_id,
			tab_id,
			geometry,
			created_at: now,
			updated_at: now,
		};
		
		// ウィンドウメタデータに追加
		{
			let mut windows = self.windows.write().await;
			windows.insert(window_id, metadata.clone());
		}
		
		// セッションのウィンドウリストに追加
		{
			let mut session_windows = self.session_windows.write().await;
			let windows = session_windows.entry(session_id).or_insert_with(Vec::new);
			windows.push(window_id);
		}
		
		// 最初のウィンドウの場合はアクティブに設定
		{
			let mut active_windows = self.active_windows.write().await;
			if !active_windows.contains_key(&session_id) {
				active_windows.insert(session_id, window_id);
			}
		}
		
		Ok(metadata)
	}
	
	/**
	 * Gets window by ID
	 * 
	 * @param window_id - Window ID
	 * @return Result<Option<WindowMetadata>> - Window metadata if found
	 */
	pub async fn get_window(&self, window_id: Uuid) -> Result<Option<WindowMetadata>> {
		let windows = self.windows.read().await;
		Ok(windows.get(&window_id).cloned())
	}
	
	/**
	 * Gets windows for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Vec<WindowMetadata>> - List of windows for the session
	 */
	pub async fn get_session_windows(&self, session_id: Uuid) -> Result<Vec<WindowMetadata>> {
		/**
		 * セッションのウィンドウを取得する関数です
		 * 
		 * 指定されたセッションに属するすべてのウィンドウを
		 * 作成順で返します。
		 * 
		 * ウィンドウは作成日時の順でソートされます
		 */
		
		let session_windows = self.session_windows.read().await;
		if let Some(window_ids) = session_windows.get(&session_id) {
			let windows = self.windows.read().await;
			let mut window_metadata: Vec<WindowMetadata> = window_ids
				.iter()
				.filter_map(|id| windows.get(id).cloned())
				.collect();
			
			// 作成日時でソート
			window_metadata.sort_by(|a, b| a.created_at.cmp(&b.created_at));
			
			return Ok(window_metadata);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets active window for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Option<WindowMetadata>> - Active window metadata if found
	 */
	pub async fn get_active_window(&self, session_id: Uuid) -> Result<Option<WindowMetadata>> {
		/**
		 * セッションのアクティブなウィンドウを取得する関数です
		 * 
		 * 指定されたセッションで現在アクティブなウィンドウの
		 * メタデータを返します。
		 * 
		 * アクティブなウィンドウがない場合は None を返します
		 */
		
		let active_windows = self.active_windows.read().await;
		if let Some(window_id) = active_windows.get(&session_id) {
			self.get_window(*window_id).await
		} else {
			Ok(None)
		}
	}
	
	/**
	 * Sets active window for a session
	 * 
	 * @param session_id - Session ID
	 * @param window_id - Window ID to activate
	 * @return Result<()> - Success or error
	 */
	pub async fn set_active_window(&self, session_id: Uuid, window_id: Uuid) -> Result<()> {
		/**
		 * セッションのアクティブなウィンドウを設定する関数です
		 * 
		 * 指定されたセッションで指定されたウィンドウを
		 * アクティブにします。
		 * 
		 * ウィンドウが存在し、セッションに属していることを
		 * 確認してから設定します
		 */
		
		// ウィンドウが存在することを確認
		{
			let windows = self.windows.read().await;
			if !windows.contains_key(&window_id) {
				return Err(anyhow::anyhow!("Window not found"));
			}
		}
		
		// セッションにウィンドウが属していることを確認
		{
			let session_windows = self.session_windows.read().await;
			if let Some(window_ids) = session_windows.get(&session_id) {
				if !window_ids.contains(&window_id) {
					return Err(anyhow::anyhow!("Window does not belong to session"));
				}
			} else {
				return Err(anyhow::anyhow!("Session not found"));
			}
		}
		
		// アクティブウィンドウを設定
		{
			let mut active_windows = self.active_windows.write().await;
			active_windows.insert(session_id, window_id);
		}
		
		Ok(())
	}
	
	/**
	 * Splits a window
	 * 
	 * @param parent_window_id - Parent window ID
	 * @param split_type - Split type (horizontal or vertical)
	 * @param new_window_name - Name for the new window
	 * @return Result<WindowMetadata> - Created window metadata
	 */
	pub async fn split_window(
		&self,
		parent_window_id: Uuid,
		split_type: SplitType,
		new_window_name: String,
	) -> Result<WindowMetadata> {
		/**
		 * ウィンドウを分割する関数です
		 * 
		 * 指定された親ウィンドウを分割し、新しいウィンドウを
		 * 作成します。
		 * 
		 * 水平分割または垂直分割を選択でき、親ウィンドウの
		 * ジオメトリを調整します
		 */
		
		// 親ウィンドウを取得
		let parent_metadata = {
			let windows = self.windows.read().await;
			if let Some(metadata) = windows.get(&parent_window_id) {
				metadata.clone()
			} else {
				return Err(anyhow::anyhow!("Parent window not found"));
			}
		};
		
		// 新しいウィンドウのジオメトリを計算
		let new_geometry = self.calculate_split_geometry(&parent_metadata.geometry, split_type.clone());
		
		// 親ウィンドウのジオメトリを調整
		{
			let mut windows = self.windows.write().await;
			if let Some(metadata) = windows.get_mut(&parent_window_id) {
				metadata.geometry = self.adjust_parent_geometry(&parent_metadata.geometry, split_type.clone());
				metadata.updated_at = Utc::now();
			}
		}
		
		// 新しいウィンドウを作成
		let new_window = self.create_window(
			new_window_name,
			WindowType::Split,
			parent_metadata.session_id,
			parent_metadata.tab_id,
			new_geometry,
		).await?;
		
		// 階層構造に追加
		{
			let mut hierarchy = self.window_hierarchy.write().await;
			let children = hierarchy.entry(parent_window_id).or_insert_with(Vec::new);
			children.push(new_window.id);
		}
		
		Ok(new_window)
	}
	
	/**
	 * Resizes a window
	 * 
	 * @param window_id - Window ID to resize
	 * @param new_geometry - New window geometry
	 * @return Result<()> - Success or error
	 */
	pub async fn resize_window(&self, window_id: Uuid, new_geometry: WindowGeometry) -> Result<()> {
		/**
		 * ウィンドウのサイズを変更する関数です
		 * 
		 * 指定されたウィンドウのジオメトリを新しい値に
		 * 変更し、更新日時を更新します。
		 * 
		 * ウィンドウが存在することを確認してからサイズを変更します
		 */
		
		let mut windows = self.windows.write().await;
		if let Some(metadata) = windows.get_mut(&window_id) {
			metadata.geometry = new_geometry;
			metadata.updated_at = Utc::now();
		} else {
			return Err(anyhow::anyhow!("Window not found"));
		}
		
		Ok(())
	}
	
	/**
	 * Deletes a window
	 * 
	 * @param window_id - Window ID to delete
	 * @return Result<()> - Success or error
	 */
	pub async fn delete_window(&self, window_id: Uuid) -> Result<()> {
		/**
		 * ウィンドウを削除する関数です
		 * 
		 * 指定されたウィンドウを削除し、セッションのウィンドウリストと
		 * 階層構造からも削除します。
		 * 
		 * 削除されたウィンドウがアクティブだった場合は、次のウィンドウを
		 * アクティブにします
		 */
		
		// ウィンドウのメタデータを取得
		let session_id = {
			let windows = self.windows.read().await;
			if let Some(metadata) = windows.get(&window_id) {
				metadata.session_id
			} else {
				return Err(anyhow::anyhow!("Window not found"));
			}
		};
		
		// 子ウィンドウも削除
		{
			let hierarchy = self.window_hierarchy.read().await;
			if let Some(children) = hierarchy.get(&window_id) {
				for &child_id in children {
					Box::pin(self.delete_window(child_id)).await?;
				}
			}
		}
		
		// ウィンドウメタデータから削除
		{
			let mut windows = self.windows.write().await;
			windows.remove(&window_id);
		}
		
		// セッションのウィンドウリストから削除
		{
			let mut session_windows = self.session_windows.write().await;
			if let Some(windows) = session_windows.get_mut(&session_id) {
				windows.retain(|&id| id != window_id);
			}
		}
		
		// 階層構造から削除
		{
			let mut hierarchy = self.window_hierarchy.write().await;
			hierarchy.remove(&window_id);
			// 親からも削除
			for (parent_id, children) in hierarchy.iter_mut() {
				children.retain(|&id| id != window_id);
			}
		}
		
		// 削除されたウィンドウがアクティブだった場合、次のウィンドウをアクティブに
		{
			let active_windows = self.active_windows.read().await;
			if let Some(&active_window_id) = active_windows.get(&session_id) {
				if active_window_id == window_id {
					// 次のウィンドウをアクティブに設定
					let session_windows = self.session_windows.read().await;
					if let Some(window_ids) = session_windows.get(&session_id) {
						if let Some(&next_window_id) = window_ids.first() {
							let mut active_windows = self.active_windows.write().await;
							active_windows.insert(session_id, next_window_id);
						}
					}
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets window count for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<usize> - Number of windows
	 */
	pub async fn get_session_window_count(&self, session_id: Uuid) -> Result<usize> {
		let session_windows = self.session_windows.read().await;
		Ok(session_windows.get(&session_id).map(|windows| windows.len()).unwrap_or(0))
	}
	
	/**
	 * Gets total window count
	 * 
	 * @return Result<usize> - Total number of windows
	 */
	pub async fn get_window_count(&self) -> Result<usize> {
		let windows = self.windows.read().await;
		Ok(windows.len())
	}
	
	/**
	 * Shuts down the window manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * ウィンドウマネージャーをシャットダウンする関数です
		 * 
		 * すべてのウィンドウデータをクリアし、リソースを解放します。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// Save persistent window data before clearing
		self.save_persistent_window_data().await?;
		
		// すべてのデータをクリア
		{
			let mut windows = self.windows.write().await;
			windows.clear();
		}
		
		{
			let mut session_windows = self.session_windows.write().await;
			session_windows.clear();
		}
		
		{
			let mut active_windows = self.active_windows.write().await;
			active_windows.clear();
		}
		
		{
			let mut window_hierarchy = self.window_hierarchy.write().await;
			window_hierarchy.clear();
		}
		
		Ok(())
	}
	
	/**
	 * Calculates split geometry
	 * 
	 * @param parent_geometry - Parent window geometry
	 * @param split_type - Split type
	 * @return WindowGeometry - Calculated geometry
	 */
	fn calculate_split_geometry(&self, parent_geometry: &WindowGeometry, split_type: SplitType) -> WindowGeometry {
		match split_type {
			SplitType::Horizontal => WindowGeometry {
				x: parent_geometry.x,
				y: parent_geometry.y + (parent_geometry.height as i32 / 2),
				width: parent_geometry.width,
				height: parent_geometry.height / 2,
			},
			SplitType::Vertical => WindowGeometry {
				x: parent_geometry.x + (parent_geometry.width as i32 / 2),
				y: parent_geometry.y,
				width: parent_geometry.width / 2,
				height: parent_geometry.height,
			},
		}
	}
	
	/**
	 * Adjusts parent geometry after split
	 * 
	 * @param parent_geometry - Parent window geometry
	 * @param split_type - Split type
	 * @return WindowGeometry - Adjusted geometry
	 */
	fn adjust_parent_geometry(&self, parent_geometry: &WindowGeometry, split_type: SplitType) -> WindowGeometry {
		match split_type {
			SplitType::Horizontal => WindowGeometry {
				x: parent_geometry.x,
				y: parent_geometry.y,
				width: parent_geometry.width,
				height: parent_geometry.height / 2,
			},
			SplitType::Vertical => WindowGeometry {
				x: parent_geometry.x,
				y: parent_geometry.y,
				width: parent_geometry.width / 2,
				height: parent_geometry.height,
			},
		}
	}
}

/**
 * Split type
 * 
 * ウィンドウ分割の種類を定義します
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SplitType {
	/// 水平分割
	Horizontal,
	/// 垂直分割
	Vertical,
} 