use crate::db::Database;
use crate::models::{Family, FamilyInput, FamilyUpdate, FamilyWithMembers, Member};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_families(db: State<'_, Database>) -> Result<Vec<Family>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, family_id, name, mailing_name, address, city, state, zip, phone,
                    photo_path, notes, children, alt_address, alt_city, alt_state, alt_zip,
                    directory_adults, directory_children, include_photo_in_directory, created_at, updated_at
             FROM families ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let families = stmt
        .query_map([], |row| {
            Ok(Family {
                id: row.get(0)?,
                family_id: row.get(1)?,
                name: row.get(2)?,
                mailing_name: row.get(3)?,
                address: row.get(4)?,
                city: row.get(5)?,
                state: row.get(6)?,
                zip: row.get(7)?,
                phone: row.get(8)?,
                photo_path: row.get(9)?,
                notes: row.get(10)?,
                children: row.get(11)?,
                alt_address: row.get(12)?,
                alt_city: row.get(13)?,
                alt_state: row.get(14)?,
                alt_zip: row.get(15)?,
                directory_adults: row.get(16)?,
                directory_children: row.get(17)?,
                include_photo_in_directory: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(families)
}

#[tauri::command]
pub fn get_family(db: State<'_, Database>, id: i64) -> Result<FamilyWithMembers, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let family = conn
        .query_row(
            "SELECT id, family_id, name, mailing_name, address, city, state, zip, phone,
                    photo_path, notes, children, alt_address, alt_city, alt_state, alt_zip,
                    directory_adults, directory_children, include_photo_in_directory, created_at, updated_at
             FROM families WHERE id = ?",
            params![id],
            |row| {
                Ok(Family {
                    id: row.get(0)?,
                    family_id: row.get(1)?,
                    name: row.get(2)?,
                    mailing_name: row.get(3)?,
                    address: row.get(4)?,
                    city: row.get(5)?,
                    state: row.get(6)?,
                    zip: row.get(7)?,
                    phone: row.get(8)?,
                    photo_path: row.get(9)?,
                    notes: row.get(10)?,
                    children: row.get(11)?,
                    alt_address: row.get(12)?,
                    alt_city: row.get(13)?,
                    alt_state: row.get(14)?,
                    alt_zip: row.get(15)?,
                    directory_adults: row.get(16)?,
                    directory_children: row.get(17)?,
                    include_photo_in_directory: row.get(18)?,
                    created_at: row.get(19)?,
                    updated_at: row.get(20)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, family_id, first_name, last_name, role, birth_date, wedding_date, phone, email,
                    photo_path, notes, sort_order, created_at, updated_at
             FROM members WHERE family_id = ? ORDER BY sort_order, last_name, first_name",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map(params![id], |row| {
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

    Ok(FamilyWithMembers { family, members })
}

#[tauri::command]
pub fn create_family(db: State<'_, Database>, family: FamilyInput) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO families (family_id, name, mailing_name, address, city, state, zip, phone, photo_path, notes, children, alt_address, alt_city, alt_state, alt_zip, directory_adults, directory_children, include_photo_in_directory)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            family.family_id,
            family.name,
            family.mailing_name,
            family.address,
            family.city,
            family.state,
            family.zip,
            family.phone,
            family.photo_path,
            family.notes,
            family.children,
            family.alt_address,
            family.alt_city,
            family.alt_state,
            family.alt_zip,
            family.directory_adults,
            family.directory_children,
            family.include_photo_in_directory,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_family(db: State<'_, Database>, id: i64, family: FamilyUpdate) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut updates = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref v) = family.family_id {
        updates.push("family_id = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.name {
        updates.push("name = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.mailing_name {
        updates.push("mailing_name = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.address {
        updates.push("address = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.city {
        updates.push("city = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.state {
        updates.push("state = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.zip {
        updates.push("zip = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.phone {
        updates.push("phone = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.photo_path {
        updates.push("photo_path = ?");
        values.push(Box::new(v.clone()));
    }

    if let Some(ref v) = family.notes {
        updates.push("notes = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.children {
        updates.push("children = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.alt_address {
        updates.push("alt_address = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.alt_city {
        updates.push("alt_city = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.alt_state {
        updates.push("alt_state = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.alt_zip {
        updates.push("alt_zip = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.directory_adults {
        updates.push("directory_adults = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = family.directory_children {
        updates.push("directory_children = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = family.include_photo_in_directory {
        updates.push("include_photo_in_directory = ?");
        values.push(Box::new(v));
    }
    if updates.is_empty() {
        return Ok(());
    }

    updates.push("updated_at = datetime('now')");
    values.push(Box::new(id));

    let sql = format!(
        "UPDATE families SET {} WHERE id = ?",
        updates.join(", ")
    );

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_family(db: State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM members WHERE family_id = ?", params![id])
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM families WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
