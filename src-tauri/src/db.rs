use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, path::Path};
use tauri::AppHandle;
use tauri::Manager;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");

        std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

        let db_path = app_data_dir.join("directory.db");
        let conn = Connection::open(&db_path)?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        db.run_migrations()?;

        // Migrate booklet images from photos/ root to photos/directory/
        let photos_dir = app_data_dir.join("photos");
        Self::migrate_directory_images(&db, &photos_dir);

        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS families (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                family_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                mailing_name TEXT,
                address TEXT,
                city TEXT,
                state TEXT,
                zip TEXT,
                phone TEXT,
                email TEXT,
                photo_path TEXT,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                family_id INTEGER NOT NULL,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                role TEXT,
                birth_date TEXT,
                wedding_date TEXT,
                phone TEXT,
                email TEXT,
                photo_path TEXT,
                notes TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS import_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                imported_at TEXT NOT NULL DEFAULT (datetime('now')),
                families_created INTEGER NOT NULL DEFAULT 0,
                families_updated INTEGER NOT NULL DEFAULT 0,
                members_created INTEGER NOT NULL DEFAULT 0,
                members_updated INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS leadership (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ministry TEXT NOT NULL,
                names TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS staff (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                role TEXT NOT NULL DEFAULT 'staff',
                photo_path TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_families_family_id ON families(family_id);
            CREATE INDEX IF NOT EXISTS idx_families_name ON families(name);
            CREATE INDEX IF NOT EXISTS idx_members_family_id ON members(family_id);
            CREATE INDEX IF NOT EXISTS idx_members_last_name ON members(last_name);
            ",
        )?;

        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Check if mailing_name column exists in families table
        let has_mailing_name: bool = conn
            .prepare("SELECT mailing_name FROM families LIMIT 1")
            .is_ok();

        if !has_mailing_name {
            conn.execute("ALTER TABLE families ADD COLUMN mailing_name TEXT", [])?;
        }

        // Check if wedding_date column exists in members table
        let has_wedding_date: bool = conn
            .prepare("SELECT wedding_date FROM members LIMIT 1")
            .is_ok();

        if !has_wedding_date {
            conn.execute("ALTER TABLE members ADD COLUMN wedding_date TEXT", [])?;
        }

        // Add children column to families table
        let has_children: bool = conn
            .prepare("SELECT children FROM families LIMIT 1")
            .is_ok();

        if !has_children {
            conn.execute("ALTER TABLE families ADD COLUMN children TEXT", [])?;
        }

        // Add alt address columns to families table
        let has_alt_address: bool = conn
            .prepare("SELECT alt_address FROM families LIMIT 1")
            .is_ok();

        if !has_alt_address {
            conn.execute("ALTER TABLE families ADD COLUMN alt_address TEXT", [])?;
            conn.execute("ALTER TABLE families ADD COLUMN alt_city TEXT", [])?;
            conn.execute("ALTER TABLE families ADD COLUMN alt_state TEXT", [])?;
            conn.execute("ALTER TABLE families ADD COLUMN alt_zip TEXT", [])?;
        }

        // Add directory_adults and directory_children columns to families table
        let has_directory_adults: bool = conn
            .prepare("SELECT directory_adults FROM families LIMIT 1")
            .is_ok();

        if !has_directory_adults {
            conn.execute("ALTER TABLE families ADD COLUMN directory_adults TEXT", [])?;
            conn.execute("ALTER TABLE families ADD COLUMN directory_children TEXT", [])?;
        }

        // One-time: populate directory_children from children where still empty
        conn.execute(
            "UPDATE families SET directory_children = children WHERE (directory_children IS NULL OR directory_children = '') AND children IS NOT NULL AND children != ''",
            [],
        )?;

        // One-time: populate directory_adults from member first names, excluding children
        // v2: re-run after phone field import fix and reimport
        let migration_done: bool = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'migration_adults_from_members_v2'",
                [],
                |row| row.get::<_, String>(0),
            )
            .map(|v| v == "done")
            .unwrap_or(false);

        if !migration_done {
            // Get all families with their children field
            let mut family_stmt = conn.prepare(
                "SELECT id, children FROM families"
            )?;
            let families: Vec<(i64, Option<String>)> = family_stmt
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            for (family_id, children_str) in &families {
                // Parse children names into a set (lowercase for comparison)
                let child_names: Vec<String> = children_str
                    .as_deref()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Get member first names in sort order
                let mut member_stmt = conn.prepare(
                    "SELECT first_name FROM members WHERE family_id = ? ORDER BY sort_order, last_name, first_name"
                )?;
                let member_names: Vec<String> = member_stmt
                    .query_map(rusqlite::params![family_id], |row| {
                        row.get::<_, String>(0)
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                // Filter out members whose first name matches a child name
                let adult_names: Vec<&String> = member_names
                    .iter()
                    .filter(|name| !child_names.contains(&name.to_lowercase()))
                    .collect();

                if !adult_names.is_empty() {
                    let adults_str = adult_names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                    conn.execute(
                        "UPDATE families SET directory_adults = ? WHERE id = ?",
                        rusqlite::params![adults_str, family_id],
                    )?;
                }
            }

            // Mark migration as done
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('migration_adults_from_members_v2', 'done')",
                [],
            )?;
        }

        // Add sort_order column to families table
        let has_sort_order: bool = conn
            .prepare("SELECT sort_order FROM families LIMIT 1")
            .is_ok();

        if !has_sort_order {
            conn.execute(
                "ALTER TABLE families ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }

        // Add include_photo_in_directory column to families table
        let has_include_photo: bool = conn
            .prepare("SELECT include_photo_in_directory FROM families LIMIT 1")
            .is_ok();

        if !has_include_photo {
            conn.execute(
                "ALTER TABLE families ADD COLUMN include_photo_in_directory INTEGER NOT NULL DEFAULT 1",
                [],
            )?;
        }

        // Add title column to staff table
        let has_staff_title: bool = conn
            .prepare("SELECT title FROM staff LIMIT 1")
            .is_ok();

        if !has_staff_title {
            conn.execute(
                "ALTER TABLE staff ADD COLUMN title TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        Ok(())
    }

    /// Move any booklet images that were saved to photos/ root into photos/directory/.
    /// The settings DB stores just the filename, so only the files need to move.
    fn migrate_directory_images(db: &Database, photos_dir: &Path) {
        let directory_dir = photos_dir.join("directory");

        let conn = match db.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        let image_keys = [
            "cover_image_path",
            "back_cover_image_path",
            "celebration_image_path",
        ];

        for key in &image_keys {
            let filename: Option<String> = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    rusqlite::params![key],
                    |row| row.get(0),
                )
                .ok()
                .filter(|s: &String| !s.is_empty());

            if let Some(ref fname) = filename {
                let old_path = photos_dir.join(fname);
                let new_path = directory_dir.join(fname);

                // Only migrate if file exists at old location but not at new location
                if old_path.exists() && !new_path.exists() {
                    let _ = fs::create_dir_all(&directory_dir);
                    let _ = fs::rename(&old_path, &new_path);

                    // Also move _full and _thumb variants
                    let stem = old_path.file_stem().unwrap_or_default().to_string_lossy();
                    for suffix in &["_full.jpg", "_thumb.jpg"] {
                        let old_variant = photos_dir.join(format!("{}{}", stem, suffix));
                        let new_variant = directory_dir.join(format!("{}{}", stem, suffix));
                        if old_variant.exists() && !new_variant.exists() {
                            let _ = fs::rename(&old_variant, &new_variant);
                        }
                    }
                }
            }
        }
    }

    pub fn get_photos_dir(app_handle: &AppHandle) -> PathBuf {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");
        let photos_dir = app_data_dir.join("photos");
        std::fs::create_dir_all(&photos_dir).expect("Failed to create photos directory");
        photos_dir
    }
}
