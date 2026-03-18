use crate::db::Database;
use crate::models::Settings;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<Option<Settings>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut settings = Settings::default();
    let mut found_any = false;

    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        found_any = true;
        match key.as_str() {
            "church_name" => settings.church_name = value,
            "church_logo_path" => settings.church_logo_path = Some(value).filter(|s| !s.is_empty()),
            "default_layout" => settings.default_layout = value,
            "page_size" => settings.page_size = value,
            "include_photos" => settings.include_photos = value == "true",
            "include_contact_info" => settings.include_contact_info = value == "true",
            "include_address" => settings.include_address = value == "true",
            "cover_image_path" => settings.cover_image_path = Some(value).filter(|s| !s.is_empty()),
            "cover_title_line1" => settings.cover_title_line1 = Some(value).filter(|s| !s.is_empty()),
            "cover_title_line2" => settings.cover_title_line2 = Some(value).filter(|s| !s.is_empty()),
            "cover_title_color" => settings.cover_title_color = Some(value).filter(|s| !s.is_empty()),
            "pastor_letter" => settings.pastor_letter = Some(value).filter(|s| !s.is_empty()),
            "mission_statement" => settings.mission_statement = Some(value).filter(|s| !s.is_empty()),
            "first_page_markdown" => settings.first_page_markdown = Some(value).filter(|s| !s.is_empty()),
            "back_cover_image_path" => settings.back_cover_image_path = Some(value).filter(|s| !s.is_empty()),
            "celebration_image_path" => settings.celebration_image_path = Some(value).filter(|s| !s.is_empty()),
            "church_address" => settings.church_address = Some(value).filter(|s| !s.is_empty()),
            "church_phone" => settings.church_phone = Some(value).filter(|s| !s.is_empty()),
            "church_email" => settings.church_email = Some(value).filter(|s| !s.is_empty()),
            "church_website" => settings.church_website = Some(value).filter(|s| !s.is_empty()),
            _ => {}
        }
    }

    if found_any {
        Ok(Some(settings))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn save_settings(db: State<'_, Database>, settings: Settings) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let pairs = vec![
        ("church_name", settings.church_name),
        (
            "church_logo_path",
            settings.church_logo_path.unwrap_or_default(),
        ),
        ("default_layout", settings.default_layout),
        ("page_size", settings.page_size),
        ("include_photos", settings.include_photos.to_string()),
        (
            "include_contact_info",
            settings.include_contact_info.to_string(),
        ),
        ("include_address", settings.include_address.to_string()),
        (
            "cover_image_path",
            settings.cover_image_path.unwrap_or_default(),
        ),
        (
            "cover_title_line1",
            settings.cover_title_line1.unwrap_or_default(),
        ),
        (
            "cover_title_line2",
            settings.cover_title_line2.unwrap_or_default(),
        ),
        (
            "cover_title_color",
            settings.cover_title_color.unwrap_or_default(),
        ),
        (
            "pastor_letter",
            settings.pastor_letter.unwrap_or_default(),
        ),
        (
            "mission_statement",
            settings.mission_statement.unwrap_or_default(),
        ),
        (
            "first_page_markdown",
            settings.first_page_markdown.unwrap_or_default(),
        ),
        (
            "back_cover_image_path",
            settings.back_cover_image_path.unwrap_or_default(),
        ),
        (
            "celebration_image_path",
            settings.celebration_image_path.unwrap_or_default(),
        ),
        (
            "church_address",
            settings.church_address.unwrap_or_default(),
        ),
        (
            "church_phone",
            settings.church_phone.unwrap_or_default(),
        ),
        (
            "church_email",
            settings.church_email.unwrap_or_default(),
        ),
        (
            "church_website",
            settings.church_website.unwrap_or_default(),
        ),
    ];

    for (key, value) in pairs {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
