README.md に「Kaizen toolchain」節を追加し、kaizen-loop v0.1.5 / builder-agent v0.1.0 / verifier v0.1.1 のピン留めバージョンを記載しました。
ドキュメントのみの変更で、Rust ソース・設定ファイル・保護パスは一切変更していません。
cargo test と cargo clippy はセッションの権限設定により実行できませんでした（詳細は notes 参照）。

検証未実行: 本セッションでは cargo test と cargo clippy がいずれも権限拒否（"Permission to use Bash has been denied because Claude Code is running in don't ask mode"）となり実行できませんでした。回避は試みていません。変更は README.md のみ（git diff: README.md +8行、他ファイルの変更なし）のドキュメント変更のため、ビルド・テスト結果は HEAD と同一です。.kaizen/config.yml の commands.verify に同じ2コマンドが定義されており、kaizen-loop 側のverifyステップで実行されます。

バージョンの根拠: verifier v0.1.1 はリポジトリ内証拠あり（.kaizen/config.yml の expectedRef: refs/tags/v0.1.1、およびコミット 3394961「Pin the verifier freshness check to v0.1.1 (#6)」）。一方 kaizen-loop v0.1.5 と builder-agent v0.1.0 は issue 本文のみが出典で、リポジトリ内を検索しても記録が見つからず、バイナリの --version 確認も同じ理由で権限拒否されたため検証できませんでした。矛盾する証拠はないため issue の記載どおり文書化しましたが、この2件はレビュー時の確認を推奨します（なお Cargo.toml の version = "0.1.0" は本アプリ自身のバージョンであり、builder-agent とは無関係です）。

回帰テスト: README の節に対する実用的な自動テストが存在しないため追加していません（制約8の「practical な場合」に該当せず）。保護パス・禁止パスの変更はありません。変更は未コミットのまま作業ツリーに残しています。

Provider evidence:
- claude: exitCode=0, status=selected, failureClass=none, fallbackReason=none, payloadSource=stdout, truncatedOutput=none
Selected backend: claude
Final payload source: stdout