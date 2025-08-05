/**
 * Hyperlink support manager for Sare terminal
 * 
 * This module provides hyperlink support capabilities including clickable links,
 * URL detection and handling for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: hyperlink_support.rs
 * Description: Hyperlink support manager for clickable links and URL detection
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;

use super::{
	HyperlinkType,
};

/**
 * Hyperlink manager
 * 
 * ハイパーリンクサポートの中心的なコンポーネントです。
 * クリック可能なリンク、URL検出と処理を担当します。
 * 
 * URLの検出、リンクの管理、クリックイベントの処理の各機能を提供し、
 * 複数のリンクタイプに対応します
 */
pub struct HyperlinkManager {
	/// 検出されたリンク
	detected_links: Arc<RwLock<HashMap<Uuid, DetectedLink>>>,
	/// URL検出パターン
	url_patterns: Vec<Regex>,
	/// リンク設定
	link_config: LinkConfig,
	/// リンクイベントハンドラー
	link_handlers: Arc<RwLock<HashMap<HyperlinkType, Box<dyn LinkHandler>>>>,
}

impl HyperlinkManager {
	/**
	 * Creates a new hyperlink manager
	 * 
	 * @return Result<HyperlinkManager> - New hyperlink manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいハイパーリンクマネージャーを作成する関数です
		 * 
		 * 検出されたリンク、URL検出パターン、リンク設定、
		 * リンクイベントハンドラーを初期化します。
		 * 
		 * HTTP/HTTPS、FTP、ファイル、メールリンクの各タイプを
		 * サポートします
		 */
		
		let url_patterns = vec![
			Regex::new(r"https?://[^\s]+").unwrap(),
			Regex::new(r"ftp://[^\s]+").unwrap(),
			Regex::new(r"file://[^\s]+").unwrap(),
			Regex::new(r"mailto:[^\s]+").unwrap(),
		];
		
		let link_config = LinkConfig {
			enable_detection: true,
			enable_clicking: true,
			enable_highlighting: true,
			max_link_length: 2048,
			link_timeout: 30,
		};
		
		Ok(Self {
			detected_links: Arc::new(RwLock::new(HashMap::new())),
			url_patterns,
			link_config,
			link_handlers: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Initializes the hyperlink manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * ハイパーリンクマネージャーを初期化する関数です
		 * 
		 * リンクイベントハンドラーを登録し、URL検出パターンを
		 * 準備します。
		 * 
		 * アプリケーション起動時に呼び出され、リンク検出の
		 * 準備を行います
		 */
		
		// リンクイベントハンドラーを登録
		{
			let mut handlers = self.link_handlers.write().await;
			handlers.insert(HyperlinkType::Http, Box::new(HttpLinkHandler::new()));
			handlers.insert(HyperlinkType::Ftp, Box::new(FtpLinkHandler::new()));
			handlers.insert(HyperlinkType::File, Box::new(FileLinkHandler::new()));
			handlers.insert(HyperlinkType::Mailto, Box::new(MailtoLinkHandler::new()));
		}
		
		Ok(())
	}
	
	/**
	 * Detects links in text
	 * 
	 * @param text - Text to scan for links
	 * @param position - Text position
	 * @return Result<Vec<DetectedLink>> - Detected links
	 */
	pub async fn detect_links(&self, text: &str, position: (u32, u32)) -> Result<Vec<DetectedLink>> {
		/**
		 * テキスト内のリンクを検出する関数です
		 * 
		 * 指定されたテキストをスキャンし、URLパターンに
		 * マッチするリンクを検出します。
		 * 
		 * 検出されたリンクは位置情報とともに保存され、
		 * クリック可能な状態になります
		 */
		
		let mut detected_links = Vec::new();
		
		for pattern in &self.url_patterns {
			for matcher in pattern.find_iter(text) {
				let url = matcher.as_str();
				let link_type = self.determine_link_type(url);
				let link_id = Uuid::new_v4();
				
				let detected_link = DetectedLink {
					id: link_id,
					url: url.to_string(),
					link_type,
					position,
					start_offset: matcher.start(),
					end_offset: matcher.end(),
					detected_at: Utc::now(),
					click_count: 0,
				};
				
				// 検出されたリンクに追加
				{
					let mut links = self.detected_links.write().await;
					links.insert(link_id, detected_link.clone());
				}
				
				detected_links.push(detected_link);
			}
		}
		
		Ok(detected_links)
	}
	
	/**
	 * Handles link click
	 * 
	 * @param link_id - Link ID to click
	 * @return Result<()> - Success or error
	 */
	pub async fn handle_link_click(&self, link_id: Uuid) -> Result<()> {
		/**
		 * リンククリックを処理する関数です
		 * 
		 * 指定されたリンクIDのリンクをクリックし、
		 * 適切なハンドラーで処理します。
		 * 
		 * リンクタイプに応じてブラウザ、ファイルマネージャー、
		 * メールクライアントなどを起動します
		 */
		
		// リンクを取得
		let link = {
			let links = self.detected_links.read().await;
			if let Some(link) = links.get(&link_id) {
				link.clone()
			} else {
				return Err(anyhow::anyhow!("Link not found"));
			}
		};
		
		// クリックカウントを更新
		{
			let mut links = self.detected_links.write().await;
			if let Some(link) = links.get_mut(&link_id) {
				link.click_count += 1;
			}
		}
		
		// リンクタイプに応じたハンドラーを取得
		let handlers = self.link_handlers.read().await;
		if let Some(handler) = handlers.get(&link.link_type) {
			handler.handle_click(&link.url).await?;
		} else {
			return Err(anyhow::anyhow!("No handler for link type"));
		}
		
		Ok(())
	}
	
	/**
	 * Gets link by ID
	 * 
	 * @param link_id - Link ID
	 * @return Result<Option<DetectedLink>> - Detected link if found
	 */
	pub async fn get_link(&self, link_id: Uuid) -> Result<Option<DetectedLink>> {
		let links = self.detected_links.read().await;
		Ok(links.get(&link_id).cloned())
	}
	
	/**
	 * Gets all detected links
	 * 
	 * @return Result<Vec<DetectedLink>> - List of all detected links
	 */
	pub async fn get_all_links(&self) -> Result<Vec<DetectedLink>> {
		/**
		 * すべての検出されたリンクを取得する関数です
		 * 
		 * 検出されたすべてのリンクを検出日時の順で
		 * 返します。
		 * 
		 * リンクは検出日時の順でソートされます
		 */
		
		let links = self.detected_links.read().await;
		let mut link_list: Vec<DetectedLink> = links.values().cloned().collect();
		link_list.sort_by(|a, b| a.detected_at.cmp(&b.detected_at));
		
		Ok(link_list)
	}
	
	/**
	 * Gets links by type
	 * 
	 * @param link_type - Link type to filter
	 * @return Result<Vec<DetectedLink>> - List of links of specified type
	 */
	pub async fn get_links_by_type(&self, link_type: HyperlinkType) -> Result<Vec<DetectedLink>> {
		let links = self.detected_links.read().await;
		let filtered_links: Vec<DetectedLink> = links
			.values()
			.filter(|link| link.link_type == link_type)
			.cloned()
			.collect();
		
		Ok(filtered_links)
	}
	
	/**
	 * Clears all detected links
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_links(&self) -> Result<()> {
		/**
		 * すべての検出されたリンクをクリアする関数です
		 * 
		 * 検出されたすべてのリンクを削除し、メモリを
		 * 解放します。
		 * 
		 * 注意: この操作は取り消しできません
		 */
		
		let mut links = self.detected_links.write().await;
		links.clear();
		
		Ok(())
	}
	
	/**
	 * Gets hyperlink count
	 * 
	 * @return Result<usize> - Number of detected links
	 */
	pub async fn get_hyperlink_count(&self) -> Result<usize> {
		let links = self.detected_links.read().await;
		Ok(links.len())
	}
	
	/**
	 * Shuts down the hyperlink manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * ハイパーリンクマネージャーをシャットダウンする関数です
		 * 
		 * 検出されたリンクをクリアし、リンクイベントハンドラーを
		 * シャットダウンします。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// リンクをクリア
		self.clear_links().await?;
		
		// リンクイベントハンドラーをクリア
		{
			let mut handlers = self.link_handlers.write().await;
			handlers.clear();
		}
		
		Ok(())
	}
	
	/**
	 * Determines link type from URL
	 * 
	 * @param url - URL to analyze
	 * @return HyperlinkType - Determined link type
	 */
	fn determine_link_type(&self, url: &str) -> HyperlinkType {
		if url.starts_with("http://") || url.starts_with("https://") {
			HyperlinkType::Http
		} else if url.starts_with("ftp://") {
			HyperlinkType::Ftp
		} else if url.starts_with("file://") {
			HyperlinkType::File
		} else if url.starts_with("mailto:") {
			HyperlinkType::Mailto
		} else {
			HyperlinkType::Other
		}
	}
}

/**
 * Detected link
 * 
 * 検出されたリンクの情報を格納します
 */
#[derive(Debug, Clone)]
pub struct DetectedLink {
	/// リンクID
	pub id: Uuid,
	/// URL
	pub url: String,
	/// リンクタイプ
	pub link_type: HyperlinkType,
	/// 位置
	pub position: (u32, u32),
	/// 開始オフセット
	pub start_offset: usize,
	/// 終了オフセット
	pub end_offset: usize,
	/// 検出日時
	pub detected_at: DateTime<Utc>,
	/// クリック回数
	pub click_count: u32,
}

/**
 * Link configuration
 * 
 * リンク設定を格納します
 */
#[derive(Debug, Clone)]
pub struct LinkConfig {
	/// 検出を有効にする
	pub enable_detection: bool,
	/// クリックを有効にする
	pub enable_clicking: bool,
	/// ハイライトを有効にする
	pub enable_highlighting: bool,
	/// 最大リンク長
	pub max_link_length: usize,
	/// リンクタイムアウト
	pub link_timeout: u32,
}

/**
 * Link handler trait
 * 
 * リンクハンドラーのトレイトを定義します
 */
#[async_trait::async_trait]
pub trait LinkHandler: Send + Sync {
	/**
	 * Handles link click
	 * 
	 * @param url - URL to handle
	 * @return Result<()> - Success or error
	 */
	async fn handle_click(&self, url: &str) -> Result<()>;
}

/**
 * HTTP link handler
 * 
 * HTTPリンクの処理を担当するコンポーネントです
 */
pub struct HttpLinkHandler {
	/// ブラウザコマンド
	browser_command: String,
}

impl HttpLinkHandler {
	/**
	 * Creates a new HTTP link handler
	 * 
	 * @return HttpLinkHandler - New HTTP link handler instance
	 */
	pub fn new() -> Self {
		Self {
			browser_command: "xdg-open".to_string(),
		}
	}
}

#[async_trait::async_trait]
impl LinkHandler for HttpLinkHandler {
	async fn handle_click(&self, url: &str) -> Result<()> {
		// ブラウザでURLを開く
		let output = tokio::process::Command::new(&self.browser_command)
			.arg(url)
			.output()
			.await?;
		
		if !output.status.success() {
			return Err(anyhow::anyhow!("Failed to open URL: {}", url));
		}
		
		Ok(())
	}
}

/**
 * FTP link handler
 * 
 * FTPリンクの処理を担当するコンポーネントです
 */
pub struct FtpLinkHandler {
	/// FTPクライアントコマンド
	ftp_command: String,
}

impl FtpLinkHandler {
	/**
	 * Creates a new FTP link handler
	 * 
	 * @return FtpLinkHandler - New FTP link handler instance
	 */
	pub fn new() -> Self {
		Self {
			ftp_command: "xdg-open".to_string(),
		}
	}
}

#[async_trait::async_trait]
impl LinkHandler for FtpLinkHandler {
	async fn handle_click(&self, url: &str) -> Result<()> {
		// FTPクライアントでURLを開く
		let output = tokio::process::Command::new(&self.ftp_command)
			.arg(url)
			.output()
			.await?;
		
		if !output.status.success() {
			return Err(anyhow::anyhow!("Failed to open FTP URL: {}", url));
		}
		
		Ok(())
	}
}

/**
 * File link handler
 * 
 * ファイルリンクの処理を担当するコンポーネントです
 */
pub struct FileLinkHandler {
	/// ファイルマネージャーコマンド
	file_manager_command: String,
}

impl FileLinkHandler {
	/**
	 * Creates a new file link handler
	 * 
	 * @return FileLinkHandler - New file link handler instance
	 */
	pub fn new() -> Self {
		Self {
			file_manager_command: "xdg-open".to_string(),
		}
	}
}

#[async_trait::async_trait]
impl LinkHandler for FileLinkHandler {
	async fn handle_click(&self, url: &str) -> Result<()> {
		// ファイルマネージャーでファイルを開く
		let output = tokio::process::Command::new(&self.file_manager_command)
			.arg(url)
			.output()
			.await?;
		
		if !output.status.success() {
			return Err(anyhow::anyhow!("Failed to open file: {}", url));
		}
		
		Ok(())
	}
}

/**
 * Mailto link handler
 * 
 * メールリンクの処理を担当するコンポーネントです
 */
pub struct MailtoLinkHandler {
	/// メールクライアントコマンド
	mail_client_command: String,
}

impl MailtoLinkHandler {
	/**
	 * Creates a new mailto link handler
	 * 
	 * @return MailtoLinkHandler - New mailto link handler instance
	 */
	pub fn new() -> Self {
		Self {
			mail_client_command: "xdg-email".to_string(),
		}
	}
}

#[async_trait::async_trait]
impl LinkHandler for MailtoLinkHandler {
	async fn handle_click(&self, url: &str) -> Result<()> {
		// メールクライアントでメールリンクを開く
		let output = tokio::process::Command::new(&self.mail_client_command)
			.arg(url)
			.output()
			.await?;
		
		if !output.status.success() {
			return Err(anyhow::anyhow!("Failed to open mailto link: {}", url));
		}
		
		Ok(())
	}
} 