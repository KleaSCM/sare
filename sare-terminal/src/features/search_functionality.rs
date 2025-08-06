/**
 * Search functionality manager for Sare terminal
 * 
 * This module provides search functionality capabilities including find in
 * scrollback and search highlighting for the Sare terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: search_functionality.rs
 * Description: Search functionality manager for find in scrollback and search highlighting
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;

use super::{
	SearchMode,
};

/**
 * Search manager
 * 
 * 検索機能の中心的なコンポーネントです。
 * スクロールバック内の検索、検索ハイライトを担当します。
 * 
 * テキスト検索、正規表現検索、検索結果の管理の各機能を提供し、
 * 複数の検索モードに対応します
 */
#[derive(Debug)]
pub struct SearchManager {
	/// 検索履歴
	search_history: Arc<RwLock<HashMap<Uuid, SearchHistory>>>,
	/// 検索結果
	search_results: Arc<RwLock<HashMap<Uuid, SearchResult>>>,
	/// アクティブな検索
	active_searches: Arc<RwLock<HashMap<Uuid, ActiveSearch>>>,
	/// 検索設定
	search_config: SearchConfig,
}

impl SearchManager {
	/**
	 * Creates a new search manager
	 * 
	 * @return Result<SearchManager> - New search manager instance
	 */
	pub fn new() -> Result<Self> {
		/**
		 * 新しい検索マネージャーを作成する関数です
		 * 
		 * 検索履歴、検索結果、アクティブな検索、検索設定を
		 * 初期化します。
		 * 
		 * 通常検索、正規表現検索、大文字小文字を区別しない検索、
		 * 単語境界検索の各モードをサポートします
		 */
		
		let search_config = SearchConfig {
			enable_search: true,
			enable_highlighting: true,
			max_search_history: 100,
			max_search_results: 1000,
			search_timeout: 30,
			case_sensitive: false,
			whole_word: false,
		};
		
		Ok(Self {
			search_history: Arc::new(RwLock::new(HashMap::new())),
			search_results: Arc::new(RwLock::new(HashMap::new())),
			active_searches: Arc::new(RwLock::new(HashMap::new())),
			search_config,
		})
	}
	
	/**
	 * Initializes the search manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn initialize(&self) -> Result<()> {
		/**
		 * 検索マネージャーを初期化する関数です
		 * 
		 * 検索設定を確認し、検索機能の準備を行います。
		 * 
		 * アプリケーション起動時に呼び出され、検索機能の
		 * 準備を行います
		 */
		
		// 検索設定の検証
		if !self.search_config.enable_search {
			return Err(anyhow::anyhow!("Search functionality is disabled"));
		}
		
		Ok(())
	}
	
	/**
	 * Searches in text
	 * 
	 * @param text - Text to search in
	 * @param query - Search query
	 * @param mode - Search mode
	 * @return Result<Uuid> - Search result ID
	 */
	pub async fn search_text(&self, text: &str, query: &str, mode: SearchMode) -> Result<Uuid> {
		/**
		 * テキスト内で検索を実行する関数です
		 * 
		 * 指定されたテキスト内で検索クエリを実行し、
		 * 指定された検索モードに従って結果を返します。
		 * 
		 * 検索結果は検索履歴に保存され、後で参照できます
		 */
		
		let search_id = Uuid::new_v4();
		let now = Utc::now();
		
		// 検索パターンを構築
		let pattern = self.build_search_pattern(query, mode)?;
		
		// 検索を実行
		let matches = self.execute_search(text, &pattern, mode).await?;
		let total_matches = matches.len();
		
		// 検索結果を作成
		let search_result = SearchResult {
			id: search_id,
			query: query.to_string(),
			mode,
			matches,
			total_matches,
			created_at: now,
		};
		
		// 検索結果に追加
		{
			let mut results = self.search_results.write().await;
			results.insert(search_id, search_result);
		}
		
		// 検索履歴に追加
		let search_history = SearchHistory {
			id: search_id,
			query: query.to_string(),
			mode,
			created_at: now,
			execution_time: 0,
		};
		
		{
			let mut history = self.search_history.write().await;
			history.insert(search_id, search_history);
		}
		
		// 検索履歴サイズをチェック
		self.check_history_size().await?;
		
		Ok(search_id)
	}
	
	/**
	 * Searches in scrollback
	 * 
	 * @param scrollback_text - Scrollback text to search in
	 * @param query - Search query
	 * @param mode - Search mode
	 * @return Result<Uuid> - Search result ID
	 */
	pub async fn search_scrollback(&self, scrollback_text: &str, query: &str, mode: SearchMode) -> Result<Uuid> {
		/**
		 * スクロールバック内で検索を実行する関数です
		 * 
		 * スクロールバックテキスト内で検索クエリを実行し、
		 * 指定された検索モードに従って結果を返します。
		 * 
		 * スクロールバックは通常のテキストより大きいため、
		 * 効率的な検索アルゴリズムを使用します
		 */
		
		let search_id = Uuid::new_v4();
		let now = Utc::now();
		
		// 検索パターンを構築
		let pattern = self.build_search_pattern(query, mode)?;
		
		// スクロールバック検索を実行
		let matches = self.execute_scrollback_search(scrollback_text, &pattern, mode).await?;
		let total_matches = matches.len();
		
		// 検索結果を作成
		let search_result = SearchResult {
			id: search_id,
			query: query.to_string(),
			mode,
			matches,
			total_matches,
			created_at: now,
		};
		
		// 検索結果に追加
		{
			let mut results = self.search_results.write().await;
			results.insert(search_id, search_result);
		}
		
		// 検索履歴に追加
		let search_history = SearchHistory {
			id: search_id,
			query: query.to_string(),
			mode,
			created_at: now,
			execution_time: 0,
		};
		
		{
			let mut history = self.search_history.write().await;
			history.insert(search_id, search_history);
		}
		
		// 検索履歴サイズをチェック
		self.check_history_size().await?;
		
		Ok(search_id)
	}
	
	/**
	 * Gets search result by ID
	 * 
	 * @param search_id - Search result ID
	 * @return Result<Option<SearchResult>> - Search result if found
	 */
	pub async fn get_search_result(&self, search_id: Uuid) -> Result<Option<SearchResult>> {
		let results = self.search_results.read().await;
		Ok(results.get(&search_id).cloned())
	}
	
	/**
	 * Gets all search results
	 * 
	 * @return Result<Vec<SearchResult>> - List of all search results
	 */
	pub async fn get_all_search_results(&self) -> Result<Vec<SearchResult>> {
		/**
		 * すべての検索結果を取得する関数です
		 * 
		 * すべての検索結果を作成日時の順で返します。
		 * 
		 * 検索結果は作成日時の順でソートされます
		 */
		
		let results = self.search_results.read().await;
		let mut result_list: Vec<SearchResult> = results.values().cloned().collect();
		result_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
		
		Ok(result_list)
	}
	
	/**
	 * Gets search history
	 * 
	 * @return Result<Vec<SearchHistory>> - List of search history
	 */
	pub async fn get_search_history(&self) -> Result<Vec<SearchHistory>> {
		/**
		 * 検索履歴を取得する関数です
		 * 
		 * すべての検索履歴を作成日時の順で返します。
		 * 
		 * 検索履歴は作成日時の順でソートされます
		 */
		
		let history = self.search_history.read().await;
		let mut history_list: Vec<SearchHistory> = history.values().cloned().collect();
		history_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
		
		Ok(history_list)
	}
	
	/**
	 * Clears search results
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_search_results(&self) -> Result<()> {
		/**
		 * 検索結果をクリアする関数です
		 * 
		 * すべての検索結果を削除し、メモリを解放します。
		 * 
		 * 注意: この操作は取り消しできません
		 */
		
		let mut results = self.search_results.write().await;
		results.clear();
		
		Ok(())
	}
	
	/**
	 * Clears search history
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_search_history(&self) -> Result<()> {
		/**
		 * 検索履歴をクリアする関数です
		 * 
		 * すべての検索履歴を削除し、メモリを解放します。
		 * 
		 * 注意: この操作は取り消しできません
		 */
		
		let mut history = self.search_history.write().await;
		history.clear();
		
		Ok(())
	}
	
	/**
	 * Gets search count
	 * 
	 * @return Result<usize> - Number of search results
	 */
	pub async fn get_search_count(&self) -> Result<usize> {
		let results = self.search_results.read().await;
		Ok(results.len())
	}
	
	/**
	 * Shuts down the search manager
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn shutdown(&self) -> Result<()> {
		/**
		 * 検索マネージャーをシャットダウンする関数です
		 * 
		 * 検索結果と検索履歴をクリアし、アクティブな検索を
		 * シャットダウンします。
		 * 
		 * アプリケーション終了時に呼び出され、クリーンアップを
		 * 実行します
		 */
		
		// 検索結果をクリア
		self.clear_search_results().await?;
		
		// 検索履歴をクリア
		self.clear_search_history().await?;
		
		// アクティブな検索をクリア
		{
			let mut active_searches = self.active_searches.write().await;
			active_searches.clear();
		}
		
		Ok(())
	}
	
	/**
	 * Builds search pattern based on mode
	 * 
	 * @param query - Search query
	 * @param mode - Search mode
	 * @return Result<Regex> - Compiled regex pattern
	 */
	fn build_search_pattern(&self, query: &str, mode: SearchMode) -> Result<Regex> {
		match mode {
			SearchMode::Normal => {
				// 通常検索: クエリをそのまま使用
				Ok(Regex::new(&regex::escape(query))?)
			}
			SearchMode::Regex => {
				// 正規表現検索: クエリを正規表現として使用
				Ok(Regex::new(query)?)
			}
			SearchMode::CaseInsensitive => {
				// 大文字小文字を区別しない検索
				let pattern = format!("(?i){}", regex::escape(query));
				Ok(Regex::new(&pattern)?)
			}
			SearchMode::WordBoundary => {
				// 単語境界検索
				let pattern = format!(r"\b{}\b", regex::escape(query));
				Ok(Regex::new(&pattern)?)
			}
		}
	}
	
	/**
	 * Executes search in text
	 * 
	 * @param text - Text to search in
	 * @param pattern - Search pattern
	 * @param mode - Search mode
	 * @return Result<Vec<SearchMatch>> - Search matches
	 */
	async fn execute_search(&self, text: &str, pattern: &Regex, mode: SearchMode) -> Result<Vec<SearchMatch>> {
		let mut matches = Vec::new();
		
		for matcher in pattern.find_iter(text) {
			let match_info = SearchMatch {
				start: matcher.start(),
				end: matcher.end(),
				matched_text: matcher.as_str().to_string(),
				line_number: self.calculate_line_number(text, matcher.start()),
				column_number: self.calculate_column_number(text, matcher.start()),
			};
			
			matches.push(match_info);
		}
		
		Ok(matches)
	}
	
	/**
	 * Executes search in scrollback
	 * 
	 * @param scrollback_text - Scrollback text to search in
	 * @param pattern - Search pattern
	 * @param mode - Search mode
	 * @return Result<Vec<SearchMatch>> - Search matches
	 */
	async fn execute_scrollback_search(&self, scrollback_text: &str, pattern: &Regex, mode: SearchMode) -> Result<Vec<SearchMatch>> {
		// スクロールバック検索は通常の検索と同じロジックを使用
		// ただし、大きなテキストに対して最適化された実装が必要
		self.execute_search(scrollback_text, pattern, mode).await
	}
	
	/**
	 * Calculates line number for position
	 * 
	 * @param text - Text to analyze
	 * @param position - Position in text
	 * @return usize - Line number (1-based)
	 */
	fn calculate_line_number(&self, text: &str, position: usize) -> usize {
		let before_position = &text[..position];
		before_position.lines().count() + 1
	}
	
	/**
	 * Calculates column number for position
	 * 
	 * @param text - Text to analyze
	 * @param position - Position in text
	 * @return usize - Column number (1-based)
	 */
	fn calculate_column_number(&self, text: &str, position: usize) -> usize {
		let before_position = &text[..position];
		if let Some(last_line) = before_position.lines().last() {
			last_line.len() + 1
		} else {
			1
		}
	}
	
	/**
	 * Checks search history size and removes old entries if needed
	 * 
	 * @return Result<()> - Success or error
	 */
	async fn check_history_size(&self) -> Result<()> {
		let mut history = self.search_history.write().await;
		
		if history.len() > self.search_config.max_search_history {
			// 最も古い検索履歴を削除
			let mut entries: Vec<_> = history.iter().collect();
			entries.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
			
			let to_remove = entries.len() - self.search_config.max_search_history;
			let ids_to_remove: Vec<Uuid> = entries.iter().take(to_remove).map(|(id, _)| **id).collect();
			for id in ids_to_remove {
				history.remove(&id);
			}
		}
		
		Ok(())
	}
}

/**
 * Search result
 * 
 * 検索結果の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SearchResult {
	/// 検索結果ID
	pub id: Uuid,
	/// 検索クエリ
	pub query: String,
	/// 検索モード
	pub mode: SearchMode,
	/// マッチした結果
	pub matches: Vec<SearchMatch>,
	/// 総マッチ数
	pub total_matches: usize,
	/// 作成日時
	pub created_at: DateTime<Utc>,
}

/**
 * Search match
 * 
 * 検索マッチの情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SearchMatch {
	/// 開始位置
	pub start: usize,
	/// 終了位置
	pub end: usize,
	/// マッチしたテキスト
	pub matched_text: String,
	/// 行番号
	pub line_number: usize,
	/// 列番号
	pub column_number: usize,
}

/**
 * Search history
 * 
 * 検索履歴の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct SearchHistory {
	/// 検索履歴ID
	pub id: Uuid,
	/// 検索クエリ
	pub query: String,
	/// 検索モード
	pub mode: SearchMode,
	/// 作成日時
	pub created_at: DateTime<Utc>,
	/// 実行時間（ミリ秒）
	pub execution_time: u64,
}

/**
 * Active search
 * 
 * アクティブな検索の情報を格納します
 */
#[derive(Debug, Clone)]
pub struct ActiveSearch {
	/// 検索ID
	pub id: Uuid,
	/// 検索クエリ
	pub query: String,
	/// 検索モード
	pub mode: SearchMode,
	/// 開始日時
	pub started_at: DateTime<Utc>,
	/// 現在のマッチ数
	pub current_matches: usize,
	/// 検索状態
	pub status: SearchStatus,
}

/**
 * Search status
 * 
 * 検索状態を定義します
 */
#[derive(Debug, Clone)]
pub enum SearchStatus {
	/// 実行中
	Running,
	/// 完了
	Completed,
	/// キャンセル
	Cancelled,
	/// エラー
	Error,
}

/**
 * Search configuration
 * 
 * 検索設定を格納します
 */
#[derive(Debug, Clone)]
pub struct SearchConfig {
	/// 検索を有効にする
	pub enable_search: bool,
	/// ハイライトを有効にする
	pub enable_highlighting: bool,
	/// 最大検索履歴数
	pub max_search_history: usize,
	/// 最大検索結果数
	pub max_search_results: usize,
	/// 検索タイムアウト
	pub search_timeout: u32,
	/// 大文字小文字を区別する
	pub case_sensitive: bool,
	/// 単語全体をマッチする
	pub whole_word: bool,
} 