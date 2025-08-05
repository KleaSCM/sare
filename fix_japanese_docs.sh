#!/bin/bash

# Fix terrible Japanese comments that just say "this is complex" instead of actual documentation
# This script will replace all the useless comments with proper Japanese documentation

echo "Fixing terrible Japanese comments..."

# Function to replace a comment pattern with proper documentation
replace_comment() {
	local file="$1"
	local old_pattern="$2"
	local new_doc="$3"
	
	echo "Fixing $file..."
	sed -i "s|$old_pattern|$new_doc|g" "$file"
}

# Fix streams.rs write_to_stream function
replace_comment "src/terminal/io/streams.rs" \
	"この関数は複雑なI/O操作を行います。" \
	"指定されたストリームIDのファイルディスクリプタにデータを書き込み、同時にバッファにもデータを追加してキャッシュ機能を提供します。ストリームの状態（Open、Closed、Error）に応じて適切な処理を行い、書き込み失敗時はストリーム状態をErrorに変更してエラーを返します。"

# Fix session.rs stop_session function  
replace_comment "src/tui/panes/session.rs" \
	"この関数は複雑なリソース管理を行います。" \
	"指定されたセッションIDのシェルセッションを段階的に終了し、ターミナルエミュレーターの停止、セッション状態の更新、メタデータの保存を行います。セッション状態をStoppingからTerminatedに変更し、最後のアクティビティ時刻を更新してセッション履歴に終了イベントを記録します。"

# Fix session.rs synchronize_sessions function
replace_comment "src/tui/panes/session.rs" \
	"この関数は複雑なセッション間通信を行います。" \
	"指定されたセッションIDリストのセッション間で環境変数の共有、作業ディレクトリの同期、セッション状態の調整を行います。セッション調整機能が有効な場合のみ実行され、各セッションの環境変数、作業ディレクトリ、セッション状態を統一します。"

# Fix layout.rs new function
replace_comment "src/tui/panes/layout.rs" \
	"この関数は複雑なレイアウト計算を行います。" \
	"指定されたレイアウトアルゴリズムと制約設定を使用してレイアウトマネージャーを初期化し、空のレイアウトツリーを作成します。アルゴリズム（BinaryTree、Grid、Manual）と制約設定（最小サイズ、間隔、アスペクト比など）を設定し、ペイン配置計算の準備を整えます。"

# Fix layout.rs calculate_layout function
replace_comment "src/tui/panes/layout.rs" \
	"この関数は複雑なレイアウトアルゴリズムを実行します。" \
	"現在のレイアウトアルゴリズムと制約設定に基づいて、指定されたペインIDリストの最適な配置を計算します。BinaryTree、Grid、Manualの各アルゴリズムに対応し、制約設定（最小サイズ、間隔、アスペクト比）を適用して各ペインの位置とサイズを決定します。"

echo "Japanese documentation fixes completed!" 