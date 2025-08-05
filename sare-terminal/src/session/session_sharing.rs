/**
 * Session sharing manager for Sare terminal
 * 
 * This module provides session sharing capabilities including collaborative
 * sessions, session sharing between users, and sharing management for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: session_sharing.rs
 * Description: Session sharing manager for collaborative sessions
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{
	SessionMetadata, SharingPermission, SharingConfig,
};

/**
 * Session sharing manager
 * 
 * セッション共有の中心的なコンポーネントです。
 * ユーザー間でのセッション共有、協調セッションの管理を
 * 担当します。
 * 
 * セッションの共有、共有解除、権限管理の各機能を提供し、
 * 複数ユーザーでの協調作業を実現します
 */
pub struct SessionSharingManager {
	/// 共有セッションの情報
	shared_sessions: Arc<RwLock<HashMap<Uuid, SharedSessionInfo>>>,
	/// ユーザーごとの共有セッション
	user_shared_sessions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
	/// 共有セッションの参加者
	session_participants: Arc<RwLock<HashMap<Uuid, Vec<ParticipantInfo>>>>,
	/// 共有セッションのイベント履歴
	session_events: Arc<RwLock<HashMap<Uuid, Vec<SharingEvent>>>>,
}

impl SessionSharingManager {
	/**
	 * Creates a new session sharing manager
	 * 
	 * @return Result<SessionSharingManager> - New session sharing manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいセッション共有マネージャーを作成する関数です
		 * 
		 * 共有セッションの情報、ユーザーごとの共有セッション、
		 * セッション参加者、イベント履歴を初期化します。
		 * 
		 * セッション共有の全機能を管理します
		 */
		
		Ok(Self {
			shared_sessions: Arc::new(RwLock::new(HashMap::new())),
			user_shared_sessions: Arc::new(RwLock::new(HashMap::new())),
			session_participants: Arc::new(RwLock::new(HashMap::new())),
			session_events: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Initializes the session sharing manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * セッション共有マネージャーを初期化する関数です
		 * 
		 * 既存の共有セッションデータを読み込み、共有マネージャーを
		 * 準備状態にします。
		 * 
		 * アプリケーション起動時に呼び出され、共有セッションの
		 * 復旧準備を行います
		 */
		
		// Load persistent session sharing data from storage
		self.load_persistent_session_sharing_data().await?;
		
		Ok(())
	}
	
	/**
	 * Loads persistent session sharing data from storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn load_persistent_session_sharing_data(&self) -> Result<()> {
		/**
		 * 永続化されたセッション共有データをストレージから読み込む関数です
		 * 
		 * 共有セッションの情報、ユーザーごとの共有セッション、
		 * セッション参加者、イベント履歴を復元します。
		 * 
		 * アプリケーション起動時に既存の共有セッション状態を
		 * 復旧するために使用されます
		 */
		
		// Get user's home directory for session sharing data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let sharing_data_file = home_dir.join(".sare_session_sharing.json");
		
		// Try to load existing session sharing data
		if let Ok(data) = tokio::fs::read_to_string(&sharing_data_file).await {
			if let Ok(sharing_data) = serde_json::from_str::<serde_json::Value>(&data) {
				// Load shared sessions info
				if let Some(shared_sessions_data) = sharing_data.get("shared_sessions") {
					if let Some(shared_sessions_object) = shared_sessions_data.as_object() {
						let mut shared_sessions = self.shared_sessions.write().await;
						for (session_id_str, session_info) in shared_sessions_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Ok(info) = serde_json::from_value::<SharedSessionInfo>(session_info.clone()) {
									shared_sessions.insert(session_id, info);
								}
							}
						}
					}
				}
				
				// Load user shared sessions mapping
				if let Some(user_shared_sessions_data) = sharing_data.get("user_shared_sessions") {
					if let Some(user_shared_sessions_object) = user_shared_sessions_data.as_object() {
						let mut user_shared_sessions = self.user_shared_sessions.write().await;
						for (user_str, session_ids_array) in user_shared_sessions_object {
							if let Some(session_ids) = session_ids_array.as_array() {
								let mut session_ids_vec = Vec::new();
								for session_id_value in session_ids {
									if let Some(session_id_str) = session_id_value.as_str() {
										if let Ok(session_id) = Uuid::parse_str(session_id_str) {
											session_ids_vec.push(session_id);
										}
									}
								}
								user_shared_sessions.insert(user_str.clone(), session_ids_vec);
							}
						}
					}
				}
				
				// Load session participants mapping
				if let Some(session_participants_data) = sharing_data.get("session_participants") {
					if let Some(session_participants_object) = session_participants_data.as_object() {
						let mut session_participants = self.session_participants.write().await;
						for (session_id_str, participants_array) in session_participants_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Some(participants) = participants_array.as_array() {
									let mut participants_vec = Vec::new();
									for participant_value in participants {
										if let Ok(participant) = serde_json::from_value::<ParticipantInfo>(participant_value.clone()) {
											participants_vec.push(participant);
										}
									}
									session_participants.insert(session_id, participants_vec);
								}
							}
						}
					}
				}
				
				// Load session events mapping
				if let Some(session_events_data) = sharing_data.get("session_events") {
					if let Some(session_events_object) = session_events_data.as_object() {
						let mut session_events = self.session_events.write().await;
						for (session_id_str, events_array) in session_events_object {
							if let Ok(session_id) = Uuid::parse_str(session_id_str) {
								if let Some(events) = events_array.as_array() {
									let mut events_vec = Vec::new();
									for event_value in events {
										if let Ok(event) = serde_json::from_value::<SharingEvent>(event_value.clone()) {
											events_vec.push(event);
										}
									}
									session_events.insert(session_id, events_vec);
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
	 * Saves persistent session sharing data to storage
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn save_persistent_session_sharing_data(&self) -> Result<()> {
		/**
		 * セッション共有データを永続化ストレージに保存する関数です
		 * 
		 * 共有セッションの情報、ユーザーごとの共有セッション、
		 * セッション参加者、イベント履歴を保存します。
		 * 
		 * アプリケーション終了時に共有セッション状態を
		 * 永続化するために使用されます
		 */
		
		// Get user's home directory for session sharing data storage
		let home_dir = dirs::home_dir().ok_or_else(|| {
			anyhow::anyhow!("Could not determine home directory")
		})?;
		
		let sharing_data_file = home_dir.join(".sare_session_sharing.json");
		
		// Prepare session sharing data for serialization
		let mut sharing_data = serde_json::Map::new();
		
		// Serialize shared sessions info
		let shared_sessions = self.shared_sessions.read().await;
		let mut shared_sessions_object = serde_json::Map::new();
		for (session_id, session_info) in shared_sessions.iter() {
			shared_sessions_object.insert(session_id.to_string(), serde_json::to_value(session_info)?);
		}
		sharing_data.insert("shared_sessions".to_string(), serde_json::Value::Object(shared_sessions_object));
		
		// Serialize user shared sessions mapping
		let user_shared_sessions = self.user_shared_sessions.read().await;
		let mut user_shared_sessions_object = serde_json::Map::new();
		for (user, session_ids) in user_shared_sessions.iter() {
			let session_ids_array: Vec<serde_json::Value> = session_ids.iter()
				.map(|id| serde_json::Value::String(id.to_string()))
				.collect();
			user_shared_sessions_object.insert(user.clone(), serde_json::Value::Array(session_ids_array));
		}
		sharing_data.insert("user_shared_sessions".to_string(), serde_json::Value::Object(user_shared_sessions_object));
		
		// Serialize session participants mapping
		let session_participants = self.session_participants.read().await;
		let mut session_participants_object = serde_json::Map::new();
		for (session_id, participants) in session_participants.iter() {
			let participants_array: Vec<serde_json::Value> = participants.iter()
				.map(|participant| serde_json::to_value(participant).unwrap_or_default())
				.collect();
			session_participants_object.insert(session_id.to_string(), serde_json::Value::Array(participants_array));
		}
		sharing_data.insert("session_participants".to_string(), serde_json::Value::Object(session_participants_object));
		
		// Serialize session events mapping
		let session_events = self.session_events.read().await;
		let mut session_events_object = serde_json::Map::new();
		for (session_id, events) in session_events.iter() {
			let events_array: Vec<serde_json::Value> = events.iter()
				.map(|event| serde_json::to_value(event).unwrap_or_default())
				.collect();
			session_events_object.insert(session_id.to_string(), serde_json::Value::Array(events_array));
		}
		sharing_data.insert("session_events".to_string(), serde_json::Value::Object(session_events_object));
		
		// Write session sharing data to file
		let sharing_data_json = serde_json::to_string_pretty(&serde_json::Value::Object(sharing_data))?;
		tokio::fs::write(&sharing_data_file, sharing_data_json).await?;
		
		Ok(())
	}
	
	/**
	 * Shares a session
	 * 
	 * @param session_id - Session ID to share
	 * @param owner - Session owner
	 * @param permission - Sharing permission
	 * @param shared_users - List of users to share with
	 * @param expires_at - Expiration time
	 * @return Result<SharedSessionInfo> - Shared session information
	 */
	pub async fn share_session(
		&self,
		session_id: Uuid,
		owner: String,
		permission: SharingPermission,
		shared_users: Vec<String>,
		expires_at: Option<DateTime<Utc>>,
	) -> Result<SharedSessionInfo> {
		/**
		 * セッションを共有する関数です
		 * 
		 * 指定されたセッションを他のユーザーと共有し、
		 * 共有セッション情報を作成します。
		 * 
		 * 共有権限、共有ユーザー、有効期限を設定できます
		 */
		
		let now = Utc::now();
		
		let shared_info = SharedSessionInfo {
			session_id,
			owner: owner.clone(),
			permission,
			shared_users: shared_users.clone(),
			shared_at: now,
			expires_at,
			active_participants: vec![owner],
			status: SharingStatus::Active,
		};
		
		// 共有セッション情報に追加
		{
			let mut shared_sessions = self.shared_sessions.write().await;
			shared_sessions.insert(session_id, shared_info.clone());
		}
		
		// ユーザーごとの共有セッションに追加
		{
			let mut user_shared_sessions = self.user_shared_sessions.write().await;
			for user in &shared_users {
				let sessions = user_shared_sessions.entry(user.clone()).or_insert_with(Vec::new);
				sessions.push(session_id);
			}
		}
		
		// 共有イベントを記録
		{
			let mut session_events = self.session_events.write().await;
			let events = session_events.entry(session_id).or_insert_with(Vec::new);
			events.push(SharingEvent {
				timestamp: now,
				event_type: SharingEventType::SessionShared,
				user: owner,
				details: format!("Shared with {} users", shared_users.len()),
			});
		}
		
		Ok(shared_info)
	}
	
	/**
	 * Unshares a session
	 * 
	 * @param session_id - Session ID to unshare
	 * @param owner - Session owner
	 * @return Result<()> - Success or error
	 */
	pub async fn unshare_session(&self, session_id: Uuid, owner: String) -> Result<()> {
		/**
		 * セッションの共有を解除する関数です
		 * 
		 * 指定されたセッションの共有を解除し、共有セッション
		 * 情報を削除します。
		 * 
		 * 所有者のみが共有解除を実行できます
		 */
		
		// 共有セッション情報を確認
		{
			let shared_sessions = self.shared_sessions.read().await;
			if let Some(shared_info) = shared_sessions.get(&session_id) {
				if shared_info.owner != owner {
					return Err(anyhow::anyhow!("Only owner can unshare session"));
				}
			} else {
				return Err(anyhow::anyhow!("Session not shared"));
			}
		}
		
		// 共有セッション情報から削除
		{
			let mut shared_sessions = self.shared_sessions.write().await;
			shared_sessions.remove(&session_id);
		}
		
		// ユーザーごとの共有セッションから削除
		{
			let mut user_shared_sessions = self.user_shared_sessions.write().await;
			for (_, sessions) in user_shared_sessions.iter_mut() {
				sessions.retain(|&id| id != session_id);
			}
		}
		
		// セッション参加者から削除
		{
			let mut session_participants = self.session_participants.write().await;
			session_participants.remove(&session_id);
		}
		
		// 共有解除イベントを記録
		{
			let mut session_events = self.session_events.write().await;
			if let Some(events) = session_events.get_mut(&session_id) {
				events.push(SharingEvent {
					timestamp: Utc::now(),
					event_type: SharingEventType::SessionUnshared,
					user: owner,
					details: "Session unshared".to_string(),
				});
			}
		}
		
		Ok(())
	}
	
	/**
	 * Joins a shared session
	 * 
	 * @param session_id - Session ID to join
	 * @param user - User joining the session
	 * @return Result<()> - Success or error
	 */
	pub async fn join_shared_session(&self, session_id: Uuid, user: String) -> Result<()> {
		/**
		 * 共有セッションに参加する関数です
		 * 
		 * 指定された共有セッションにユーザーを参加させ、
		 * 参加者リストに追加します。
		 * 
		 * セッションが共有されていることを確認してから
		 * 参加を許可します
		 */
		
		// 共有セッション情報を確認
		{
			let shared_sessions = self.shared_sessions.read().await;
			if let Some(shared_info) = shared_sessions.get(&session_id) {
				if !shared_info.shared_users.contains(&user) {
					return Err(anyhow::anyhow!("User not authorized to join session"));
				}
				
				if let Some(expires_at) = shared_info.expires_at {
					if Utc::now() > expires_at {
						return Err(anyhow::anyhow!("Session sharing has expired"));
					}
				}
			} else {
				return Err(anyhow::anyhow!("Session not shared"));
			}
		}
		
		// 参加者情報を作成
		let participant_info = ParticipantInfo {
			user: user.clone(),
			joined_at: Utc::now(),
			permission: {
				let shared_sessions = self.shared_sessions.read().await;
				if let Some(shared_info) = shared_sessions.get(&session_id) {
					shared_info.permission
				} else {
					SharingPermission::ReadOnly
				}
			},
			status: ParticipantStatus::Active,
		};
		
		// セッション参加者に追加
		{
			let mut session_participants = self.session_participants.write().await;
			let participants = session_participants.entry(session_id).or_insert_with(Vec::new);
			participants.push(participant_info);
		}
		
		// アクティブ参加者に追加
		{
			let mut shared_sessions = self.shared_sessions.write().await;
			if let Some(shared_info) = shared_sessions.get_mut(&session_id) {
				if !shared_info.active_participants.contains(&user) {
					shared_info.active_participants.push(user.clone());
				}
			}
		}
		
		// 参加イベントを記録
		{
			let mut session_events = self.session_events.write().await;
			let events = session_events.entry(session_id).or_insert_with(Vec::new);
			events.push(SharingEvent {
				timestamp: Utc::now(),
				event_type: SharingEventType::UserJoined,
				user,
				details: "User joined session".to_string(),
			});
		}
		
		Ok(())
	}
	
	/**
	 * Leaves a shared session
	 * 
	 * @param session_id - Session ID to leave
	 * @param user - User leaving the session
	 * @return Result<()> - Success or error
	 */
	pub async fn leave_shared_session(&self, session_id: Uuid, user: String) -> Result<()> {
		/**
		 * 共有セッションから退出する関数です
		 * 
		 * 指定された共有セッションからユーザーを退出させ、
		 * 参加者リストから削除します。
		 * 
		 * アクティブ参加者リストからも削除します
		 */
		
		// セッション参加者から削除
		{
			let mut session_participants = self.session_participants.write().await;
			if let Some(participants) = session_participants.get_mut(&session_id) {
				participants.retain(|p| p.user != user);
			}
		}
		
		// アクティブ参加者から削除
		{
			let mut shared_sessions = self.shared_sessions.write().await;
			if let Some(shared_info) = shared_sessions.get_mut(&session_id) {
				shared_info.active_participants.retain(|u| u != &user);
			}
		}
		
		// 退出イベントを記録
		{
			let mut session_events = self.session_events.write().await;
			if let Some(events) = session_events.get_mut(&session_id) {
				events.push(SharingEvent {
					timestamp: Utc::now(),
					event_type: SharingEventType::UserLeft,
					user: user.clone(),
					details: "User left session".to_string(),
				});
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets shared session information
	 * 
	 * @param session_id - Session ID
	 * @return Result<Option<SharedSessionInfo>> - Shared session information if found
	 */
	pub async fn get_shared_session_info(&self, session_id: Uuid) -> Result<Option<SharedSessionInfo>> {
		let shared_sessions = self.shared_sessions.read().await;
		Ok(shared_sessions.get(&session_id).cloned())
	}
	
	/**
	 * Gets participants for a shared session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Vec<ParticipantInfo>> - List of participants
	 */
	pub async fn get_session_participants(&self, session_id: Uuid) -> Result<Vec<ParticipantInfo>> {
		/**
		 * 共有セッションの参加者を取得する関数です
		 * 
		 * 指定された共有セッションの参加者リストを
		 * 参加日時の順で返します。
		 * 
		 * 参加者は参加日時の順でソートされます
		 */
		
		let session_participants = self.session_participants.read().await;
		if let Some(participants) = session_participants.get(&session_id) {
			let mut sorted_participants = participants.clone();
			sorted_participants.sort_by(|a, b| a.joined_at.cmp(&b.joined_at));
			return Ok(sorted_participants);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets sharing events for a session
	 * 
	 * @param session_id - Session ID
	 * @return Result<Vec<SharingEvent>> - List of sharing events
	 */
	pub async fn get_session_events(&self, session_id: Uuid) -> Result<Vec<SharingEvent>> {
		/**
		 * セッションの共有イベントを取得する関数です
		 * 
		 * 指定されたセッションの共有イベント履歴を
		 * タイムスタンプの順で返します。
		 * 
		 * イベントはタイムスタンプの順でソートされます
		 */
		
		let session_events = self.session_events.read().await;
		if let Some(events) = session_events.get(&session_id) {
			let mut sorted_events = events.clone();
			sorted_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
			return Ok(sorted_events);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets shared sessions for a user
	 * 
	 * @param user - User name
	 * @return Result<Vec<SharedSessionInfo>> - List of shared sessions
	 */
	pub async fn get_user_shared_sessions(&self, user: &str) -> Result<Vec<SharedSessionInfo>> {
		/**
		 * ユーザーの共有セッションを取得する関数です
		 * 
		 * 指定されたユーザーが参加可能な共有セッションの
		 * リストを返します。
		 * 
		 * セッションは共有日時の順でソートされます
		 */
		
		let user_shared_sessions = self.user_shared_sessions.read().await;
		if let Some(session_ids) = user_shared_sessions.get(user) {
			let shared_sessions = self.shared_sessions.read().await;
			let mut sessions: Vec<SharedSessionInfo> = session_ids
				.iter()
				.filter_map(|id| shared_sessions.get(id).cloned())
				.collect();
			
			// 共有日時でソート
			sessions.sort_by(|a, b| a.shared_at.cmp(&b.shared_at));
			
			return Ok(sessions);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets sharing count
	 * 
	 * @return Result<usize> - Number of shared sessions
	 */
	pub async fn get_sharing_count(&self) -> Result<usize> {
		let shared_sessions = self.shared_sessions.read().await;
		Ok(shared_sessions.len())
	}
	
	/**
	 * Shuts down the session sharing manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * セッション共有マネージャーをシャットダウンする関数です
		 * 
		 * すべての共有セッションデータをクリアし、リソースを解放します。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// Save persistent session sharing data before clearing
		self.save_persistent_session_sharing_data().await?;
		
		// すべてのデータをクリア
		{
			let mut shared_sessions = self.shared_sessions.write().await;
			shared_sessions.clear();
		}
		
		{
			let mut user_shared_sessions = self.user_shared_sessions.write().await;
			user_shared_sessions.clear();
		}
		
		{
			let mut session_participants = self.session_participants.write().await;
			session_participants.clear();
		}
		
		{
			let mut session_events = self.session_events.write().await;
			session_events.clear();
		}
		
		Ok(())
	}
}

/**
 * Shared session information
 * 
 * 共有セッションの情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SharedSessionInfo {
	/// セッションID
	pub session_id: Uuid,
	/// 所有者
	pub owner: String,
	/// 共有権限
	pub permission: SharingPermission,
	/// 共有ユーザーリスト
	pub shared_users: Vec<String>,
	/// 共有開始日時
	pub shared_at: DateTime<Utc>,
	/// 共有終了日時
	pub expires_at: Option<DateTime<Utc>>,
	/// アクティブな参加者
	pub active_participants: Vec<String>,
	/// 共有状態
	pub status: SharingStatus,
}

/**
 * Sharing status
 * 
 * 共有状態を定義します
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SharingStatus {
	/// アクティブ
	Active,
	/// 一時停止
	Suspended,
	/// 終了
	Terminated,
}

/**
 * Participant information
 * 
 * 参加者の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct ParticipantInfo {
	/// ユーザー名
	pub user: String,
	/// 参加日時
	pub joined_at: DateTime<Utc>,
	/// 権限
	pub permission: SharingPermission,
	/// 参加者状態
	pub status: ParticipantStatus,
}

/**
 * Participant status
 * 
 * 参加者状態を定義します
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantStatus {
	/// アクティブ
	Active,
	/// 一時停止
	Suspended,
	/// 退出
	Left,
}

/**
 * Sharing event
 * 
 * 共有イベントを格納します
 */
#[derive(Debug, Clone)]
pub struct SharingEvent {
	/// タイムスタンプ
	pub timestamp: DateTime<Utc>,
	/// イベントタイプ
	pub event_type: SharingEventType,
	/// ユーザー
	pub user: String,
	/// 詳細
	pub details: String,
}

/**
 * Sharing event type
 * 
 * 共有イベントの種類を定義します
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SharingEventType {
	/// セッション共有
	SessionShared,
	/// セッション共有解除
	SessionUnshared,
	/// ユーザー参加
	UserJoined,
	/// ユーザー退出
	UserLeft,
	/// 権限変更
	PermissionChanged,
} 