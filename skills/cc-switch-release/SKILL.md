---
name: cc-switch-release
description: Manage version sync, release commits, tags, pushes, and post-release verification for the cc-switch fork. Use when the user asks to bump or align versions, cut a new release, create or push a tag, verify GitHub Actions or Releases, or troubleshoot why a build artifact did not appear in the release for this repository.
---

# CC Switch Release

Use this skill for repo-specific release work in `cc-switch`.

## Core Rules

- Treat `origin` as the release repo: `cp-yu/cc-switch-web`.
- Treat `upstream` as the source mirror. Do not use it for release verification unless the user explicitly asks.
- Pass `-R cp-yu/cc-switch-web` to `gh` when checking Actions or Releases. The default `gh` repo may point elsewhere.
- Treat official GitHub Releases as Web-only. `Build Web Release` runs on tag push. `Desktop Release (Manual)` is manual only.
- Keep version fields aligned in:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/tauri.conf.json`
- Prefer `scripts/release-manager.mjs` over hand-editing version files when the task is version sync or release cutting.

## Standard Workflow

1. Inspect repo state first.
   Run `git status --short --branch`, `git log --oneline --decorate -5`, and if needed read [references/release-facts.md](references/release-facts.md).
2. Decide the operation.
   Use `sync` when the user only wants version fields updated.
   Use `release` when the user wants commit + tag + optional push.
3. Preserve user intent in the index.
   Do not auto-stage unrelated files.
   If releasing, require the intended code changes to be staged before calling the release tool.
4. Run the repo helper.
   Version sync only:
   `pnpm release:sync-version -- <version>`
   Full release:
   `pnpm release:cut -- <version> --push`
5. Verify the result.
   Confirm `git status` is clean, HEAD/tag moved as expected, and version values match.
   For remote verification, use:
   `gh -R cp-yu/cc-switch-web run list --limit 5`
   `gh -R cp-yu/cc-switch-web release view v<version> --json tagName,name,publishedAt,url,assets`

## Failure Handling

- If `release-manager.mjs` rejects the worktree, fix staging hygiene instead of bypassing the check.
- If a tag already exists remotely, do not rewrite it unless the user explicitly asks for destructive history changes.
- If Actions succeeded but a release is missing, verify the run and release against `cp-yu/cc-switch-web`, not the upstream repo.
- If the task involves changing release behavior itself, edit the workflow or helper script first, then cut a new tag.

## Resources

- Read [references/release-facts.md](references/release-facts.md) when you need the exact repo/remotes/workflow/version-file map.
