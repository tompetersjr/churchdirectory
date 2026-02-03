# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A cross-platform desktop application for managing and publishing church photo directories. Built with Tauri 2 (Rust backend) and Vue 3 (TypeScript frontend).

## Commands

```bash
npm install              # Install dependencies
npm run tauri dev        # Run in development mode (hot reload)
npm run tauri build      # Build production executable
npm run build            # Build frontend only (with TypeScript check)
npm run dev              # Run Vite dev server only (no Tauri)
```

Development server runs at `http://localhost:1420`. Frontend changes hot reload; Rust changes require restart.

## Architecture

### Frontend (src/)
- **Framework:** Vue 3 + TypeScript + Vite
- **State Management:** Pinia stores in `src/stores/`
- **Routing:** Vue Router in `src/router.ts`
- **Styling:** Tailwind CSS 4.1
- **Types:** All interfaces in `src/types.ts`

### Backend (src-tauri/)
- **Framework:** Tauri 2 + Rust
- **Database:** SQLite via rusqlite (bundled)
- **Key modules:**
  - `db.rs` - Database initialization and queries
  - `import.rs` - XLSX parsing with calamine
  - `photos.rs` - Image upload, resize, and storage
  - `pdf.rs` - PDF generation with printpdf
  - `backup.rs` - ZIP archive backup/restore
  - `commands/` - Tauri IPC command handlers

### IPC Pattern
Frontend invokes Rust commands via `invoke()` from `@tauri-apps/api/core`:
```typescript
import { invoke } from '@tauri-apps/api/core';
const families = await invoke<Family[]>('get_families');
```

Commands are registered in `src-tauri/src/lib.rs` and implemented in `src-tauri/src/commands/`.

## Database

SQLite database at `{APP_DATA}/directory.db` with tables:
- `families` - Family records with address, contact info, photo path
- `members` - Individual members linked to families via `family_id`
- `import_history` - Tracks XLSX imports

Key relationships: Members belong to families via foreign key with `ON DELETE CASCADE`.

## File Storage

Photos stored in app data directory:
- Family photos: `photos/families/{family_id}/`
- Member photos: `photos/members/{member_id}/`

Images auto-resized to max 1200px width on upload.

## Key Data Flow

1. **Family/Member CRUD:** Vue components → Pinia store → Tauri invoke → Rust commands → SQLite
2. **XLSX Import:** File picker → calamine parser → duplicate detection → batch insert
3. **PDF Generation:** User options → Rust assembles content → printpdf renders → file save dialog
4. **Backup:** ZIP creation with database, photos, settings, and manifest.json

## TypeScript Configuration

Strict mode enabled. Path alias `@/*` maps to `./src/*`.
