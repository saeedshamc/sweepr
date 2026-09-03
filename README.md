# Sweepr

Cross-platform junk file scanner and cleanup tool built with **Tauri 2**, **React**, **TypeScript**, and **Rust**.

Nothing is deleted without explicit user confirmation. Default deletion goes to the Recycle Bin / Trash.

## Features

- Rules-driven scan targets (`src-tauri/resources/scan_rules.json`) — extend without changing scanner logic
- Streaming scan progress via Tauri events
- Risk tiers: safe / caution / risky (risky & informational items are not deletable)
- Category grouping + size chart
- Dry-run mode and permanent-delete opt-in
- Audit log at the OS data directory (`sweepr/deletion_log.jsonl`)
- Docker disk usage via CLI (`docker system df`) — no direct Docker storage edits

## Develop

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Custom rules

Copy `src-tauri/resources/scan_rules.json` to your config dir as `sweepr/scan_rules.json` to override defaults at runtime without rebuilding.
