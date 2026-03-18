use crate::db::Database;
use crate::models::{Leadership, LeadershipInput};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_leadership(db: State<'_, Database>) -> Result<Vec<Leadership>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, ministry, names, sort_order, created_at, updated_at
             FROM leadership ORDER BY ministry COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| {
            Ok(Leadership {
                id: row.get(0)?,
                ministry: row.get(1)?,
                names: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn create_leadership(
    db: State<'_, Database>,
    input: LeadershipInput,
) -> Result<Leadership, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO leadership (ministry, names, sort_order) VALUES (?, ?, ?)",
        params![input.ministry, input.names, input.sort_order],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let entry = conn
        .query_row(
            "SELECT id, ministry, names, sort_order, created_at, updated_at
             FROM leadership WHERE id = ?",
            params![id],
            |row| {
                Ok(Leadership {
                    id: row.get(0)?,
                    ministry: row.get(1)?,
                    names: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(entry)
}

#[tauri::command]
pub fn update_leadership(
    db: State<'_, Database>,
    id: i64,
    input: LeadershipInput,
) -> Result<Leadership, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE leadership SET ministry = ?, names = ?, sort_order = ?, updated_at = datetime('now') WHERE id = ?",
        params![input.ministry, input.names, input.sort_order, id],
    )
    .map_err(|e| e.to_string())?;

    let entry = conn
        .query_row(
            "SELECT id, ministry, names, sort_order, created_at, updated_at
             FROM leadership WHERE id = ?",
            params![id],
            |row| {
                Ok(Leadership {
                    id: row.get(0)?,
                    ministry: row.get(1)?,
                    names: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(entry)
}

#[tauri::command]
pub fn delete_leadership(db: State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM leadership WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
