use crate::db::Database;
use crate::models::BackupManifest;
use chrono::Utc;
use rusqlite::params;
use std::fs::{self, File};
use std::io::{Read, Write};
use tauri::{AppHandle, Manager, State};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BACKUP_VERSION: &str = "1.0";

#[tauri::command]
pub fn create_backup(
    app_handle: AppHandle,
    db: State<'_, Database>,
    output_path: String,
) -> Result<BackupManifest, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let db_path = app_data_dir.join("directory.db");
    let photos_dir = app_data_dir.join("photos");

    let family_count: i64;
    let member_count: i64;
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        family_count = conn
            .query_row("SELECT COUNT(*) FROM families", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        member_count = conn
            .query_row("SELECT COUNT(*) FROM members", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
    }

    let mut photo_count = 0;
    if photos_dir.exists() {
        for entry in WalkDir::new(&photos_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                photo_count += 1;
            }
        }
    }

    let manifest = BackupManifest {
        version: BACKUP_VERSION.to_string(),
        created_at: Utc::now().to_rfc3339(),
        app_version: APP_VERSION.to_string(),
        family_count: family_count as usize,
        member_count: member_count as usize,
        photo_count,
    };

    let file = File::create(&output_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| e.to_string())?;

    if db_path.exists() {
        let mut db_file = File::open(&db_path).map_err(|e| e.to_string())?;
        let mut db_contents = Vec::new();
        db_file
            .read_to_end(&mut db_contents)
            .map_err(|e| e.to_string())?;

        zip.start_file("directory.db", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&db_contents).map_err(|e| e.to_string())?;
    }

    if photos_dir.exists() {
        for entry in WalkDir::new(&photos_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let relative_path = path
                    .strip_prefix(&photos_dir)
                    .map_err(|e| e.to_string())?;
                let archive_path = format!("photos/{}", relative_path.to_string_lossy());

                let mut file = File::open(path).map_err(|e| e.to_string())?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).map_err(|e| e.to_string())?;

                zip.start_file(&archive_path, options)
                    .map_err(|e| e.to_string())?;
                zip.write_all(&contents).map_err(|e| e.to_string())?;
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;

    Ok(manifest)
}

#[tauri::command]
pub fn preview_restore(backup_path: String) -> Result<BackupManifest, String> {
    let file = File::open(&backup_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|_| "Invalid backup: manifest.json not found")?;

    let mut manifest_contents = String::new();
    manifest_file
        .read_to_string(&mut manifest_contents)
        .map_err(|e| e.to_string())?;

    let manifest: BackupManifest =
        serde_json::from_str(&manifest_contents).map_err(|e| e.to_string())?;

    Ok(manifest)
}

#[tauri::command]
pub fn restore_backup(
    app_handle: AppHandle,
    db: State<'_, Database>,
    backup_path: String,
    replace_existing: bool,
) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let file = File::open(&backup_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    {
        let _ = archive
            .by_name("manifest.json")
            .map_err(|_| "Invalid backup: manifest.json not found")?;
    }

    if replace_existing {
        let photos_dir = app_data_dir.join("photos");
        if photos_dir.exists() {
            fs::remove_dir_all(&photos_dir).map_err(|e| e.to_string())?;
        }

        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM members", [])
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM families", [])
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM import_history", [])
                .map_err(|e| e.to_string())?;
        }
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();

        if name == "manifest.json" {
            continue;
        }

        if name == "directory.db" {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(|e| e.to_string())?;

            let temp_db_path = app_data_dir.join("restore_temp.db");
            fs::write(&temp_db_path, &contents).map_err(|e| e.to_string())?;

            {
                let source_conn =
                    rusqlite::Connection::open(&temp_db_path).map_err(|e| e.to_string())?;
                let conn = db.conn.lock().map_err(|e| e.to_string())?;

                let mut stmt = source_conn
                    .prepare(
                        "SELECT family_id, name, address, city, state, zip, phone, email, photo_path, notes FROM families",
                    )
                    .map_err(|e| e.to_string())?;

                let families = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, Option<String>>(3)?,
                            row.get::<_, Option<String>>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, Option<String>>(6)?,
                            row.get::<_, Option<String>>(7)?,
                            row.get::<_, Option<String>>(8)?,
                            row.get::<_, Option<String>>(9)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?;

                for family_result in families {
                    let (family_id, name, address, city, state, zip, phone, email, photo_path, notes) =
                        family_result.map_err(|e| e.to_string())?;

                    let existing: Option<i64> = conn
                        .query_row(
                            "SELECT id FROM families WHERE family_id = ?",
                            params![&family_id],
                            |row| row.get(0),
                        )
                        .ok();

                    let db_family_id = if let Some(id) = existing {
                        if replace_existing {
                            conn.execute(
                                "UPDATE families SET name = ?, address = ?, city = ?, state = ?, zip = ?, phone = ?, email = ?, photo_path = ?, notes = ?, updated_at = datetime('now') WHERE id = ?",
                                params![name, address, city, state, zip, phone, email, photo_path, notes, id],
                            ).map_err(|e| e.to_string())?;
                        }
                        id
                    } else {
                        conn.execute(
                            "INSERT INTO families (family_id, name, address, city, state, zip, phone, email, photo_path, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                            params![family_id, name, address, city, state, zip, phone, email, photo_path, notes],
                        ).map_err(|e| e.to_string())?;
                        conn.last_insert_rowid()
                    };

                    let source_family_id: i64 = source_conn
                        .query_row(
                            "SELECT id FROM families WHERE family_id = ?",
                            params![&family_id],
                            |row| row.get(0),
                        )
                        .map_err(|e| e.to_string())?;

                    let mut member_stmt = source_conn
                        .prepare(
                            "SELECT first_name, last_name, role, birth_date, phone, email, photo_path, notes, sort_order FROM members WHERE family_id = ?",
                        )
                        .map_err(|e| e.to_string())?;

                    let members = member_stmt
                        .query_map(params![source_family_id], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, Option<String>>(2)?,
                                row.get::<_, Option<String>>(3)?,
                                row.get::<_, Option<String>>(4)?,
                                row.get::<_, Option<String>>(5)?,
                                row.get::<_, Option<String>>(6)?,
                                row.get::<_, Option<String>>(7)?,
                                row.get::<_, i32>(8)?,
                            ))
                        })
                        .map_err(|e| e.to_string())?;

                    for member_result in members {
                        let (first_name, last_name, role, birth_date, phone, email, photo_path, notes, sort_order) =
                            member_result.map_err(|e| e.to_string())?;

                        let existing_member: Option<i64> = conn
                            .query_row(
                                "SELECT id FROM members WHERE family_id = ? AND first_name = ? AND last_name = ?",
                                params![db_family_id, &first_name, &last_name],
                                |row| row.get(0),
                            )
                            .ok();

                        if let Some(member_id) = existing_member {
                            if replace_existing {
                                conn.execute(
                                    "UPDATE members SET role = ?, birth_date = ?, phone = ?, email = ?, photo_path = ?, notes = ?, sort_order = ?, updated_at = datetime('now') WHERE id = ?",
                                    params![role, birth_date, phone, email, photo_path, notes, sort_order, member_id],
                                ).map_err(|e| e.to_string())?;
                            }
                        } else {
                            conn.execute(
                                "INSERT INTO members (family_id, first_name, last_name, role, birth_date, phone, email, photo_path, notes, sort_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                params![db_family_id, first_name, last_name, role, birth_date, phone, email, photo_path, notes, sort_order],
                            ).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }

            fs::remove_file(&temp_db_path).ok();
        } else if name.starts_with("photos/") {
            let relative_path = name.strip_prefix("photos/").unwrap();
            let dest_path = app_data_dir.join("photos").join(relative_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }

            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
            fs::write(&dest_path, &contents).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
