/**
 * Session management module for Sare terminal
 * 
 * This module provides comprehensive session management capabilities including
 * detached sessions, session recovery, named sessions, tab support, window
 * management, and session sharing for collaborative work.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Main session management module
 */

pub mod session_manager;
pub mod session_store;
pub mod tab_manager;
pub mod window_manager;
pub mod session_sharing;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/**
 * Session management system
 * 
 * セッション管理システムのメインエントリーポイントです。
 * すべてのセッション関連の機能を統合し、統一されたインターフェースを提供します。
 * 
 * デタッチされたセッション、セッション復旧、名前付きセッション、タブサポート、
 * ウィンドウ管理、セッション共有の各機能を管理します
 */
#[derive(Debug)]
pub struct SessionSystem {
	/// セッションマネージャー
	session_manager: Arc<session_manager::SessionManager>,
	/// セッションストア
	session_store: Arc<session_store::SessionStore>,
	/// タブマネージャー
	tab_manager: Arc<tab_manager::TabManager>,
	/// ウィンドウマネージャー
	window_manager: Arc<window_manager::WindowManager>,
	/// セッション共有マネージャー
	sharing_manager: Arc<session_sharing::SessionSharingManager>,
}

impl SessionSystem {
	/**
	 * Creates a new session system
	 * 
	 * @return SessionSystem - New session system instance
	 */
	pub fn new() -> Result<Self> {
		let session_store = Arc::new(session_store::SessionStore::new()?);
		let session_manager = Arc::new(session_manager::SessionManager::new(session_store.clone())?);
		let tab_manager = Arc::new(tab_manager::TabManager::new()?);
		let window_manager = Arc::new(window_manager::WindowManager::new()?);
		let sharing_manager = Arc::new(session_sharing::SessionSharingManager::new()?);
		
		Ok(Self {
			session_manager,
			session_store,
			tab_manager,
			window_manager,
			sharing_manager,
		})
	}
	
	/**
	 * Gets the session manager
	 * 
	 * @return Arc<SessionManager> - Session manager reference
	 */
	pub fn session_manager(&self) -> Arc<session_manager::SessionManager> {
		self.session_manager.clone()
	}
	
	/**
	 * Gets the session store
	 * 
	 * @return Arc<SessionStore> - Session store reference
	 */
	pub fn session_store(&self) -> Arc<session_store::SessionStore> {
		self.session_store.clone()
	}
	
	/**
	 * Gets the tab manager
	 * 
	 * @return Arc<TabManager> - Tab manager reference
	 */
	pub fn tab_manager(&self) -> Arc<tab_manager::TabManager> {
		self.tab_manager.clone()
	}
	
	/**
	 * Gets the window manager
	 * 
	 * @return Arc<WindowManager> - Window manager reference
	 */
	pub fn window_manager(&self) -> Arc<window_manager::WindowManager> {
		self.window_manager.clone()
	}
	
	/**
	 * Gets the sharing manager
	 * 
	 * @return Arc<SessionSharingManager> - Sharing manager reference
	 */
	pub fn sharing_manager(&self) -> Arc<session_sharing::SessionSharingManager> {
		self.sharing_manager.clone()
	}
	
	/**
	 * Initializes the session system
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		// セッションストアを初期化
		self.session_store.initialize().await?;
		
		// 既存のセッションを復旧
		self.session_manager.recover_sessions().await?;
		
		// タブマネージャーを初期化
		self.tab_manager.initialize().await?;
		
		// ウィンドウマネージャーを初期化
		self.window_manager.initialize().await?;
		
		// セッション共有マネージャーを初期化
		self.sharing_manager.initialize().await?;
		
		Ok(())
	}
	
	/**
	 * Shuts down the session system
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		// すべてのセッションを保存
		self.session_manager.save_all_sessions().await?;
		
		// セッションストアをシャットダウン
		self.session_store.shutdown().await?;
		
		// タブマネージャーをシャットダウン
		self.tab_manager.shutdown().await?;
		
		// ウィンドウマネージャーをシャットダウン
		self.window_manager.shutdown().await?;
		
		// セッション共有マネージャーをシャットダウン
		self.sharing_manager.shutdown().await?;
		
		Ok(())
	}
	
	/**
	 * Gets system status
	 * 
	 * @return SessionSystemStatus - System status
	 */
	pub async fn get_status(&self) -> Result<SessionSystemStatus> {
		let session_count = self.session_manager.get_session_count().await?;
		let tab_count = self.tab_manager.get_tab_count().await?;
		let window_count = self.window_manager.get_window_count().await?;
		let sharing_count = self.sharing_manager.get_sharing_count().await?;
		
		Ok(SessionSystemStatus {
			session_count,
			tab_count,
			window_count,
			sharing_count,
			timestamp: Utc::now(),
		})
	}
}

/**
 * Session system status
 * 
 * セッションシステムの状態情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SessionSystemStatus {
	/// アクティブなセッション数
	pub session_count: usize,
	/// アクティブなタブ数
	pub tab_count: usize,
	/// アクティブなウィンドウ数
	pub window_count: usize,
	/// 共有セッション数
	pub sharing_count: usize,
	/// タイムスタンプ
	pub timestamp: DateTime<Utc>,
}

/**
 * Session types
 * 
 * セッションの種類を定義します
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionType {
	/// 通常のセッション
	Normal,
	/// デタッチされたセッション
	Detached,
	/// 共有セッション
	Shared,
	/// 一時的なセッション
	Temporary,
}

/**
 * Session state
 * 
 * セッションの状態を定義します
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
	/// アクティブ
	Active,
	/// デタッチ済み
	Detached,
	/// 一時停止
	Suspended,
	/// 終了
	Terminated,
	/// 復旧中
	Recovering,
}

/**
 * Tab types
 * 
 * タブの種類を定義します
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TabType {
	/// 通常のタブ
	Normal,
	/// システムタブ
	System,
	/// 共有タブ
	Shared,
}

/**
 * Window types
 * 
 * ウィンドウの種類を定義します
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WindowType {
	/// 通常のウィンドウ
	Normal,
	/// 分割ウィンドウ
	Split,
	/// フローティングウィンドウ
	Floating,
	/// フルスクリーンウィンドウ
	Fullscreen,
}

/**
 * Sharing permissions
 * 
 * 共有権限を定義します
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SharingPermission {
	/// 読み取り専用
	ReadOnly,
	/// 読み書き可能
	ReadWrite,
	/// 管理者権限
	Admin,
}

/**
 * Session metadata
 * 
 * セッションのメタデータを格納します
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
	/// セッションID
	pub id: Uuid,
	/// セッション名
	pub name: String,
	/// セッションタイプ
	pub session_type: SessionType,
	/// セッション状態
	pub state: SessionState,
	/// 作成日時
	pub created_at: DateTime<Utc>,
	/// 更新日時
	pub updated_at: DateTime<Utc>,
	/// 所有者
	pub owner: String,
	/// 共有設定
	pub sharing_config: Option<SharingConfig>,
	/// カスタムメタデータ
	pub custom_metadata: HashMap<String, String>,
}

/**
 * Sharing configuration
 * 
 * 共有設定を格納します
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
	/// 共有権限
	pub permission: SharingPermission,
	/// 共有ユーザーリスト
	pub shared_users: Vec<String>,
	/// 共有開始日時
	pub shared_at: DateTime<Utc>,
	/// 共有終了日時
	pub expires_at: Option<DateTime<Utc>>,
}

/**
 * Tab metadata
 * 
 * タブのメタデータを格納します
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMetadata {
	/// タブID
	pub id: Uuid,
	/// タブ名
	pub name: String,
	/// タブタイプ
	pub tab_type: TabType,
	/// 親セッションID
	pub session_id: Uuid,
	/// 作成日時
	pub created_at: DateTime<Utc>,
	/// 更新日時
	pub updated_at: DateTime<Utc>,
}

/**
 * Window metadata
 * 
 * ウィンドウのメタデータを格納します
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowMetadata {
	/// ウィンドウID
	pub id: Uuid,
	/// ウィンドウ名
	pub name: String,
	/// ウィンドウタイプ
	pub window_type: WindowType,
	/// 親セッションID
	pub session_id: Uuid,
	/// 親タブID
	pub tab_id: Option<Uuid>,
	/// 位置とサイズ
	pub geometry: WindowGeometry,
	/// 作成日時
	pub created_at: DateTime<Utc>,
	/// 更新日時
	pub updated_at: DateTime<Utc>,
}

/**
 * Window geometry
 * 
 * ウィンドウの位置とサイズを格納します
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
	/// X座標
	pub x: i32,
	/// Y座標
	pub y: i32,
	/// 幅
	pub width: u32,
	/// 高さ
	pub height: u32,
}

impl Default for WindowGeometry {
	fn default() -> Self {
		Self {
			x: 0,
			y: 0,
			width: 800,
			height: 600,
		}
	}
} 