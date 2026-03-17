<div align="center">

# CC Switch Fork

English | [中文](README_ZH.md) | [日本語](README_JA.md)

This fork keeps the Web runtime as the only official GitHub Release deliverable.

</div>

## Scope

This fork is used for local customization and ongoing development. The current codebase provides:

- Configuration management for Claude Code, Codex, Gemini, OpenCode, and OpenClaw
- MCP, prompts, skills, proxy, failover, and usage-related features
- A single-binary Web runtime for official releases
- Tauri desktop code kept in-repo for local development only

## Screenshots

|                  Main Interface                   |                  Add Provider                  |
| :-----------------------------------------------: | :--------------------------------------------: |
| ![Main Interface](assets/screenshots/main-en.png) | ![Add Provider](assets/screenshots/add-en.png) |

## Official Release Assets

GitHub Releases publish the Web runtime only.

| Platform | Asset | Run |
| --- | --- | --- |
| Windows x86_64 | `cc-switch-web-v{version}-windows-x86_64.exe` | `./cc-switch-web-v{version}-windows-x86_64.exe` |
| macOS x86_64 | `cc-switch-web-v{version}-macos-x86_64` | `chmod +x ./cc-switch-web-v{version}-macos-x86_64 && ./cc-switch-web-v{version}-macos-x86_64` |
| Linux x86_64 | `cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` | `chmod +x ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04 && ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` |

### Runtime defaults

- URL: `http://127.0.0.1:17666`
- Port override: `CC_SWITCH_PORT=8080`
- Host override: `CC_SWITCH_HOST=0.0.0.0`
- Linux compatibility baseline: Ubuntu 20.04+

### Platform notes

- Windows: run the `.exe` directly in PowerShell or Command Prompt.
- macOS: the binary is unsigned. If Gatekeeper blocks it, remove the quarantine attribute and run again.
- Linux: official assets are built on Ubuntu 20.04 to keep the minimum supported baseline explicit.

```bash
xattr -d com.apple.quarantine ./cc-switch-web-v{version}-macos-x86_64
```

## Local Development

### Requirements

- Node.js 18+
- pnpm 8+ or npm
- Rust 1.85+
- Tauri CLI 2.8+ for desktop-only local development

### Common Commands

```bash
# Install dependencies
pnpm install

# Web development
pnpm dev:server
pnpm dev:web

# Type checking
pnpm typecheck

# Frontend unit tests
pnpm test:unit

# Build embedded Web frontend
pnpm build:web
```

### Local Web Launch

```bash
./start-web.sh
```

Then open:

```text
http://localhost:17666
```

Stop the service:

```bash
./stop-web.sh
```

Runtime files are written to `./.run/web/` by default:

- log: `backend.log`
- pid: `backend.pid`

To override the runtime directory:

```bash
CC_SWITCH_RUNTIME_DIR=/tmp/cc-switch-web ./start-web.sh
```

### Manual Web Build

```bash
pnpm build:web
cargo build --release --manifest-path crates/server/Cargo.toml
./crates/server/target/release/cc-switch-web
```

### Local Linux release-parity build

```bash
./build-web-release.sh
```

This script emits `release-web/cc-switch-web-v{version}-linux-x86_64-ubuntu20.04`.

## Tech Stack

- Frontend: React 18, TypeScript, Vite, TailwindCSS, TanStack Query
- Backend: Tauri 2, Rust, tokio, serde
- Testing: vitest, MSW, @testing-library/react

## Project Layout

```text
src/                 frontend code
src-tauri/           Tauri desktop backend
crates/server/       web server
crates/core/         shared core logic
tests/               frontend tests
assets/              screenshots and assets
docs/                supplementary documentation
```

## License

See [LICENSE](LICENSE).
