# Security Policy

## Supported Versions

Whitebaseは開発中のため、原則として最新の`main`ブランチのみを対象にセキュリティ修正を行います。

| Version | Supported |
| ------- | --------- |
| main    | Yes       |
| 過去のコミット・リリース | No |

## Reporting a Vulnerability

セキュリティ上の問題を発見した場合は、公開Issueへ詳細を書き込まないでください。

GitHubのPrivate vulnerability reporting、またはリポジトリのSecurity Advisoriesから非公開で報告してください。

報告には、可能な範囲で以下を含めてください。

- 問題の概要
- 影響を受けるコンポーネント
- 再現手順
- 想定される影響
- 回避策または修正案
- 関連するログやスクリーンショット

秘密鍵、アクセストークン、個人情報などは含めないでください。

## Response Process

報告内容を確認後、以下を行います。

1. 問題を再現・評価します
2. 影響範囲を確認します
3. 修正または回避策を準備します
4. 必要に応じてSecurity Advisoryを公開します
5. 修正版を`main`へ反映します

## Scope

主な対象範囲は以下です。

- Whitebase Core
- Whitebase Server
- Whitebase Control Center
- Desktop App
- Web App
- Language Adapters
- CI/CD設定
- 依存関係とビルド環境

通常の不具合や機能提案は、公開Issueのテンプレートを使用してください。
