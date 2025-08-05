/**
 * Paste protection manager for Sare terminal
 * 
 * This module provides paste protection capabilities including secure paste
 * handling and paste bracketing for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: paste_protection.rs
 * Description: Paste protection manager for secure paste handling and paste bracketing
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{
	PasteProtectionLevel,
};

/**
 * Paste protection manager
 * 
 * ペースト保護機能の中心的なコンポーネントです。
 * セキュアなペースト処理、ペーストブラケティングを担当します。
 * 
 * ペースト保護、セキュリティチェック、ペースト履歴の管理の各機能を提供し、
 * 複数の保護レベルに対応します
 */
pub struct PasteProtectionManager {
	/// ペースト保護設定
	protection_configs: Arc<RwLock<HashMap<Uuid, ProtectionConfig>>>,
	/// ペースト履歴
	paste_history: Arc<RwLock<Vec<PasteEntry>>>,
	/// セキュリティチェッカー
	security_checkers: Arc<RwLock<Vec<Box<dyn SecurityChecker>>>>,
	/// ペースト保護設定
	paste_config: PasteConfig,
}

impl PasteProtectionManager {
	/**
	 * Creates a new paste protection manager
	 * 
	 * @return Result<PasteProtectionManager> - New paste protection manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しいペースト保護マネージャーを作成する関数です
		 * 
		 * ペースト保護設定、ペースト履歴、セキュリティチェッカー、
		 * ペースト設定を初期化します。
		 * 
		 * 保護なし、低レベル保護、中レベル保護、高レベル保護の各レベルを
		 * サポートします
		 */
		
		let paste_config = PasteConfig {
			enable_protection: true,
			enable_bracketing: true,
			enable_history: true,
			max_paste_history: 100,
			paste_timeout: 30,
			default_protection_level: PasteProtectionLevel::Medium,
		};
		
		Ok(Self {
			protection_configs: Arc::new(RwLock::new(HashMap::new())),
			paste_history: Arc::new(RwLock::new(Vec::new())),
			security_checkers: Arc::new(RwLock::new(Vec::new())),
			paste_config,
		})
	}
	
	/**
	 * Initializes the paste protection manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * ペースト保護マネージャーを初期化する関数です
		 * 
		 * セキュリティチェッカーを登録し、ペースト保護機能の
		 * 準備を行います。
		 * 
		 * アプリケーション起動時に呼び出され、ペースト保護機能の
		 * 準備を行います
		 */
		
		// セキュリティチェッカーを登録
		{
			let mut checkers = self.security_checkers.write().await;
			checkers.push(Box::new(ScriptChecker::new()));
			checkers.push(Box::new(UrlChecker::new()));
			checkers.push(Box::new(CommandChecker::new()));
			checkers.push(Box::new(PathChecker::new()));
		}
		
		Ok(())
	}
	
	/**
	 * Processes paste with protection
	 * 
	 * @param content - Content to paste
	 * @param protection_level - Protection level
	 * @return Result<PasteResult> - Processed paste result
	 */
	pub async fn process_paste(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<PasteResult> {
		/**
		 * 保護付きでペーストを処理する関数です
		 * 
		 * 指定されたコンテンツを指定された保護レベルで処理し、
		 * セキュリティチェックを実行します。
		 * 
		 * ペースト履歴に追加され、セキュリティ分析が行われます
		 */
		
		let paste_id = Uuid::new_v4();
		let now = Utc::now();
		
		// セキュリティチェックを実行
		let security_result = self.perform_security_check(content, protection_level).await?;
		
		// ペースト結果を作成
		let paste_result = PasteResult {
			id: paste_id,
			original_content: content.to_string(),
			processed_content: security_result.processed_content,
			protection_level,
			security_result,
			created_at: now,
		};
		
		// ペースト履歴に追加
		let paste_entry = PasteEntry {
			id: paste_id,
			content: content.to_string(),
			protection_level,
			security_result: security_result.clone(),
			created_at: now,
		};
		
		{
			let mut history = self.paste_history.write().await;
			history.push(paste_entry);
			
			// 履歴サイズをチェック
			if history.len() > self.paste_config.max_paste_history {
				history.remove(0);
			}
		}
		
		Ok(paste_result)
	}
	
	/**
	 * Enables paste bracketing
	 * 
	 * @param session_id - Session ID
	 * @return Result<()> - Success or error
	 */
	pub async fn enable_paste_bracketing(&self, session_id: Uuid) -> Result<()> {
		/**
		 * ペーストブラケティングを有効にする関数です
		 * 
		 * 指定されたセッションでペーストブラケティングを有効にし、
		 * ペースト保護を強化します。
		 * 
		 * ペーストブラケティングにより、ペーストされたコンテンツが
		 * 明確に識別されます
		 */
		
		let protection_config = ProtectionConfig {
			id: Uuid::new_v4(),
			session_id,
			protection_level: PasteProtectionLevel::High,
			bracketing_enabled: true,
			created_at: Utc::now(),
		};
		
		{
			let mut configs = self.protection_configs.write().await;
			configs.insert(protection_config.id, protection_config);
		}
		
		Ok(())
	}
	
	/**
	 * Disables paste bracketing
	 * 
	 * @param session_id - Session ID
	 * @return Result<()> - Success or error
	 */
	pub async fn disable_paste_bracketing(&self, session_id: Uuid) -> Result<()> {
		/**
		 * ペーストブラケティングを無効にする関数です
		 * 
		 * 指定されたセッションでペーストブラケティングを無効にし、
		 * 通常のペースト処理に戻します。
		 * 
		 * ペーストブラケティングが無効になり、ペースト保護が
		 * 標準レベルに戻ります
		 */
		
		{
			let mut configs = self.protection_configs.write().await;
			configs.retain(|_, config| config.session_id != session_id);
		}
		
		Ok(())
	}
	
	/**
	 * Gets paste history
	 * 
	 * @return Result<Vec<PasteEntry>> - List of paste history
	 */
	pub async fn get_paste_history(&self) -> Result<Vec<PasteEntry>> {
		/**
		 * ペースト履歴を取得する関数です
		 * 
		 * すべてのペースト履歴を作成日時の順で返します。
		 * 
		 * ペースト履歴は作成日時の順でソートされます
		 */
		
		let history = self.paste_history.read().await;
		let mut history_list: Vec<PasteEntry> = history.clone();
		history_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
		
		Ok(history_list)
	}
	
	/**
	 * Clears paste history
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_paste_history(&self) -> Result<()> {
		/**
		 * ペースト履歴をクリアする関数です
		 * 
		 * すべてのペースト履歴を削除し、メモリを解放します。
		 * 
		 * 注意: この操作は取り消しできません
		 */
		
		let mut history = self.paste_history.write().await;
		history.clear();
		
		Ok(())
	}
	
	/**
	 * Gets protection count
	 * 
	 * @return Result<usize> - Number of protection configs
	 */
	pub async fn get_protection_count(&self) -> Result<usize> {
		let configs = self.protection_configs.read().await;
		Ok(configs.len())
	}
	
	/**
	 * Shuts down the paste protection manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * ペースト保護マネージャーをシャットダウンする関数です
		 * 
		 * ペースト履歴をクリアし、セキュリティチェッカーを
		 * シャットダウンします。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// ペースト履歴をクリア
		self.clear_paste_history().await?;
		
		// セキュリティチェッカーをクリア
		{
			let mut checkers = self.security_checkers.write().await;
			checkers.clear();
		}
		
		// 保護設定をクリア
		{
			let mut configs = self.protection_configs.write().await;
			configs.clear();
		}
		
		Ok(())
	}
	
	/**
	 * Performs security check on content
	 * 
	 * @param content - Content to check
	 * @param protection_level - Protection level
	 * @return Result<SecurityResult> - Security check result
	 */
	async fn perform_security_check(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<SecurityResult> {
		let mut processed_content = content.to_string();
		let mut security_issues = Vec::new();
		let mut warnings = Vec::new();
		
		// セキュリティチェッカーを実行
		let checkers = self.security_checkers.read().await;
		for checker in checkers.iter() {
			match checker.check_content(content, protection_level).await {
				Ok(check_result) => {
					if let Some(issue) = check_result.issue {
						security_issues.push(issue);
					}
					if let Some(warning) = check_result.warning {
						warnings.push(warning);
					}
					if let Some(processed) = check_result.processed_content {
						processed_content = processed;
					}
				}
				Err(e) => {
					security_issues.push(SecurityIssue {
						issue_type: "CheckerError".to_string(),
						description: e.to_string(),
						severity: SecuritySeverity::High,
					});
				}
			}
		}
		
		Ok(SecurityResult {
			processed_content,
			security_issues,
			warnings,
			protection_level,
		})
	}
}

/**
 * Paste result
 * 
 * ペースト結果の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct PasteResult {
	/// ペースト結果ID
	pub id: Uuid,
	/// 元のコンテンツ
	pub original_content: String,
	/// 処理されたコンテンツ
	pub processed_content: String,
	/// 保護レベル
	pub protection_level: PasteProtectionLevel,
	/// セキュリティ結果
	pub security_result: SecurityResult,
	/// 作成日時
	pub created_at: DateTime<Utc>,
}

/**
 * Security result
 * 
 * セキュリティ結果の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SecurityResult {
	/// 処理されたコンテンツ
	pub processed_content: String,
	/// セキュリティ問題
	pub security_issues: Vec<SecurityIssue>,
	/// 警告
	pub warnings: Vec<SecurityWarning>,
	/// 保護レベル
	pub protection_level: PasteProtectionLevel,
}

/**
 * Security issue
 * 
 * セキュリティ問題の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SecurityIssue {
	/// 問題タイプ
	pub issue_type: String,
	/// 説明
	pub description: String,
	/// 深刻度
	pub severity: SecuritySeverity,
}

/**
 * Security warning
 * 
 * セキュリティ警告の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SecurityWarning {
	/// 警告タイプ
	pub warning_type: String,
	/// 説明
	pub description: String,
	/// 深刻度
	pub severity: SecuritySeverity,
}

/**
 * Security severity
 * 
 * セキュリティ深刻度を定義します
 */
#[derive(Debug, Clone)]
pub enum SecuritySeverity {
	/// 低
	Low,
	/// 中
	Medium,
	/// 高
	High,
	/// 重大
	Critical,
}

/**
 * Check result
 * 
 * チェック結果の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct CheckResult {
	/// 処理されたコンテンツ
	pub processed_content: Option<String>,
	/// セキュリティ問題
	pub issue: Option<SecurityIssue>,
	/// 警告
	pub warning: Option<SecurityWarning>,
}

/**
 * Paste entry
 * 
 * ペーストエントリの情報を格納します
 */
#[derive(Debug, Clone)]
pub struct PasteEntry {
	/// エントリID
	pub id: Uuid,
	/// コンテンツ
	pub content: String,
	/// 保護レベル
	pub protection_level: PasteProtectionLevel,
	/// セキュリティ結果
	pub security_result: SecurityResult,
	/// 作成日時
	pub created_at: DateTime<Utc>,
}

/**
 * Protection config
 * 
 * 保護設定の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct ProtectionConfig {
	/// 設定ID
	pub id: Uuid,
	/// セッションID
	pub session_id: Uuid,
	/// 保護レベル
	pub protection_level: PasteProtectionLevel,
	/// ブラケティングが有効
	pub bracketing_enabled: bool,
	/// 作成日時
	pub created_at: DateTime<Utc>,
}

/**
 * Paste configuration
 * 
 * ペースト設定を格納します
 */
#[derive(Debug, Clone)]
pub struct PasteConfig {
	/// 保護を有効にする
	pub enable_protection: bool,
	/// ブラケティングを有効にする
	pub enable_bracketing: bool,
	/// 履歴を有効にする
	pub enable_history: bool,
	/// 最大ペースト履歴数
	pub max_paste_history: usize,
	/// ペーストタイムアウト
	pub paste_timeout: u32,
	/// デフォルト保護レベル
	pub default_protection_level: PasteProtectionLevel,
}

/**
 * Security checker trait
 * 
 * セキュリティチェッカーのトレイトを定義します
 */
#[async_trait::async_trait]
pub trait SecurityChecker: Send + Sync {
	/**
	 * Checks content for security issues
	 * 
	 * @param content - Content to check
	 * @param protection_level - Protection level
	 * @return Result<CheckResult> - Check result
	 */
	async fn check_content(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<CheckResult>;
}

/**
 * Script checker
 * 
 * スクリプトチェッカーを担当するコンポーネントです
 */
pub struct ScriptChecker {
	/// スクリプトパターン
	script_patterns: Vec<String>,
}

impl ScriptChecker {
	/**
	 * Creates a new script checker
	 * 
	 * @return ScriptChecker - New script checker instance
	 */
	pub fn new() -> Self {
		Self {
			script_patterns: vec![
				"<script".to_string(),
				"javascript:".to_string(),
				"vbscript:".to_string(),
				"data:text/html".to_string(),
			],
		}
	}
}

#[async_trait::async_trait]
impl SecurityChecker for ScriptChecker {
	async fn check_content(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<CheckResult> {
		let mut processed_content = content.to_string();
		let mut issue = None;
		
		for pattern in &self.script_patterns {
			if content.to_lowercase().contains(&pattern.to_lowercase()) {
				issue = Some(SecurityIssue {
					issue_type: "ScriptInjection".to_string(),
					description: format!("Potential script injection detected: {}", pattern),
					severity: SecuritySeverity::High,
				});
				
				// 高レベル保護ではスクリプトを削除
				if protection_level == PasteProtectionLevel::High {
					processed_content = processed_content.replace(pattern, "");
				}
			}
		}
		
		Ok(CheckResult {
			processed_content: Some(processed_content),
			issue,
			warning: None,
		})
	}
}

/**
 * URL checker
 * 
 * URLチェッカーを担当するコンポーネントです
 */
pub struct UrlChecker {
	/// URLパターン
	url_patterns: Vec<String>,
}

impl UrlChecker {
	/**
	 * Creates a new URL checker
	 * 
	 * @return UrlChecker - New URL checker instance
	 */
	pub fn new() -> Self {
		Self {
			url_patterns: vec![
				"http://".to_string(),
				"https://".to_string(),
				"ftp://".to_string(),
				"file://".to_string(),
			],
		}
	}
}

#[async_trait::async_trait]
impl SecurityChecker for UrlChecker {
	async fn check_content(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<CheckResult> {
		let mut warning = None;
		
		for pattern in &self.url_patterns {
			if content.to_lowercase().contains(&pattern.to_lowercase()) {
				warning = Some(SecurityWarning {
					warning_type: "UrlDetected".to_string(),
					description: format!("URL detected: {}", pattern),
					severity: SecuritySeverity::Low,
				});
			}
		}
		
		Ok(CheckResult {
			processed_content: None,
			issue: None,
			warning,
		})
	}
}

/**
 * Command checker
 * 
 * コマンドチェッカーを担当するコンポーネントです
 */
pub struct CommandChecker {
	/// コマンドパターン
	command_patterns: Vec<String>,
}

impl CommandChecker {
	/**
	 * Creates a new command checker
	 * 
	 * @return CommandChecker - New command checker instance
	 */
	pub fn new() -> Self {
		Self {
			command_patterns: vec![
				"rm -rf".to_string(),
				"sudo".to_string(),
				"chmod".to_string(),
				"chown".to_string(),
				"dd if=".to_string(),
			],
		}
	}
}

#[async_trait::async_trait]
impl SecurityChecker for CommandChecker {
	async fn check_content(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<CheckResult> {
		let mut issue = None;
		
		for pattern in &self.command_patterns {
			if content.to_lowercase().contains(&pattern.to_lowercase()) {
				issue = Some(SecurityIssue {
					issue_type: "DangerousCommand".to_string(),
					description: format!("Dangerous command detected: {}", pattern),
					severity: SecuritySeverity::Critical,
				});
			}
		}
		
		Ok(CheckResult {
			processed_content: None,
			issue,
			warning: None,
		})
	}
}

/**
 * Path checker
 * 
 * パスチェッカーを担当するコンポーネントです
 */
pub struct PathChecker {
	/// 危険なパスパターン
	dangerous_path_patterns: Vec<String>,
}

impl PathChecker {
	/**
	 * Creates a new path checker
	 * 
	 * @return PathChecker - New path checker instance
	 */
	pub fn new() -> Self {
		Self {
			dangerous_path_patterns: vec![
				"/etc/passwd".to_string(),
				"/etc/shadow".to_string(),
				"/root".to_string(),
				"/home/".to_string(),
			],
		}
	}
}

#[async_trait::async_trait]
impl SecurityChecker for PathChecker {
	async fn check_content(&self, content: &str, protection_level: PasteProtectionLevel) -> Result<CheckResult> {
		let mut issue = None;
		
		for pattern in &self.dangerous_path_patterns {
			if content.contains(pattern) {
				issue = Some(SecurityIssue {
					issue_type: "DangerousPath".to_string(),
					description: format!("Dangerous path detected: {}", pattern),
					severity: SecuritySeverity::Medium,
				});
			}
		}
		
		Ok(CheckResult {
			processed_content: None,
			issue,
			warning: None,
		})
	}
} 