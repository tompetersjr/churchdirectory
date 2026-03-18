use crate::db::Database;
use crate::models::{Staff, StaffInput};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_staff(db: State<'_, Database>) -> Result<Vec<Staff>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, title, role, photo_path, sort_order, created_at, updated_at
             FROM staff ORDER BY sort_order, name",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| {
            Ok(Staff {
                id: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                role: row.get(3)?,
                photo_path: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn create_staff(
    db: State<'_, Database>,
    input: StaffInput,
) -> Result<Staff, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO staff (name, title, role, sort_order) VALUES (?, ?, ?, ?)",
        params![input.name, input.title, input.role, input.sort_order],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    query_staff_by_id(&conn, id)
}

#[tauri::command]
pub fn update_staff(
    db: State<'_, Database>,
    id: i64,
    input: StaffInput,
) -> Result<Staff, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE staff SET name = ?, title = ?, role = ?, sort_order = ?, updated_at = datetime('now') WHERE id = ?",
        params![input.name, input.title, input.role, input.sort_order, id],
    )
    .map_err(|e| e.to_string())?;

    query_staff_by_id(&conn, id)
}

#[tauri::command]
pub fn delete_staff(db: State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM staff WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_staff_photo(
    app_handle: tauri::AppHandle,
    db: State<'_, Database>,
    staff_id: i64,
    source_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let staff_dir = photos_dir.join("staff");
    std::fs::create_dir_all(&staff_dir).map_err(|e| e.to_string())?;

    let saved_path = crate::photos::process_and_save_image_public(&source_path, &staff_dir, "staff")?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE staff SET photo_path = ?, updated_at = datetime('now') WHERE id = ?",
        params![saved_path, staff_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(saved_path)
}

fn query_staff_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Staff, String> {
    conn.query_row(
        "SELECT id, name, title, role, photo_path, sort_order, created_at, updated_at
         FROM staff WHERE id = ?",
        params![id],
        |row| {
            Ok(Staff {
                id: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                role: row.get(3)?,
                photo_path: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}
