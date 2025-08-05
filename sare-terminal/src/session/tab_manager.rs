/**
 * Tab manager for Sare terminal
 * 
 * This module provides tab management capabilities including multiple tabs
 * within windows, tab creation, switching, and management for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: tab_manager.rs
 * Description: Tab manager for multiple tabs within windows
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{
	TabMetadata, TabType, SessionMetadata,
};

/**
 * Tab manager
 * 
 * タブ管理の中心的なコンポーネントです。
 * ウィンドウ内の複数のタブの作成、切り替え、管理を
 * 担当します。
 * 
 * タブの作成、削除、切り替え、名前変更の各機能を提供し、
 * セッションごとのタブ管理を実現します
 */
pub struct TabManager {
	/// タブのメタデータ
	tabs: Arc<RwLock<HashMap<Uuid, TabMetadata>>>,
	/// セッションごとのタブリスト
	session_tabs: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
	/// アクティブなタブ
	active_tabs: Arc<RwLock<HashMap<Uuid, Uuid>>>, // session_id -> tab_id
	/// タブの順序
	tab_order: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>, // session_id -> tab_ids
}

impl TabManager {
	/**
	 * Creates a new tab manager
	 * 
	 * @return Result<TabManager> - New tab manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいタブマネージャーを作成する関数です
		 * 
		 * タブのメタデータ、セッションごとのタブリスト、
		 * アクティブなタブ、タブの順序を初期化します。
		 * 
		 * 各セッションに対してタブ管理を提供します
		 */
		
		Ok(Self {
			tabs: Arc::new(RwLock::new(HashMap::new())),
			session_tabs: Arc::new(RwLock::new(HashMap::new())),
			active_tabs: Arc::new(RwLock::new(HashMap::new())),
			tab_order: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Initializes the tab manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * タブマネージャーを初期化する関数です
		 * 
		 * 既存のタブデータを読み込み、タブマネージャーを
		 * 準備状態にします。
		 * 
		 * アプリケーション起動時に呼び出され、タブの
		 * 復旧準備を行います
		 */
		
		// Load persistent tab data from storage
		self.load_persistent_tab_data().await?;
		
		Ok(())
	}
	
	/**
	 * Loads persistent tab data from storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn load_persistent_tab_data(&self) -> Result<()> {
		/**
		 * 永続化されたタブデータをストレージから読み込む関数です
		 * 
		 * タブのメタデータ、セッションごとのタブリスト、
		 * アクティブなタブ、タブの順序を復元します。
		 * 
		 * アプリケーション起動時に既存のタブ状態を
		 * 復旧するために使用されます
		 */
		
		// Get user's home directory for tab data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let tab_data_file = home_dir.join(".sare_tabs.json");
		
		// Try to load existing tab data
		if let Ok(data) = std::fs::read_to_string(&tab_data_file) {
			if let Ok(tab_data) = serde_json::from_str::<serde_json::Value>(&data) {
				// Load tabs metadata
				if let Some(tabs_data) = tab_data.get("tabs") {
					if let Some(tabs_object) = tabs_data.as_object() {
						let mut tabs = self.tabs.write().await;
						for (tab_id_str, tab_metadata) in tabs_object {
							if let Ok(tab_id) = Uuid::parse_str(tab_id_str) {
								if let Ok(metadata) = serde_json::from_value::<TabMetadata>(tab_metadata.clone()) {
									tabs.insert(tab_id, metadata);
								}
							}
						}
					}
				}
				
				// Load session tabs mapping
				if let Some(session_tabs_data) = tab_data.get("session_tabs") {
					if let Some(session_tabs_object) = session_tabs_data.as_object() {
						let mut session_tabs = self.session_tabs.write().await;
						for (session_id_str, tab_ids_array) in session_tabs_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Some(tab_ids) = tab_ids_array.as_array() {
									let mut tab_ids_vec = Vec::new();
									for tab_id_value in tab_ids {
										if let Some(tab_id_str) = tab_id_value.as_str() {
											if let Ok(tab_id) = Uuid::parse_str(tab_id_str) {
												tab_ids_vec.push(tab_id);
											}
										}
									}
									session_tabs.insert(session_id, tab_ids_vec);
								}
							}
						}
					}
				}
				
				// Load active tabs mapping
				if let Some(active_tabs_data) = tab_data.get("active_tabs") {
					if let Some(active_tabs_object) = active_tabs_data.as_object() {
						let mut active_tabs = self.active_tabs.write().await;
						for (session_id_str, tab_id_str) in active_tabs_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Ok(tab_id) = Uuid::parse_str(tab_id_str.as_str().unwrap_or("")) {
									active_tabs.insert(session_id, tab_id);
								}
							}
						}
					}
				}
				
				// Load tab order mapping
				if let Some(tab_order_data) = tab_data.get("tab_order") {
					if let Some(tab_order_object) = tab_order_data.as_object() {
						let mut tab_order = self.tab_order.write().await;
						for (session_id_str, tab_ids_array) in tab_order_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Some(tab_ids) = tab_ids_array.as_array() {
									let mut tab_ids_vec = Vec::new();
									for tab_id_value in tab_ids {
										if let Some(tab_id_str) = tab_id_value.as_str() {
											if let Ok(tab_id) = Uuid::parse_str(tab_id_str) {
												tab_ids_vec.push(tab_id);
											}
										}
									}
									tab_order.insert(session_id, tab_ids_vec);
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
	 * Saves persistent tab data to storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn save_persistent_tab_data(&self) -> Result<()> {
		/**
		 * タブデータを永続化ストレージに保存する関数です
		 * 
		 * タブのメタデータ、セッションごとのタブリスト、
		 * アクティブなタブ、タブの順序を保存します。
		 * 
		 * アプリケーション終了時にタブ状態を
		 * 永続化するために使用されます
		 */
		
		// Get user's home directory for tab data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let tab_data_file = home_dir.join(".sare_tabs.json");
		
		// Prepare tab data for serialization
		let mut tab_data = serde_json::Map::new();
		
		// Serialize tabs metadata
		let tabs = self.tabs.read().await;
		let mut tabs_object = serde_json::Map::new();
		for (tab_id, metadata) in tabs.iter() {
			tabs_object.insert(tab_id.to_string(), serde_json::to_value(metadata)?);
		}
		tab_data.insert("tabs".to_string(), serde_json::Value::Object(tabs_object));
		
		// Serialize session tabs mapping
		let session_tabs = self.session_tabs.read().await;
		let mut session_tabs_object = serde_json::Map::new();
		for (session_id, tab_ids) in session_tabs.iter() {
			let tab_ids_array: Vec<serde_json::Value> = tab_ids.iter()
				.map(|id| serde_json::Value::String(id.to_string()))
				.collect();
			session_tabs_object.insert(session_id.to_string(), serde_json::Value::Array(tab_ids_array));
		}
		tab_data.insert("session_tabs".to_string(), serde_json::Value::Object(session_tabs_object));
		
		// Serialize active tabs mapping
		let active_tabs = self.active_tabs.read().await;
		let mut active_tabs_object = serde_json::Map::new();
		for (session_id, tab_id) in active_tabs.iter() {
			active_tabs_object.insert(session_id.to_string(), serde_json::Value::String(tab_id.to_string()));
		}
		tab_data.insert("active_tabs".to_string(), serde_json::Value::Object(active_tabs_object));
		
		// Serialize tab order mapping
		let tab_order = self.tab_order.read().await;
		let mut tab_order_object = serde_json::Map::new();
		for (session_id, tab_ids) in tab_order.iter() {
			let tab_ids_array: Vec<serde_json::Value> = tab_ids.iter()
				.map(|id| serde_json::Value::String(id.to_string()))
				.collect();
			tab_order_object.insert(session_id.to_string(), serde_json::Value::Array(tab_ids_array));
		}
		tab_data.insert("tab_order".to_string(), serde_json::Value::Object(tab_order_object));
		
		// Write tab data to file
		let tab_data_json = serde_json::to_string_pretty(&serde_json::Value::Object(tab_data))?;
		std::fs::write(&tab_data_file, tab_data_json)?;
		
		Ok(())
	}
	
	/**
	 * Creates a new tab
	 * 
	 * @param name - Tab name
	 * @param tab_type - Tab type
	 * @param session_id - Parent session ID
	 * @return Result<TabMetadata> - Created tab metadata
	 */
	pub async fn create_tab(
		&self,
		name: String,
		tab_type: TabType,
		session_id: Uuid,
	) -> Result<TabMetadata> {
		/**
		 * 新しいタブを作成する関数です
		 * 
		 * 指定された名前、タイプ、親セッションIDでタブを作成し、
		 * セッションのタブリストに追加します。
		 * 
		 * タブIDは自動生成され、作成日時と更新日時が
		 * 自動的に設定されます
		 */
		
		let tab_id = Uuid::new_v4();
		let now = Utc::now();
		
		let metadata = TabMetadata {
			id: tab_id,
			name: name.clone(),
			tab_type,
			session_id,
			created_at: now,
			updated_at: now,
		};
		
		// タブメタデータに追加
		{
			let mut tabs = self.tabs.write().await;
			tabs.insert(tab_id, metadata.clone());
		}
		
		// セッションのタブリストに追加
		{
			let mut session_tabs = self.session_tabs.write().await;
			let tabs = session_tabs.entry(session_id).or_insert_with(Vec::new);
			tabs.push(tab_id);
		}
		
		// タブの順序に追加
		{
			let mut tab_order = self.tab_order.write().await;
			let order = tab_order.entry(session_id).or_insert_with(Vec::new);
			order.push(tab_id);
		}
		
		// 最初のタブの場合はアクティブに設定
		{
			let mut active_tabs = self.active_tabs.write().await;
			if !active_tabs.contains_key(&session_id) {
				active_tabs.insert(session_id, tab_id);
			}
		}
		
		Ok(metadata)
	}
	
	/**
	 * Gets tab by ID
	 * 
	 * @param tab_id - Tab ID
	 * @return Result<Option<TabMetadata>> - Tab metadata if found
	 */
	pub async fn get_tab(&self, tab_id: Uuid) -> Result<Option<TabMetadata>> {
		let tabs = self.tabs.read().await;
		Ok(tabs.get(&tab_id).cloned())
	}
	
	/**
	 * Gets tabs for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Vec<TabMetadata>> - List of tabs for the session
	 */
	pub async fn get_session_tabs(&self, session_id: Uuid) -> Result<Vec<TabMetadata>> {
		/**
		 * セッションのタブを取得する関数です
		 * 
		 * 指定されたセッションに属するすべてのタブを
		 * 順序付きで返します。
		 * 
		 * タブは作成順でソートされます
		 */
		
		let session_tabs = self.session_tabs.read().await;
		if let Some(tab_ids) = session_tabs.get(&session_id) {
			let tabs = self.tabs.read().await;
			let mut tab_metadata: Vec<TabMetadata> = tab_ids
				.iter()
				.filter_map(|id| tabs.get(id).cloned())
				.collect();
			
			// 作成日時でソート
			tab_metadata.sort_by(|a, b| a.created_at.cmp(&b.created_at));
			
			return Ok(tab_metadata);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets active tab for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Option<TabMetadata>> - Active tab metadata if found
	 */
	pub async fn get_active_tab(&self, session_id: Uuid) -> Result<Option<TabMetadata>> {
		/**
		 * セッションのアクティブなタブを取得する関数です
		 * 
		 * 指定されたセッションで現在アクティブなタブの
		 * メタデータを返します。
		 * 
		 * アクティブなタブがない場合は None を返します
		 */
		
		let active_tabs = self.active_tabs.read().await;
		if let Some(tab_id) = active_tabs.get(&session_id) {
			self.get_tab(*tab_id).await
		} else {
			Ok(None)
		}
	}
	
	/**
	 * Sets active tab for a session
	 * 
	 * @param session_id - Session ID
	 * @param tab_id - Tab ID to activate
	 * @return Result<()> - Success or error
	 */
	pub async fn set_active_tab(&self, session_id: Uuid, tab_id: Uuid) -> Result<()> {
		/**
		 * セッションのアクティブなタブを設定する関数です
		 * 
		 * 指定されたセッションで指定されたタブを
		 * アクティブにします。
		 * 
		 * タブが存在し、セッションに属していることを
		 * 確認してから設定します
		 */
		
		// タブが存在することを確認
		{
			let tabs = self.tabs.read().await;
			if !tabs.contains_key(&tab_id) {
				return Err(anyhow::anyhow!("Tab not found"));
			}
		}
		
		// セッションにタブが属していることを確認
		{
			let session_tabs = self.session_tabs.read().await;
			if let Some(tab_ids) = session_tabs.get(&session_id) {
				if !tab_ids.contains(&tab_id) {
					return Err(anyhow::anyhow!("Tab does not belong to session"));
				}
			} else {
				return Err(anyhow::anyhow!("Session not found"));
			}
		}
		
		// アクティブタブを設定
		{
			let mut active_tabs = self.active_tabs.write().await;
			active_tabs.insert(session_id, tab_id);
		}
		
		Ok(())
	}
	
	/**
	 * Renames a tab
	 * 
	 * @param tab_id - Tab ID to rename
	 * @param new_name - New tab name
	 * @return Result<()> - Success or error
	 */
	pub async fn rename_tab(&self, tab_id: Uuid, new_name: String) -> Result<()> {
		/**
		 * タブの名前を変更する関数です
		 * 
		 * 指定されたタブの名前を新しい名前に変更し、
		 * 更新日時を更新します。
		 * 
		 * タブが存在することを確認してから名前を変更します
		 */
		
		let mut tabs = self.tabs.write().await;
		if let Some(metadata) = tabs.get_mut(&tab_id) {
			metadata.name = new_name;
			metadata.updated_at = Utc::now();
		} else {
			return Err(anyhow::anyhow!("Tab not found"));
		}
		
		Ok(())
	}
	
	/**
	 * Deletes a tab
	 * 
	 * @param tab_id - Tab ID to delete
	 * @return Result<()> - Success or error
	 */
	pub async fn delete_tab(&self, tab_id: Uuid) -> Result<()> {
		/**
		 * タブを削除する関数です
		 * 
		 * 指定されたタブを削除し、セッションのタブリストと
		 * タブの順序からも削除します。
		 * 
		 * 削除されたタブがアクティブだった場合は、次のタブを
		 * アクティブにします
		 */
		
		// タブのメタデータを取得
		let session_id = {
			let tabs = self.tabs.read().await;
			if let Some(metadata) = tabs.get(&tab_id) {
				metadata.session_id
			} else {
				return Err(anyhow::anyhow!("Tab not found"));
			}
		};
		
		// タブメタデータから削除
		{
			let mut tabs = self.tabs.write().await;
			tabs.remove(&tab_id);
		}
		
		// セッションのタブリストから削除
		{
			let mut session_tabs = self.session_tabs.write().await;
			if let Some(tabs) = session_tabs.get_mut(&session_id) {
				tabs.retain(|&id| id != tab_id);
			}
		}
		
		// タブの順序から削除
		{
			let mut tab_order = self.tab_order.write().await;
			if let Some(order) = tab_order.get_mut(&session_id) {
				order.retain(|&id| id != tab_id);
			}
		}
		
		// 削除されたタブがアクティブだった場合、次のタブをアクティブに
		{
			let active_tabs = self.active_tabs.read().await;
			if let Some(&active_tab_id) = active_tabs.get(&session_id) {
				if active_tab_id == tab_id {
					// 次のタブをアクティブに設定
					let session_tabs = self.session_tabs.read().await;
					if let Some(tab_ids) = session_tabs.get(&session_id) {
						if let Some(&next_tab_id) = tab_ids.first() {
							let mut active_tabs = self.active_tabs.write().await;
							active_tabs.insert(session_id, next_tab_id);
						}
					}
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Moves a tab to a new position
	 * 
	 * @param tab_id - Tab ID to move
	 * @param new_index - New position index
	 * @return Result<()> - Success or error
	 */
	pub async fn move_tab(&self, tab_id: Uuid, new_index: usize) -> Result<()> {
		/**
		 * タブを新しい位置に移動する関数です
		 * 
		 * 指定されたタブを新しいインデックス位置に移動し、
		 * タブの順序を更新します。
		 * 
		 * インデックスが範囲外の場合は最後に移動します
		 */
		
		// タブのセッションIDを取得
		let session_id = {
			let tabs = self.tabs.read().await;
			if let Some(metadata) = tabs.get(&tab_id) {
				metadata.session_id
			} else {
				return Err(anyhow::anyhow!("Tab not found"));
			}
		};
		
		// タブの順序を更新
		{
			let mut tab_order = self.tab_order.write().await;
			if let Some(order) = tab_order.get_mut(&session_id) {
				// 現在の位置を削除
				order.retain(|&id| id != tab_id);
				
				// 新しい位置に挿入
				let actual_index = new_index.min(order.len());
				order.insert(actual_index, tab_id);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets tab count for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<usize> - Number of tabs
	 */
	pub async fn get_session_tab_count(&self, session_id: Uuid) -> Result<usize> {
		let session_tabs = self.session_tabs.read().await;
		Ok(session_tabs.get(&session_id).map(|tabs| tabs.len()).unwrap_or(0))
	}
	
	/**
	 * Gets total tab count
	 * 
	 * @return Result<usize> - Total number of tabs
	 */
	pub async fn get_tab_count(&self) -> Result<usize> {
		let tabs = self.tabs.read().await;
		Ok(tabs.len())
	}
	
	/**
	 * Shuts down the tab manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * タブマネージャーをシャットダウンする関数です
		 * 
		 * すべてのタブデータをクリアし、リソースを解放します。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// Save persistent tab data before clearing
		self.save_persistent_tab_data().await?;
		
		// すべてのデータをクリア
		{
			let mut tabs = self.tabs.write().await;
			tabs.clear();
		}
		
		{
			let mut session_tabs = self.session_tabs.write().await;
			session_tabs.clear();
		}
		
		{
			let mut active_tabs = self.active_tabs.write().await;
			active_tabs.clear();
		}
		
		{
			let mut tab_order = self.tab_order.write().await;
			tab_order.clear();
		}
		
		Ok(())
	}
	
	/**
	 * Gets next tab in session
	 * 
	 * @param session_id - Session ID
	 * @param current_tab_id - Current tab ID
	 * @return Result<Option<TabMetadata>> - Next tab metadata if found
	 */
	pub async fn get_next_tab(&self, session_id: Uuid, current_tab_id: Uuid) -> Result<Option<TabMetadata>> {
		/**
		 * セッションの次のタブを取得する関数です
		 * 
		 * 指定されたセッションで現在のタブの次のタブを
		 * 取得します。
		 * 
		 * 最後のタブの場合は最初のタブを返します
		 */
		
		let tab_order = self.tab_order.read().await;
		if let Some(order) = tab_order.get(&session_id) {
			if let Some(current_index) = order.iter().position(|&id| id == current_tab_id) {
				let next_index = (current_index + 1) % order.len();
				let next_tab_id = order[next_index];
				return self.get_tab(next_tab_id).await;
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Gets previous tab in session
	 * 
	 * @param session_id - Session ID
	 * @param current_tab_id - Current tab ID
	 * @return Result<Option<TabMetadata>> - Previous tab metadata if found
	 */
	pub async fn get_previous_tab(&self, session_id: Uuid, current_tab_id: Uuid) -> Result<Option<TabMetadata>> {
		/**
		 * セッションの前のタブを取得する関数です
		 * 
		 * 指定されたセッションで現在のタブの前のタブを
		 * 取得します。
		 * 
		 * 最初のタブの場合は最後のタブを返します
		 */
		
		let tab_order = self.tab_order.read().await;
		if let Some(order) = tab_order.get(&session_id) {
			if let Some(current_index) = order.iter().position(|&id| id == current_tab_id) {
				let prev_index = if current_index == 0 {
					order.len() - 1
				} else {
					current_index - 1
				};
				let prev_tab_id = order[prev_index];
				return self.get_tab(prev_tab_id).await;
			}
		}
		
		Ok(None)
	}
} 