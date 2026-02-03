use crate::db::Database;
use crate::models::{DuplicateMatch, ImportFamilyPreview, ImportMemberPreview, ImportPreview, ImportResult};
use calamine::{open_workbook, Reader, Xlsx};
use chrono::NaiveDate;
use rusqlite::params;
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

#[derive(Debug)]
struct ImportRow {
    family_id: String,
    family_name: String,
    mailing_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    member_first_name: String,
    member_last_name: String,
    member_role: Option<String>,
    member_phone: Option<String>,
    member_email: Option<String>,
    member_birth_date: Option<String>,
    member_wedding_date: Option<String>,
}

/// Normalize a date string to a consistent format.
/// Handles dates with or without years (e.g., "1/15" becomes "01-15", "1/15/1990" becomes "1990-01-15")
/// Also handles formats like "07/20/    " where year is blank/spaces
fn normalize_date(date_str: &str) -> Option<String> {
    let date_str = date_str.trim();
    if date_str.is_empty() {
        return None;
    }

    // Try to parse various date formats
    // Handle Excel serial dates (numbers)
    if let Ok(serial) = date_str.parse::<f64>() {
        // Excel serial date: days since 1899-12-30
        let days = serial as i64;
        if days > 0 && days < 100000 {
            // Convert Excel serial to date
            let base = NaiveDate::from_ymd_opt(1899, 12, 30)?;
            let date = base.checked_add_signed(chrono::TimeDelta::days(days))?;
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    // Split by common delimiters and filter out empty/whitespace parts
    let parts: Vec<&str> = date_str
        .split(|c| c == '/' || c == '-' || c == '.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match parts.len() {
        2 => {
            // Month/Day only (no year)
            let month: u32 = parts[0].parse().ok()?;
            let day: u32 = parts[1].parse().ok()?;
            if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                Some(format!("{:02}-{:02}", month, day))
            } else {
                None
            }
        }
        3 => {
            // Full date with year
            let p0: u32 = parts[0].parse().ok()?;
            let p1: u32 = parts[1].parse().ok()?;
            let p2: u32 = parts[2].parse().ok()?;

            // Determine format: MM/DD/YYYY, DD/MM/YYYY, or YYYY-MM-DD
            if p0 > 31 {
                // YYYY-MM-DD format
                let year = p0;
                let month = p1;
                let day = p2;
                if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                    Some(format!("{:04}-{:02}-{:02}", year, month, day))
                } else {
                    None
                }
            } else if p2 > 31 {
                // MM/DD/YYYY or DD/MM/YYYY format - assume MM/DD/YYYY (US format)
                let month = p0;
                let day = p1;
                let year = if p2 < 100 {
                    if p2 > 50 { 1900 + p2 } else { 2000 + p2 }
                } else {
                    p2
                };
                if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                    Some(format!("{:04}-{:02}-{:02}", year, month, day))
                } else {
                    None
                }
            } else {
                // All parts <= 31, assume MM/DD/YY with 2-digit year
                let month = p0;
                let day = p1;
                let year = if p2 < 100 {
                    if p2 > 50 { 1900 + p2 } else { 2000 + p2 }
                } else {
                    p2
                };
                if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                    Some(format!("{:04}-{:02}-{:02}", year, month, day))
                } else {
                    None
                }
            }
        }
        _ => {
            // Try to preserve as-is if it looks like a valid date string
            Some(date_str.to_string())
        }
    }
}

fn parse_xlsx(file_path: &str) -> Result<Vec<ImportRow>, String> {
    let path = Path::new(file_path);
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("No sheets found in workbook")?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Failed to read sheet: {}", e))?;

    let mut rows = Vec::new();
    let mut headers: HashMap<String, usize> = HashMap::new();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            for (col_idx, cell) in row.iter().enumerate() {
                let header = cell.to_string().to_lowercase().trim().to_string();
                headers.insert(header, col_idx);
            }
            continue;
        }

        let get_cell = |name: &str| -> Option<String> {
            headers.get(name).and_then(|&idx| {
                row.get(idx).map(|c| {
                    let s = c.to_string().trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }).flatten()
            })
        };

        let family_id = get_cell("family id")
            .or_else(|| get_cell("familyid"))
            .or_else(|| get_cell("family_id"));

        let family_name = get_cell("family name")
            .or_else(|| get_cell("familyname"))
            .or_else(|| get_cell("family_name"))
            .or_else(|| get_cell("last name"))
            .or_else(|| get_cell("lastname"));

        let first_name = get_cell("first name")
            .or_else(|| get_cell("firstname"))
            .or_else(|| get_cell("first_name"));

        let last_name = get_cell("last name")
            .or_else(|| get_cell("lastname"))
            .or_else(|| get_cell("last_name"));

        if let (Some(fid), Some(fname), Some(mfirst), Some(mlast)) =
            (family_id, family_name, first_name, last_name)
        {
            // Try various email column names
            let email = get_cell("email")
                .or_else(|| get_cell("e-mail"))
                .or_else(|| get_cell("email address"))
                .or_else(|| get_cell("emailaddress"));

            // Try various phone column names
            let phone = get_cell("phone")
                .or_else(|| get_cell("telephone"))
                .or_else(|| get_cell("phone number"))
                .or_else(|| get_cell("phonenumber"))
                .or_else(|| get_cell("home phone"))
                .or_else(|| get_cell("cell phone"))
                .or_else(|| get_cell("mobile"));

            // Try various mailing name column names
            let mailing_name = get_cell("mailing name")
                .or_else(|| get_cell("mailingname"))
                .or_else(|| get_cell("mailing_name"))
                .or_else(|| get_cell("envelope name"))
                .or_else(|| get_cell("mail name"));

            // Try various birth date column names
            let birth_date_raw = get_cell("birth date")
                .or_else(|| get_cell("birthdate"))
                .or_else(|| get_cell("birth_date"))
                .or_else(|| get_cell("birthday"))
                .or_else(|| get_cell("dob"))
                .or_else(|| get_cell("date of birth"));
            let birth_date = birth_date_raw.and_then(|d| normalize_date(&d));

            // Try various wedding date column names
            let wedding_date_raw = get_cell("wedding date")
                .or_else(|| get_cell("weddingdate"))
                .or_else(|| get_cell("wedding_date"))
                .or_else(|| get_cell("anniversary"))
                .or_else(|| get_cell("marriage date"))
                .or_else(|| get_cell("married"));
            let wedding_date = wedding_date_raw.and_then(|d| normalize_date(&d));

            rows.push(ImportRow {
                family_id: fid,
                family_name: fname,
                mailing_name,
                address: get_cell("address").or_else(|| get_cell("street address")).or_else(|| get_cell("street")),
                city: get_cell("city"),
                state: get_cell("state"),
                zip: get_cell("zip").or_else(|| get_cell("zipcode")).or_else(|| get_cell("zip code")).or_else(|| get_cell("postal code")),
                phone: phone.clone(),
                email: email.clone(),
                member_first_name: mfirst,
                member_last_name: mlast,
                member_role: get_cell("role").or_else(|| get_cell("relationship")).or_else(|| get_cell("member type")),
                member_phone: phone,
                member_email: email,
                member_birth_date: birth_date,
                member_wedding_date: wedding_date,
            });
        }
    }

    Ok(rows)
}

#[tauri::command]
pub fn preview_import(db: State<'_, Database>, file_path: String) -> Result<ImportPreview, String> {
    let rows = parse_xlsx(&file_path)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut families_map: HashMap<String, ImportFamilyPreview> = HashMap::new();
    let mut duplicates = Vec::new();

    for row in &rows {
        let entry = families_map.entry(row.family_id.clone()).or_insert_with(|| {
            ImportFamilyPreview {
                family_id: row.family_id.clone(),
                name: row.family_name.clone(),
                address: row.address.clone(),
                members: Vec::new(),
                is_duplicate: false,
                existing_family_id: None,
            }
        });

        entry.members.push(ImportMemberPreview {
            first_name: row.member_first_name.clone(),
            last_name: row.member_last_name.clone(),
            role: row.member_role.clone(),
        });
    }

    for family in families_map.values_mut() {
        let existing_by_id: Option<(i64, String)> = conn
            .query_row(
                "SELECT id, name FROM families WHERE family_id = ?",
                params![family.family_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((existing_id, existing_name)) = existing_by_id {
            family.is_duplicate = true;
            family.existing_family_id = Some(existing_id);
            duplicates.push(DuplicateMatch {
                import_family_id: family.family_id.clone(),
                import_name: family.name.clone(),
                existing_id,
                existing_name,
                match_type: "id".to_string(),
            });
        } else {
            let existing_by_name: Option<(i64, String)> = conn
                .query_row(
                    "SELECT id, family_id FROM families WHERE LOWER(name) = LOWER(?)",
                    params![family.name],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            if let Some((existing_id, existing_family_id)) = existing_by_name {
                family.is_duplicate = true;
                family.existing_family_id = Some(existing_id);
                duplicates.push(DuplicateMatch {
                    import_family_id: family.family_id.clone(),
                    import_name: family.name.clone(),
                    existing_id,
                    existing_name: existing_family_id,
                    match_type: "name".to_string(),
                });
            }
        }
    }

    let families: Vec<ImportFamilyPreview> = families_map.into_values().collect();
    let total_members: usize = families.iter().map(|f| f.members.len()).sum();

    Ok(ImportPreview {
        total_families: families.len(),
        total_members,
        families,
        duplicates,
    })
}

#[tauri::command]
pub fn execute_import(
    db: State<'_, Database>,
    file_path: String,
    update_duplicates: bool,
) -> Result<ImportResult, String> {
    let rows = parse_xlsx(&file_path)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut result = ImportResult {
        families_created: 0,
        families_updated: 0,
        members_created: 0,
        members_updated: 0,
        errors: Vec::new(),
    };

    let mut families_map: HashMap<String, Vec<&ImportRow>> = HashMap::new();
    for row in &rows {
        families_map.entry(row.family_id.clone()).or_default().push(row);
    }

    for (family_id, family_rows) in families_map {
        let first_row = family_rows.first().unwrap();

        let existing_family_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM families WHERE family_id = ?",
                params![family_id],
                |row| row.get(0),
            )
            .ok();

        let db_family_id = if let Some(existing_id) = existing_family_id {
            if update_duplicates {
                if let Err(e) = conn.execute(
                    "UPDATE families SET name = ?, mailing_name = ?, address = ?, city = ?, state = ?, zip = ?, phone = ?, email = ?, updated_at = datetime('now') WHERE id = ?",
                    params![
                        first_row.family_name,
                        first_row.mailing_name,
                        first_row.address,
                        first_row.city,
                        first_row.state,
                        first_row.zip,
                        first_row.phone,
                        first_row.email,
                        existing_id,
                    ],
                ) {
                    result.errors.push(format!("Failed to update family {}: {}", family_id, e));
                    continue;
                }
                result.families_updated += 1;
            }
            existing_id
        } else {
            match conn.execute(
                "INSERT INTO families (family_id, name, mailing_name, address, city, state, zip, phone, email) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    family_id,
                    first_row.family_name,
                    first_row.mailing_name,
                    first_row.address,
                    first_row.city,
                    first_row.state,
                    first_row.zip,
                    first_row.phone,
                    first_row.email,
                ],
            ) {
                Ok(_) => {
                    result.families_created += 1;
                    conn.last_insert_rowid()
                }
                Err(e) => {
                    result.errors.push(format!("Failed to create family {}: {}", family_id, e));
                    continue;
                }
            }
        };

        for (sort_order, row) in family_rows.iter().enumerate() {
            let existing_member_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM members WHERE family_id = ? AND LOWER(first_name) = LOWER(?) AND LOWER(last_name) = LOWER(?)",
                    params![db_family_id, row.member_first_name, row.member_last_name],
                    |row| row.get(0),
                )
                .ok();

            if let Some(member_id) = existing_member_id {
                if update_duplicates {
                    if let Err(e) = conn.execute(
                        "UPDATE members SET role = ?, phone = ?, email = ?, birth_date = ?, wedding_date = ?, sort_order = ?, updated_at = datetime('now') WHERE id = ?",
                        params![row.member_role, row.member_phone, row.member_email, row.member_birth_date, row.member_wedding_date, sort_order as i32, member_id],
                    ) {
                        result.errors.push(format!("Failed to update member {} {}: {}", row.member_first_name, row.member_last_name, e));
                    } else {
                        result.members_updated += 1;
                    }
                }
            } else {
                if let Err(e) = conn.execute(
                    "INSERT INTO members (family_id, first_name, last_name, role, phone, email, birth_date, wedding_date, sort_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![db_family_id, row.member_first_name, row.member_last_name, row.member_role, row.member_phone, row.member_email, row.member_birth_date, row.member_wedding_date, sort_order as i32],
                ) {
                    result.errors.push(format!("Failed to create member {} {}: {}", row.member_first_name, row.member_last_name, e));
                } else {
                    result.members_created += 1;
                }
            }
        }
    }

    let filename = Path::new(&file_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.clone());

    let _ = conn.execute(
        "INSERT INTO import_history (filename, families_created, families_updated, members_created, members_updated) VALUES (?, ?, ?, ?, ?)",
        params![
            filename,
            result.families_created,
            result.families_updated,
            result.members_created,
            result.members_updated,
        ],
    );

    Ok(result)
}
