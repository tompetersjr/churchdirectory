use crate::db::Database;
use crate::models::{Member, MemberInput, MemberUpdate};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_members(db: State<'_, Database>, family_id: i64) -> Result<Vec<Member>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, family_id, first_name, last_name, role, birth_date, wedding_date, phone, email,
                    photo_path, notes, sort_order, created_at, updated_at
             FROM members WHERE family_id = ? ORDER BY sort_order, last_name, first_name",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map(params![family_id], |row| {
            Ok(Member {
                id: row.get(0)?,
                family_id: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                role: row.get(4)?,
                birth_date: row.get(5)?,
                wedding_date: row.get(6)?,
                phone: row.get(7)?,
                email: row.get(8)?,
                photo_path: row.get(9)?,
                notes: row.get(10)?,
                sort_order: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

#[tauri::command]
pub fn get_member(db: State<'_, Database>, id: i64) -> Result<Member, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, family_id, first_name, last_name, role, birth_date, wedding_date, phone, email,
                photo_path, notes, sort_order, created_at, updated_at
         FROM members WHERE id = ?",
        params![id],
        |row| {
            Ok(Member {
                id: row.get(0)?,
                family_id: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                role: row.get(4)?,
                birth_date: row.get(5)?,
                wedding_date: row.get(6)?,
                phone: row.get(7)?,
                email: row.get(8)?,
                photo_path: row.get(9)?,
                notes: row.get(10)?,
                sort_order: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_member(db: State<'_, Database>, member: MemberInput) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO members (family_id, first_name, last_name, role, birth_date, wedding_date, phone, email, photo_path, notes, sort_order)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            member.family_id,
            member.first_name,
            member.last_name,
            member.role,
            member.birth_date,
            member.wedding_date,
            member.phone,
            member.email,
            member.photo_path,
            member.notes,
            member.sort_order,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_member(db: State<'_, Database>, id: i64, member: MemberUpdate) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut updates = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref v) = member.first_name {
        updates.push("first_name = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.last_name {
        updates.push("last_name = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.role {
        updates.push("role = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.birth_date {
        updates.push("birth_date = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.wedding_date {
        updates.push("wedding_date = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.phone {
        updates.push("phone = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.email {
        updates.push("email = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.photo_path {
        updates.push("photo_path = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = member.notes {
        updates.push("notes = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = member.sort_order {
        updates.push("sort_order = ?");
        values.push(Box::new(v));
    }

    if updates.is_empty() {
        return Ok(());
    }

    updates.push("updated_at = datetime('now')");
    values.push(Box::new(id));

    let sql = format!(
        "UPDATE members SET {} WHERE id = ?",
        updates.join(", ")
    );

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_member(db: State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM members WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
