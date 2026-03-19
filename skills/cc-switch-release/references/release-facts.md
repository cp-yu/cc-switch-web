# CC Switch Release Facts

## Repos and remotes

- Release repo for this workspace:
  - `origin = git@github.com:cp-yu/cc-switch-web.git`
- Upstream mirror:
  - `upstream = https://github.com/farion1231/cc-switch.git`
- Use `gh -R cp-yu/cc-switch-web ...` for release and Actions checks.

## Version sources

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

These three files must stay aligned for each release.

## Release helper

- Primary helper:
  - `scripts/release-manager.mjs`
- Package scripts:
  - `pnpm release:sync-version -- <version>`
  - `pnpm release:cut -- <version> --push`

`release-manager.mjs` will:

- validate semver input
- require a clean worktree except for staged changes
- update the managed version files
- stage managed version files
- create the release commit
- create an annotated tag
- optionally push branch and tag

## GitHub workflows

- Automatic official release workflow:
  - `Build Web Release`
- Manual desktop workflow:
  - `Desktop Release (Manual)`

`Build Web Release` is the official release path for this fork and publishes Web runtime assets only.

## Official release assets

- Windows:
  - `cc-switch-web-v{version}-windows-x86_64.exe`
- Linux:
  - `cc-switch-web-v{version}-linux-x86_64-ubuntu20.04`

## Verification commands

```bash
git status --short --branch
git log --oneline --decorate -5
node -p "require('./package.json').version"
node -p "require('./src-tauri/tauri.conf.json').version"
sed -n '1,8p' src-tauri/Cargo.toml
gh -R cp-yu/cc-switch-web run list --limit 5
gh -R cp-yu/cc-switch-web release view v<version> --json tagName,name,publishedAt,url,assets
```
