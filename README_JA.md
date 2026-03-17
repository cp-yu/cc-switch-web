<div align="center">

# CC Switch Fork

[English](README.md) | [中文](README_ZH.md) | 日本語

このリポジトリの GitHub Releases は Web 実行版のみを正式公開します。

</div>

## 位置づけ

この fork はローカルカスタマイズと継続開発向けです。現在のコードベースは次を提供します。

- Claude Code、Codex、Gemini、OpenCode、OpenClaw の設定管理
- MCP、Prompts、Skills、プロキシ、フェイルオーバー、使用量関連機能
- 正式配布物としての単一バイナリ Web ランタイム
- ローカル開発専用としてリポジトリに残す Tauri デスクトップコード

## スクリーンショット

|                  メイン画面                   |                  プロバイダ追加                  |
| :-------------------------------------------: | :----------------------------------------------: |
| ![Main Interface](assets/screenshots/main-en.png) | ![Add Provider](assets/screenshots/add-en.png) |

## 正式リリース資産

GitHub Releases では Web ランタイムのみを公開します。

| プラットフォーム | アセット名 | 実行方法 |
| --- | --- | --- |
| Windows x86_64 | `cc-switch-web-v{version}-windows-x86_64.exe` | `./cc-switch-web-v{version}-windows-x86_64.exe` |
| macOS x86_64 | `cc-switch-web-v{version}-macos-x86_64` | `chmod +x ./cc-switch-web-v{version}-macos-x86_64 && ./cc-switch-web-v{version}-macos-x86_64` |
| Linux x86_64 | `cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` | `chmod +x ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04 && ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` |

### 既定値

- URL: `http://127.0.0.1:17666`
- ポート変更: `CC_SWITCH_PORT=8080`
- ホスト変更: `CC_SWITCH_HOST=0.0.0.0`
- Linux 互換ベースライン: Ubuntu 20.04+

### プラットフォーム注記

- Windows: `.exe` をそのまま実行します。
- macOS: 署名なしバイナリです。Gatekeeper に遮断された場合は隔離属性を削除してください。
- Linux: 正式アセットは Ubuntu 20.04 上でビルドし、最低互換ラインを明示します。

```bash
xattr -d com.apple.quarantine ./cc-switch-web-v{version}-macos-x86_64
```

## ローカル開発

### 必要環境

- Node.js 18+
- pnpm 8+ または npm
- Rust 1.85+
- Tauri CLI 2.8+（デスクトップのローカル開発時のみ）

### よく使うコマンド

```bash
# 依存関係をインストール
pnpm install

# Web 開発
pnpm dev:server
pnpm dev:web

# 型チェック
pnpm typecheck

# フロントエンド単体テスト
pnpm test:unit

# 埋め込み Web フロントエンドをビルド
pnpm build:web
```

### ローカル Web 起動

```bash
./start-web.sh
```

起動後のアクセス先：

```text
http://localhost:17666
```

停止：

```bash
./stop-web.sh
```

実行時ファイルは既定で `./.run/web/` に書き込まれます。

- ログ：`backend.log`
- PID：`backend.pid`

別ディレクトリを使う場合：

```bash
CC_SWITCH_RUNTIME_DIR=/tmp/cc-switch-web ./start-web.sh
```

### 手動で Web ビルド

```bash
pnpm build:web
cargo build --release --manifest-path crates/server/Cargo.toml
./crates/server/target/release/cc-switch-web
```

### ローカル Linux リリース互換ビルド

```bash
./build-web-release.sh
```

このスクリプトは `release-web/cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` を出力します。

## 技術スタック

- フロントエンド：React 18、TypeScript、Vite、TailwindCSS、TanStack Query
- バックエンド：Tauri 2、Rust、tokio、serde
- テスト：vitest、MSW、@testing-library/react

## ディレクトリ構成

```text
src/                 フロントエンドコード
src-tauri/           Tauri デスクトップバックエンド
crates/server/       Web サーバー
crates/core/         共通コアロジック
tests/               フロントエンドテスト
assets/              スクリーンショットなどの素材
docs/                補助ドキュメント
```

## ライセンス

[LICENSE](LICENSE) を参照してください。
