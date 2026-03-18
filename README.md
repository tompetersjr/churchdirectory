# Church Photo Directory

A desktop application for managing church member directories and generating print-ready PDF directories.

Built with [Tauri 2](https://tauri.app/), [Vue 3](https://vuejs.org/), and [Rust](https://www.rust-lang.org/).

## Features

- **Family & Member Management** - Add, edit, and organize families and their members
- **Photo Management** - Upload family photos and crop individual member avatars
- **PDF Generation** - Create print-ready directories with customizable layouts
- **Data Import** - Bulk import families from Servant Keeper exports (CSV or Excel)
- **Backup & Restore** - Full backup and restore of all data and photos
- **Dark Mode** - System, light, and dark theme support
- **Cross-Platform** - Runs on Windows, macOS, and Linux

## Installation

Download the latest release for your platform from the [Releases](https://github.com/tompetersjr/churchdirectory/releases) page:

| Platform | Download |
|----------|----------|
| Windows | `.msi` or `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Linux | `.deb` or `.AppImage` |

### macOS Note

The app is not signed with an Apple Developer certificate. On first launch:
1. Right-click the app and select "Open"
2. Click "Open" in the dialog to bypass Gatekeeper

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.77+
- Platform-specific dependencies:

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

**Windows:**
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload

### Setup

```bash
# Clone the repository
git clone https://github.com/tompetersjr/churchdirectory.git
cd churchdirectory

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

Installers will be created in `src-tauri/target/release/bundle/`.

## Project Structure

```
dccdirectory/
├── src/                    # Vue frontend
│   ├── components/         # Reusable Vue components
│   ├── views/              # Page components
│   ├── stores/             # Pinia state management
│   └── types.ts            # TypeScript types
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── db.rs           # SQLite database
│   │   ├── pdf.rs          # PDF generation
│   │   ├── photos.rs       # Photo processing
│   │   ├── backup.rs       # Backup/restore
│   │   └── import.rs       # Excel import
│   └── Cargo.toml          # Rust dependencies
└── package.json            # Node dependencies
```

## Tech Stack

**Frontend:**
- Vue 3 (Composition API)
- TypeScript
- Tailwind CSS 4
- Pinia (state management)
- Vue Router

**Backend:**
- Rust
- Tauri 2
- SQLite (rusqlite)
- printpdf (PDF generation)
- image (image processing)
- calamine (Excel parsing)
- csv (CSV parsing)

## Data Import

The import feature is designed around **Servant Keeper** exports. Export your directory data as a CSV or Excel file with the following fields:

| Field | Required | Notes |
|-------|----------|-------|
| Family ID | Yes | Used to group individuals into families |
| Last Name | Yes | Also used as the Family Name if no Family Name column is present |
| First Name | Yes | |
| Family Name | No | Defaults to Last Name if missing |
| Mailing Name | No | e.g., "Mr. & Mrs. John Smith" |
| Address | No | |
| City | No | |
| State | No | |
| Zip Code | No | |
| Phone | No | Family/home phone number |
| Cell Phone | No | Individual cell phone |
| E-Mail | No | |
| Birth Date | No | |
| Wedding Date | No | |
| Children | No | Comma-separated list of children's names. Stored on the family record and used for automatic "Child" role detection. |
| Alt Address | No | |
| Alt City | No | |
| Alt State | No | |
| Alt Zip Code | No | |

Families are built by grouping individuals that share the same **Family ID**. The **Children** field is a comma-separated list of children's names that is stored on the family record. Members whose first name appears in this list are automatically assigned the "Child" role.

## Data Storage

All data is stored locally in your system's app data directory:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/com.churchdirectory.app/` |
| Windows | `%APPDATA%\com.churchdirectory.app\` |
| Linux | `~/.local/share/com.churchdirectory.app/` |

## License

MIT
