/**
 * Session store for Sare terminal
 * 
 * This module provides session persistence and recovery capabilities for the
 * Sare terminal, including session storage, loading, and management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: session_store.rs
 * Description: Session store for persistence and recovery
 */

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{
	SessionMetadata, SessionType, SessionState, SharingConfig, SharingPermission,
};

/**
 * Session store
 * 
 * セッションの永続化と復旧を担当するコンポーネントです。
 * ファイルシステムにセッションデータを保存し、アプリケーション
 * 再起動時にセッションを復旧します。
 * 
 * セッションの保存、読み込み、削除の各機能を提供し、
 * デタッチされたセッションの永続化を実現します
 */
pub struct SessionStore {
	/// セッションデータの保存ディレクトリ
	storage_dir: PathBuf,
	/// セッションデータのキャッシュ
	session_cache: Arc<RwLock<HashMap<Uuid, SessionMetadata>>>,
	/// ストアの初期化状態
	initialized: Arc<RwLock<bool>>,
}

impl SessionStore {
	/**
	 * Creates a new session store
	 * 
	 * @return Result<SessionStore> - New session store instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいセッションストアを作成する関数です
		 * 
		 * デフォルトのストレージディレクトリを設定し、
		 * セッションキャッシュを初期化します。
		 * 
		 * ストレージディレクトリは ~/.sare/sessions に
		 * 設定されます
		 */
		
		let storage_dir = dirs::home_dir()
			.ok_or_else(|| anyhow::anyhow!("Home directory not found"))?
			.join(".sare")
			.join("sessions");
		
		Ok(Self {
			storage_dir,
			session_cache: Arc::new(RwLock::new(HashMap::new())),
			initialized: Arc::new(RwLock::new(false)),
		})
	}
	
	/**
	 * Initializes the session store
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * セッションストアを初期化する関数です
		 * 
		 * ストレージディレクトリを作成し、既存のセッション
		 * データを読み込みます。
		 * 
		 * アプリケーション起動時に呼び出され、セッションの
		 * 復旧準備を行います
		 */
		
		// ストレージディレクトリを作成
		if !self.storage_dir.exists() {
			fs::create_dir_all(&self.storage_dir).await?;
		}
		
		// 既存のセッションデータを読み込み
		self.load_all_sessions().await?;
		
		// 初期化完了をマーク
		{
			let mut initialized = self.initialized.write().await;
			*initialized = true;
		}
		
		Ok(())
	}
	
	/**
	 * Saves a session
	 * 
	 * @param metadata - Session metadata to save
	 * @return Result<()> - Success or error
	 */
	pub async fn save_session(&self, metadata: &SessionMetadata) -> Result<()> {
		/**
		 * セッションを保存する関数です
		 * 
		 * セッションメタデータをファイルシステムに保存し、
		 * キャッシュにも追加します。
		 * 
		 * セッションIDをファイル名として使用し、JSON形式で
		 * 保存されます
		 */
		
		// キャッシュに追加
		{
			let mut cache = self.session_cache.write().await;
			cache.insert(metadata.id, metadata.clone());
		}
		
		// ファイルに保存
		let file_path = self.storage_dir.join(format!("{}.json", metadata.id));
		let json_data = serde_json::to_string_pretty(metadata)?;
		fs::write(file_path, json_data).await?;
		
		Ok(())
	}
	
	/**
	 * Loads a session by ID
	 * 
	 * @param session_id - Session ID to load
	 * @return Result<Option<SessionMetadata>> - Session metadata if found
	 */
	pub async fn load_session(&self, session_id: Uuid) -> Result<Option<SessionMetadata>> {
		/**
		 * セッションIDでセッションを読み込む関数です
		 * 
		 * キャッシュからセッションを検索し、見つからない場合は
		 * ファイルシステムから読み込みます。
		 * 
		 * セッションが見つからない場合は None を返します
		 */
		
		// キャッシュから検索
		{
			let cache = self.session_cache.read().await;
			if let Some(metadata) = cache.get(&session_id) {
				return Ok(Some(metadata.clone()));
			}
		}
		
		// ファイルから読み込み
		let file_path = self.storage_dir.join(format!("{}.json", session_id));
		if file_path.exists() {
			let json_data = fs::read_to_string(file_path).await?;
			let metadata: SessionMetadata = serde_json::from_str(&json_data)?;
			
			// キャッシュに追加
			{
				let mut cache = self.session_cache.write().await;
				cache.insert(session_id, metadata.clone());
			}
			
			return Ok(Some(metadata));
		}
		
		Ok(None)
	}
	
	/**
	 * Loads all sessions
	 * 
	 * @return Result<Vec<SessionMetadata>> - List of all sessions
	 */
	pub async fn load_all_sessions(&self) -> Result<Vec<SessionMetadata>> {
		/**
		 * すべてのセッションを読み込む関数です
		 * 
		 * ストレージディレクトリ内のすべてのセッションファイルを
		 * 読み込み、キャッシュに追加します。
		 * 
		 * アプリケーション起動時に呼び出され、既存のセッションを
		 * 復旧します
		 */
		
		let mut sessions = Vec::new();
		
		// ストレージディレクトリ内のファイルを列挙
		let mut entries = fs::read_dir(&self.storage_dir).await?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) == Some("json") {
				// ファイル名からセッションIDを抽出
				if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
					if let Ok(session_id) = file_name.parse::<Uuid>() {
						// セッションを読み込み
						if let Some(metadata) = self.load_session(session_id).await? {
							sessions.push(metadata);
						}
					}
				}
			}
		}
		
		Ok(sessions)
	}
	
	/**
	 * Loads detached sessions
	 * 
	 * @return Result<Vec<SessionMetadata>> - List of detached sessions
	 */
	pub async fn load_detached_sessions(&self) -> Result<Vec<SessionMetadata>> {
		/**
		 * デタッチされたセッションを読み込む関数です
		 * 
		 * すべてのセッションを読み込み、デタッチ状態の
		 * セッションのみをフィルタリングして返します。
		 * 
		 * セッション復旧時に使用され、デタッチされたセッションを
		 * 復旧可能な状態にします
		 */
		
		let all_sessions = self.load_all_sessions().await?;
		let detached_sessions: Vec<SessionMetadata> = all_sessions
			.into_iter()
			.filter(|metadata| metadata.state == SessionState::Detached)
			.collect();
		
		Ok(detached_sessions)
	}
	
	/**
	 * Deletes a session
	 * 
	 * @param session_id - Session ID to delete
	 * @return Result<()> - Success or error
	 */
	pub async fn delete_session(&self, session_id: Uuid) -> Result<()> {
		/**
		 * セッションを削除する関数です
		 * 
		 * キャッシュからセッションを削除し、ファイルシステムからも
		 * セッションファイルを削除します。
		 * 
		 * セッションの完全な削除を行い、復旧不可能にします
		 */
		
		// キャッシュから削除
		{
			let mut cache = self.session_cache.write().await;
			cache.remove(&session_id);
		}
		
		// ファイルを削除
		let file_path = self.storage_dir.join(format!("{}.json", session_id));
		if file_path.exists() {
			fs::remove_file(file_path).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Gets session count
	 * 
	 * @return Result<usize> - Number of sessions
	 */
	pub async fn get_session_count(&self) -> Result<usize> {
		let cache = self.session_cache.read().await;
		Ok(cache.len())
	}
	
	/**
	 * Clears all sessions
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_all_sessions(&self) -> Result<()> {
		/**
		 * すべてのセッションをクリアする関数です
		 * 
		 * キャッシュをクリアし、ストレージディレクトリ内の
		 * すべてのセッションファイルを削除します。
		 * 
		 * 注意: この操作は取り消しできません
		 */
		
		// キャッシュをクリア
		{
			let mut cache = self.session_cache.write().await;
			cache.clear();
		}
		
		// すべてのセッションファイルを削除
		let mut entries = fs::read_dir(&self.storage_dir).await?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) == Some("json") {
				fs::remove_file(path).await?;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Shuts down the session store
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * セッションストアをシャットダウンする関数です
		 * 
		 * キャッシュをクリアし、リソースを解放します。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// キャッシュをクリア
		{
			let mut cache = self.session_cache.write().await;
			cache.clear();
		}
		
		// 初期化状態をリセット
		{
			let mut initialized = self.initialized.write().await;
			*initialized = false;
		}
		
		Ok(())
	}
	
	/**
	 * Gets storage directory
	 * 
	 * @return PathBuf - Storage directory path
	 */
	pub fn storage_directory(&self) -> PathBuf {
		self.storage_dir.clone()
	}
	
	/**
	 * Checks if store is initialized
	 * 
	 * @return bool - True if initialized
	 */
	pub async fn is_initialized(&self) -> bool {
		let initialized = self.initialized.read().await;
		*initialized
	}
	
	/**
	 * Exports session data
	 * 
	 * @param session_id - Session ID to export
	 * @param export_path - Export file path
	 * @return Result<()> - Success or error
	 */
	pub async fn export_session(&self, session_id: Uuid, export_path: PathBuf) -> Result<()> {
		/**
		 * セッションデータをエクスポートする関数です
		 * 
		 * 指定されたセッションのデータを指定されたパスに
		 * エクスポートします。
		 * 
		 * バックアップやセッションの移行に使用できます
		 */
		
		if let Some(metadata) = self.load_session(session_id).await? {
			let json_data = serde_json::to_string_pretty(&metadata)?;
			fs::write(export_path, json_data).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Imports session data
	 * 
	 * @param import_path - Import file path
	 * @return Result<SessionMetadata> - Imported session metadata
	 */
	pub async fn import_session(&self, import_path: PathBuf) -> Result<SessionMetadata> {
		/**
		 * セッションデータをインポートする関数です
		 * 
		 * 指定されたファイルからセッションデータを読み込み、
		 * ストアに追加します。
		 * 
		 * バックアップからの復旧やセッションの移行に使用できます
		 */
		
		let json_data = fs::read_to_string(import_path).await?;
		let metadata: SessionMetadata = serde_json::from_str(&json_data)?;
		
		// セッションを保存
		self.save_session(&metadata).await?;
		
		Ok(metadata)
	}
} 