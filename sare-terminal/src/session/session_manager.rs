/**
 * Session manager for Sare terminal
 * 
 * This module provides session management capabilities including detached sessions,
 * session recovery, named sessions, and session persistence for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: session_manager.rs
 * Description: Session manager for detached sessions and recovery
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
	SessionMetadata, SessionType, SessionState, SharingConfig, SharingPermission,
	session_store::SessionStore,
};

/**
 * Session manager
 * 
 * セッション管理の中心的なコンポーネントです。
 * デタッチされたセッション、セッション復旧、名前付きセッションの
 * 管理を担当します。
 * 
 * セッションの作成、削除、復旧、永続化の各機能を提供し、
 * tmuxやscreenのような高度なセッション管理を実現します
 */
#[derive(Debug)]
pub struct SessionManager {
	/// セッションストア
	session_store: Arc<SessionStore>,
	/// アクティブなセッション
	active_sessions: Arc<RwLock<HashMap<Uuid, SessionMetadata>>>,
	/// デタッチされたセッション
	detached_sessions: Arc<RwLock<HashMap<Uuid, SessionMetadata>>>,
	/// セッション名からIDへのマッピング
	session_name_map: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl SessionManager {
	/**
	 * Creates a new session manager
	 * 
	 * @param session_store - Session store reference
	 * @return Result<SessionManager> - New session manager instance
	 */
	pub fn new(session_store: Arc<SessionStore>) -> Result<Self> {
		Ok(Self {
			session_store,
			active_sessions: Arc::new(RwLock::new(HashMap::new())),
			detached_sessions: Arc::new(RwLock::new(HashMap::new())),
			session_name_map: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Creates a new session
	 * 
	 * @param name - Session name
	 * @param session_type - Session type
	 * @param owner - Session owner
	 * @return Result<SessionMetadata> - Created session metadata
	 */
	pub async fn create_session(
		&self,
		name: String,
		session_type: SessionType,
		owner: String,
	) -> Result<SessionMetadata> {
		/**
		 * 新しいセッションを作成する関数です
		 * 
		 * 指定された名前、タイプ、所有者でセッションを作成し、
		 * アクティブなセッションとして登録します。
		 * 
		 * セッションIDは自動生成され、作成日時と更新日時が
		 * 自動的に設定されます
		 */
		
		let session_id = Uuid::new_v4();
		let now = Utc::now();
		
		let metadata = SessionMetadata {
			id: session_id,
			name: name.clone(),
			session_type,
			state: SessionState::Active,
			created_at: now,
			updated_at: now,
			owner,
			sharing_config: None,
			custom_metadata: HashMap::new(),
		};
		
		// アクティブセッションに追加
		{
			let mut sessions = self.active_sessions.write().await;
			sessions.insert(session_id, metadata.clone());
		}
		
		// 名前マッピングに追加
		{
			let mut name_map = self.session_name_map.write().await;
			name_map.insert(name, session_id);
		}
		
		// セッションストアに保存
		self.session_store.save_session(&metadata).await?;
		
		Ok(metadata)
	}
	
	/**
	 * Detaches a session
	 * 
	 * @param session_id - Session ID to detach
	 * @return Result<()> - Success or error
	 */
	pub async fn detach_session(&self, session_id: Uuid) -> Result<()> {
		/**
		 * セッションをデタッチする関数です
		 * 
		 * アクティブなセッションをデタッチ状態に変更し、
		 * デタッチされたセッションリストに移動します。
		 * 
		 * デタッチされたセッションは後で再接続可能で、
		 * セッションの状態は保持されます
		 */
		
		let mut sessions = self.active_sessions.write().await;
		if let Some(mut metadata) = sessions.remove(&session_id) {
			metadata.state = SessionState::Detached;
			metadata.updated_at = Utc::now();
			
			// デタッチされたセッションに追加
			{
				let mut detached = self.detached_sessions.write().await;
				detached.insert(session_id, metadata.clone());
			}
			
			// セッションストアに保存
			self.session_store.save_session(&metadata).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Attaches to a detached session
	 * 
	 * @param session_id - Session ID to attach
	 * @return Result<SessionMetadata> - Attached session metadata
	 */
	pub async fn attach_session(&self, session_id: Uuid) -> Result<SessionMetadata> {
		/**
		 * デタッチされたセッションに再接続する関数です
		 * 
		 * デタッチされたセッションをアクティブ状態に戻し、
		 * アクティブセッションリストに移動します。
		 * 
		 * セッションの状態とデータは保持され、継続して
		 * 使用できます
		 */
		
		let mut detached = self.detached_sessions.write().await;
		if let Some(mut metadata) = detached.remove(&session_id) {
			metadata.state = SessionState::Active;
			metadata.updated_at = Utc::now();
			
			// アクティブセッションに追加
			{
				let mut sessions = self.active_sessions.write().await;
				sessions.insert(session_id, metadata.clone());
			}
			
			// セッションストアに保存
			self.session_store.save_session(&metadata).await?;
			
			return Ok(metadata);
		}
		
		Err(anyhow::anyhow!("Session not found or not detached"))
	}
	
	/**
	 * Recovers detached sessions
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn recover_sessions(&self) -> Result<()> {
		/**
		 * デタッチされたセッションを復旧する関数です
		 * 
		 * セッションストアからデタッチされたセッションを読み込み、
		 * デタッチされたセッションリストに追加します。
		 * 
		 * アプリケーション起動時に呼び出され、以前のセッションを
		 * 復旧します
		 */
		
		let detached_sessions = self.session_store.load_detached_sessions().await?;
		
		for metadata in detached_sessions {
			let session_id = metadata.id;
			let mut detached = self.detached_sessions.write().await;
			detached.insert(session_id, metadata);
		}
		
		Ok(())
	}
	
	/**
	 * Gets session by ID
	 * 
	 * @param session_id - Session ID
	 * @return Result<Option<SessionMetadata>> - Session metadata if found
	 */
	pub async fn get_session(&self, session_id: Uuid) -> Result<Option<SessionMetadata>> {
		// アクティブセッションから検索
		{
			let sessions = self.active_sessions.read().await;
			if let Some(metadata) = sessions.get(&session_id) {
				return Ok(Some(metadata.clone()));
			}
		}
		
		// デタッチされたセッションから検索
		{
			let detached = self.detached_sessions.read().await;
			if let Some(metadata) = detached.get(&session_id) {
				return Ok(Some(metadata.clone()));
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Gets session by name
	 * 
	 * @param name - Session name
	 * @return Result<Option<SessionMetadata>> - Session metadata if found
	 */
	pub async fn get_session_by_name(&self, name: &str) -> Result<Option<SessionMetadata>> {
		let name_map = self.session_name_map.read().await;
		if let Some(session_id) = name_map.get(name) {
			self.get_session(*session_id).await
		} else {
			Ok(None)
		}
	}
	
	/**
	 * Lists all sessions
	 * 
	 * @return Result<Vec<SessionMetadata>> - List of all sessions
	 */
	pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
		/**
		 * すべてのセッションをリストアップする関数です
		 * 
		 * アクティブなセッションとデタッチされたセッションを
		 * 統合して返します。
		 * 
		 * セッションは作成日時の順でソートされます
		 */
		
		let mut sessions = Vec::new();
		
		// アクティブセッションを追加
		{
			let active = self.active_sessions.read().await;
			sessions.extend(active.values().cloned());
		}
		
		// デタッチされたセッションを追加
		{
			let detached = self.detached_sessions.read().await;
			sessions.extend(detached.values().cloned());
		}
		
		// 作成日時でソート
		sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
		
		Ok(sessions)
	}
	
	/**
	 * Lists detached sessions
	 * 
	 * @return Result<Vec<SessionMetadata>> - List of detached sessions
	 */
	pub async fn list_detached_sessions(&self) -> Result<Vec<SessionMetadata>> {
		let detached = self.detached_sessions.read().await;
		let mut sessions: Vec<SessionMetadata> = detached.values().cloned().collect();
		sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
		Ok(sessions)
	}
	
	/**
	 * Terminates a session
	 * 
	 * @param session_id - Session ID to terminate
	 * @return Result<()> - Success or error
	 */
	pub async fn terminate_session(&self, session_id: Uuid) -> Result<()> {
		/**
		 * セッションを終了する関数です
		 * 
		 * セッションを終了状態に変更し、アクティブセッションと
		 * デタッチされたセッションの両方から削除します。
		 * 
		 * セッションストアからも削除され、完全に終了します
		 */
		
		// アクティブセッションから削除
		{
			let mut sessions = self.active_sessions.write().await;
			if let Some(metadata) = sessions.remove(&session_id) {
				// 名前マッピングから削除
				{
					let mut name_map = self.session_name_map.write().await;
					name_map.remove(&metadata.name);
				}
				
				// セッションストアから削除
				self.session_store.delete_session(session_id).await?;
				return Ok(());
			}
		}
		
		// デタッチされたセッションから削除
		{
			let mut detached = self.detached_sessions.write().await;
			if let Some(metadata) = detached.remove(&session_id) {
				// 名前マッピングから削除
				{
					let mut name_map = self.session_name_map.write().await;
					name_map.remove(&metadata.name);
				}
				
				// セッションストアから削除
				self.session_store.delete_session(session_id).await?;
				return Ok(());
			}
		}
		
		Err(anyhow::anyhow!("Session not found"))
	}
	
	/**
	 * Saves all sessions
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn save_all_sessions(&self) -> Result<()> {
		/**
		 * すべてのセッションを保存する関数です
		 * 
		 * アクティブなセッションとデタッチされたセッションを
		 * すべてセッションストアに保存します。
		 * 
		 * アプリケーション終了時に呼び出され、セッションの
		 * 永続化を行います
		 */
		
		// アクティブセッションを保存
		{
			let sessions = self.active_sessions.read().await;
			for metadata in sessions.values() {
				self.session_store.save_session(metadata).await?;
			}
		}
		
		// デタッチされたセッションを保存
		{
			let detached = self.detached_sessions.read().await;
			for metadata in detached.values() {
				self.session_store.save_session(metadata).await?;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets session count
	 * 
	 * @return Result<usize> - Number of sessions
	 */
	pub async fn get_session_count(&self) -> Result<usize> {
		let active_count = self.active_sessions.read().await.len();
		let detached_count = self.detached_sessions.read().await.len();
		Ok(active_count + detached_count)
	}
	
	/**
	 * Shares a session
	 * 
	 * @param session_id - Session ID to share
	 * @param permission - Sharing permission
	 * @param shared_users - List of users to share with
	 * @param expires_at - Expiration time
	 * @return Result<()> - Success or error
	 */
	pub async fn share_session(
		&self,
		session_id: Uuid,
		permission: SharingPermission,
		shared_users: Vec<String>,
		expires_at: Option<DateTime<Utc>>,
	) -> Result<()> {
		/**
		 * セッションを共有する関数です
		 * 
		 * 指定されたセッションを他のユーザーと共有し、
		 * 共有設定を更新します。
		 * 
		 * 共有権限、共有ユーザー、有効期限を設定できます
		 */
		
		let sharing_config = SharingConfig {
			permission,
			shared_users,
			shared_at: Utc::now(),
			expires_at,
		};
		
		// アクティブセッションを更新
		{
			let mut sessions = self.active_sessions.write().await;
			if let Some(metadata) = sessions.get_mut(&session_id) {
				metadata.sharing_config = Some(sharing_config.clone());
				metadata.updated_at = Utc::now();
				self.session_store.save_session(metadata).await?;
				return Ok(());
			}
		}
		
		// デタッチされたセッションを更新
		{
			let mut detached = self.detached_sessions.write().await;
			if let Some(metadata) = detached.get_mut(&session_id) {
				metadata.sharing_config = Some(sharing_config);
				metadata.updated_at = Utc::now();
				self.session_store.save_session(metadata).await?;
				return Ok(());
			}
		}
		
		Err(anyhow::anyhow!("Session not found"))
	}
	
	/**
	 * Unshares a session
	 * 
	 * @param session_id - Session ID to unshare
	 * @return Result<()> - Success or error
	 */
	pub async fn unshare_session(&self, session_id: Uuid) -> Result<()> {
		/**
		 * セッションの共有を解除する関数です
		 * 
		 * 指定されたセッションの共有設定を削除し、
		 * プライベートセッションに戻します
		 */
		
		// アクティブセッションを更新
		{
			let mut sessions = self.active_sessions.write().await;
			if let Some(metadata) = sessions.get_mut(&session_id) {
				metadata.sharing_config = None;
				metadata.updated_at = Utc::now();
				self.session_store.save_session(metadata).await?;
				return Ok(());
			}
		}
		
		// デタッチされたセッションを更新
		{
			let mut detached = self.detached_sessions.write().await;
			if let Some(metadata) = detached.get_mut(&session_id) {
				metadata.sharing_config = None;
				metadata.updated_at = Utc::now();
				self.session_store.save_session(metadata).await?;
				return Ok(());
			}
		}
		
		Err(anyhow::anyhow!("Session not found"))
	}
} 