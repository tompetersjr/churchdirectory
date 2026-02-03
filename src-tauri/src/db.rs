use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
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

        Ok(())
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
