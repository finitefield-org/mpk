# Repository instructions

- Git コマンドは常に `/usr/bin/git` を使用してください。
- Git の認証情報は macOS キーチェーンに保存されているため、状態確認を含む
  Git 操作はサンドボックス外で実行してください。
- このリポジトリに必要な Git 操作はすべて許可されています。
- GitHub Actions や workflow ファイルを作成、更新、実行、監視、依存しないで
  ください。`.github/workflows/` は置かないでください。
- `gh run`、`gh workflow`、`workflow_dispatch` は使用しないでください。
- 検証はローカルで実行してください。通常の確認には
  `./scripts/check-fast.sh`、リリース確認には README に記載されたローカル
  Linux ゲートを使用してください。
