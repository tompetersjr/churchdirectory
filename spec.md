# Church Photo Directory Application

## Overview

A cross-platform desktop application for managing and publishing a church photo directory. Built with Tauri, Vue 3, and Tailwind CSS 4.1.

## Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Framework | Tauri | 2.x |
| Frontend | Vue 3 | 3.x |
| Build Tool | Vite | 5.x |
| CSS | Tailwind CSS | 4.1 |
| Language (Backend) | Rust | Latest stable |
| Database | SQLite | 3.x |
| PDF Generation | `printpdf` (Rust) or `pdf-lib` (JS) | Latest |
| XLSX Parsing | `calamine` (Rust) or `xlsx` (JS) | Latest |

## Data Model

### Database Schema

```sql
-- Families table (groups members by Family ID)
CREATE TABLE families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_id TEXT UNIQUE NOT NULL,        -- From XLSX "Family ID"
    mailing_name TEXT,                      -- From XLSX "Mailing Name"
    address TEXT,
    city TEXT,
    state TEXT,
    zip_code TEXT,
    alt_address TEXT,
    alt_city TEXT,
    alt_state TEXT,
    alt_zip_code TEXT,
    phone TEXT,
    include_in_directory BOOLEAN DEFAULT 1,
    family_photo_path TEXT,                 -- Optional family photo
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Members table (individual people)
CREATE TABLE members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_id INTEGER NOT NULL,             -- FK to families.id
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT,
    cell_phone TEXT,
    birth_date DATE,
    wedding_date DATE,
    photo_path TEXT,                        -- Individual photo
    is_child BOOLEAN DEFAULT 0,             -- Derived from "Children" field
    include_in_directory BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE
);

-- Import history for tracking imports
CREATE TABLE import_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    records_imported INTEGER,
    records_updated INTEGER,
    imported_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### XLSX Column Mapping

| XLSX Header | Database Field | Table |
|-------------|----------------|-------|
| Family ID | family_id | families |
| Mailing Name | mailing_name | families |
| Address | address | families |
| City | city | families |
| State | state | families |
| Zip Code | zip_code | families |
| Alt Address | alt_address | families |
| Alt City | alt_city | families |
| Alt State | alt_state | families |
| Alt Zip Code | alt_zip_code | families |
| Phone | phone | families |
| Include in Directory | include_in_directory | families & members |
| Last Name | last_name | members |
| First Name | first_name | members |
| E-Mail | email | members |
| Cell Phone | cell_phone | members |
| Birth Date | birth_date | members |
| Wedding Date | wedding_date | members |
| Picture | photo_path | members |
| Children | is_child (parsed) | members |

## Features

### 1. XLSX Import

**Requirements:**
- Parse XLSX files with the specified column headers
- Handle missing or extra columns gracefully
- Detect existing records by `Family ID` + `First Name` + `Last Name`
- Update existing records or insert new ones
- Display import preview before committing
- Show import summary (new records, updated records, errors)
- Maintain import history log

**Import Logic:**
1. Read XLSX file
2. Validate required columns exist
3. Group rows by Family ID
4. For each family:
   - Check if family exists (by Family ID)
   - Update or create family record
   - Process each member in the family
   - Check if member exists (by family + name)
   - Update or create member record
5. Report results

### 2. Family & Member Management

**Family List View:**
- Display all families in searchable/filterable list
- Search by family name, member name, address
- Filter by: has photo, include in directory, no members
- Sort by: family name, date added, date modified
- Quick actions: edit, delete, toggle directory inclusion

**Family Detail/Edit View:**
- **Editable Family Fields:**
  - Mailing Name
  - Address (street, city, state, zip)
  - Alternate Address (street, city, state, zip)
  - Phone number
  - Include in Directory (checkbox)
  - Family Photo (upload/remove)
- View list of family members
- Add new member to family
- Remove member from family
- Delete entire family (with confirmation)

**Member Detail/Edit View:**
- **Editable Member Fields:**
  - First Name
  - Last Name
  - Email
  - Cell Phone
  - Birth Date (date picker)
  - Wedding Date (date picker)
  - Is Child (checkbox)
  - Include in Directory (checkbox)
  - Photo (upload/remove)
- Move member to different family
- Delete member (with confirmation)

**Add New Family:**
- Create family with required Family ID (auto-generate or manual)
- Enter all family fields
- Optionally add members during creation

**Add New Member:**
- Select existing family or create new
- Enter all member fields
- Optionally upload photo during creation

**Inline Editing:**
- Quick edit mode for common fields directly in list view
- Toggle directory inclusion with single click
- Bulk actions: select multiple families/members for:
  - Include/exclude from directory
  - Delete

**Data Validation:**
- Required fields: Family ID, First Name, Last Name
- Email format validation
- Phone number format suggestions
- Date validation for birth/wedding dates
- Duplicate detection warnings

### 3. Photo Management

**Requirements:**
- Support common formats: JPG, PNG, WEBP
- Store photos in app data directory: `{APP_DATA}/photos/`
- Naming convention: `{family_id}_{member_id}_{timestamp}.{ext}`
- Auto-resize/compress for storage efficiency (max 1200px width)
- Generate thumbnails for list views (200px)
- Support drag-and-drop upload
- Support file picker upload

**Photo Storage Structure:**
```
{APP_DATA}/
├── database.sqlite
└── photos/
    ├── families/
    │   └── {family_id}/
    │       └── family_{timestamp}.jpg
    └── members/
        └── {member_id}/
            ├── original_{timestamp}.jpg
            └── thumb_{timestamp}.jpg
```

### 4. PDF Directory Generation

**Requirements:**
- Generate printable photo directory PDF
- Organize by family
- Include configurable content:
  - Family photo (if available) or member photos
  - Family name (Mailing Name)
  - Address (primary or alternate)
  - Phone numbers
  - Member names and details
- Multiple layout options:
  - Grid layout (4-6 families per page)
  - List layout (detailed, 2-3 families per page)
- Page size options: Letter, A4
- Cover page with church name/logo
- Table of contents (alphabetical)
- Page numbers

**PDF Structure:**
1. Cover page (customizable title, optional logo)
2. Table of contents
3. Directory pages (sorted alphabetically by family name)
4. Optional: Member index

### 5. Settings & Configuration

**Application Settings:**
- Church name (for PDF cover)
- Church logo (for PDF cover)
- Default PDF layout preference
- Photo storage location
- Backup location

### 6. Backup & Restore

**Backup Requirements:**
- Create a single ZIP archive containing:
  - SQLite database file (`database.sqlite`)
  - All photos (`photos/` directory)
  - Application settings (`settings.json`)
- Filename format: `church-directory-backup-YYYY-MM-DD-HHmmss.zip`
- User selects destination folder via file picker
- Show progress indicator during backup
- Display backup summary (file count, total size)

**Backup Process:**
1. User clicks "Create Backup" in Settings or toolbar
2. File picker opens to select destination folder
3. Application creates ZIP archive with timestamp
4. Progress bar shows compression progress
5. Success message with backup location and size

**Restore Requirements:**
- Select a backup ZIP file via file picker
- Validate backup contents before restore
- Preview what will be restored (record counts, photo counts)
- Confirm before overwriting existing data
- Option to merge or replace existing data
- Restore database, photos, and settings

**Restore Process:**
1. User clicks "Restore from Backup" in Settings
2. File picker opens to select `.zip` backup file
3. Application validates ZIP structure
4. Preview dialog shows:
   - Number of families/members in backup
   - Number of photos
   - Backup date/time
   - Warning about data replacement
5. User confirms restore
6. Application extracts and restores data
7. Application restarts/reloads with restored data

**Backup Archive Structure:**
```
church-directory-backup-2024-01-15-143022.zip
├── database.sqlite
├── settings.json
├── manifest.json              # Backup metadata
└── photos/
    ├── families/
    │   └── ...
    └── members/
        └── ...
```

**Manifest File (manifest.json):**
```json
{
  "version": "1.0",
  "created_at": "2024-01-15T14:30:22Z",
  "app_version": "1.0.0",
  "family_count": 45,
  "member_count": 128,
  "photo_count": 89
}
```

## User Interface

### Main Navigation

```
┌─────────────────────────────────────────────────────────┐
│  Church Photo Directory                    [Settings]   │
├─────────────────────────────────────────────────────────┤
│  [Families] [Import] [Generate PDF] [Backup]            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────┐  ┌───────────────────────────────┐ │
│  │ Family List     │  │ Family Detail                 │ │
│  │                 │  │                               │ │
│  │ [Search...]     │  │  [Photo]     Smith Family     │ │
│  │                 │  │             123 Main St       │ │
│  │ > Smith Family  │  │             Springfield, IL   │ │
│  │   Johnson Family│  │                               │ │
│  │   Williams Fam. │  │  Members:                     │ │
│  │   ...           │  │  - John Smith (photo)         │ │
│  │                 │  │  - Jane Smith (photo)         │ │
│  │                 │  │  - Billy Smith (child)        │ │
│  │                 │  │                               │ │
│  └─────────────────┘  └───────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Screens

1. **Dashboard/Family List** - Main view with searchable family list
2. **Family Detail/Edit** - View and edit family information and members
3. **Member Detail/Edit** - View and edit individual member information
4. **Add Family** - Create new family with optional members
5. **Add Member** - Add member to existing family
6. **Import Wizard** - Multi-step import process
7. **PDF Generator** - Configure and generate directory PDF
8. **Backup/Restore** - Create backups and restore from archives
9. **Settings** - Application configuration

## Build System

### Development

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev
```

### Production Builds

**Target Platforms:**
- Windows: `.msi` installer, `.exe` portable
- macOS: `.dmg` installer, `.app` bundle
- Linux: `.deb`, `.AppImage`, `.rpm`

**Build Commands:**
```bash
# Build for current platform
npm run tauri build

# Build for specific platform (CI/CD)
npm run tauri build -- --target <target>
```

### CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/release.yml
- Build on push to main/release tags
- Matrix build: windows-latest, macos-latest, ubuntu-latest
- Code signing (optional)
- Auto-publish releases with artifacts
```

## Project Structure

```
dccdirectory/
├── src/                    # Vue frontend
│   ├── assets/
│   ├── components/
│   │   ├── families/
│   │   │   ├── FamilyList.vue
│   │   │   ├── FamilyDetail.vue
│   │   │   ├── FamilyForm.vue
│   │   │   └── FamilyCard.vue
│   │   ├── members/
│   │   │   ├── MemberList.vue
│   │   │   ├── MemberDetail.vue
│   │   │   ├── MemberForm.vue
│   │   │   └── MemberCard.vue
│   │   ├── common/
│   │   │   ├── PhotoUpload.vue
│   │   │   ├── ConfirmDialog.vue
│   │   │   ├── SearchBar.vue
│   │   │   └── DataTable.vue
│   │   ├── ImportWizard.vue
│   │   ├── PdfGenerator.vue
│   │   └── BackupRestore.vue
│   ├── views/
│   │   ├── HomeView.vue
│   │   ├── FamilyEditView.vue
│   │   ├── MemberEditView.vue
│   │   ├── ImportView.vue
│   │   ├── GenerateView.vue
│   │   ├── BackupView.vue
│   │   └── SettingsView.vue
│   ├── stores/             # Pinia stores
│   │   ├── families.ts
│   │   └── settings.ts
│   ├── lib/
│   │   └── tauri.ts        # Tauri API wrappers
│   ├── App.vue
│   ├── main.ts
│   └── style.css
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── db.rs           # SQLite operations
│   │   ├── import.rs       # XLSX import logic
│   │   ├── photos.rs       # Photo management
│   │   ├── pdf.rs          # PDF generation
│   │   └── backup.rs       # Backup and restore
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/
├── index.html
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── spec.md
```

## Rust Dependencies (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
calamine = "0.24"           # XLSX parsing
image = "0.25"              # Image processing
printpdf = "0.7"            # PDF generation
chrono = "0.4"              # Date handling
uuid = { version = "1", features = ["v4"] }
zip = "0.6"                 # Backup archive creation/extraction
walkdir = "2"               # Directory traversal for backup
```

## JavaScript Dependencies (package.json)

```json
{
  "dependencies": {
    "vue": "^3.4",
    "vue-router": "^4.3",
    "pinia": "^2.1",
    "@tauri-apps/api": "^2"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "vite": "^5",
    "@vitejs/plugin-vue": "^5",
    "tailwindcss": "^4.1",
    "typescript": "^5"
  }
}
```

## Implementation Phases

### Phase 1: Project Setup
- [ ] Initialize Tauri + Vue + Vite project
- [ ] Configure Tailwind CSS 4.1
- [ ] Set up SQLite database with schema
- [ ] Create basic navigation structure

### Phase 2: Core Data Management
- [ ] Implement family CRUD operations (create, read, update, delete)
- [ ] Implement member CRUD operations (create, read, update, delete)
- [ ] Build family list view with search/filter/sort
- [ ] Build family detail/edit view with form validation
- [ ] Build member detail/edit view with form validation
- [ ] Add new family workflow
- [ ] Add new member workflow
- [ ] Implement bulk actions (multi-select delete, toggle inclusion)
- [ ] Confirmation dialogs for destructive actions

### Phase 3: Import Feature
- [ ] XLSX parsing with calamine
- [ ] Import preview UI
- [ ] Duplicate detection logic
- [ ] Import execution with progress

### Phase 4: Photo Management
- [ ] Photo upload component
- [ ] Image resizing/compression
- [ ] Photo storage system
- [ ] Thumbnail generation

### Phase 5: PDF Generation
- [ ] PDF layout templates
- [ ] Directory content assembly
- [ ] Cover page generation
- [ ] Export functionality

### Phase 6: Backup & Restore
- [ ] Backup creation (ZIP archive)
- [ ] Manifest file generation
- [ ] Backup validation
- [ ] Restore functionality
- [ ] Merge vs replace options

### Phase 7: Polish & Build
- [ ] Settings management
- [ ] Error handling & validation
- [ ] Build configuration
- [ ] Cross-platform testing
- [ ] Installer creation

## Testing Strategy

- **Unit Tests:** Rust backend logic (db, import, pdf)
- **Component Tests:** Vue components with Vitest
- **E2E Tests:** Tauri integration with test harness
- **Manual Testing:** Cross-platform verification

## Security Considerations

- SQLite database stored in app data directory
- No network requests (fully offline capable)
- File system access restricted to app directories
- Input validation on all imported data
