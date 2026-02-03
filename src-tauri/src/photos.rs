use crate::db::Database;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::imageops::FilterType;
use image::GenericImageView;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, State};
use uuid::Uuid;

const MAX_IMAGE_SIZE: u32 = 1200;
const THUMBNAIL_SIZE: u32 = 200;

#[tauri::command]
pub fn save_family_photo(
    app_handle: AppHandle,
    db: State<'_, Database>,
    family_id: i64,
    source_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let families_dir = photos_dir.join("families");
    fs::create_dir_all(&families_dir).map_err(|e| e.to_string())?;

    let saved_path = process_and_save_image(&source_path, &families_dir, "family")?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE families SET photo_path = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![saved_path, family_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(saved_path)
}

#[tauri::command]
pub fn save_member_photo(
    app_handle: AppHandle,
    db: State<'_, Database>,
    member_id: i64,
    source_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let members_dir = photos_dir.join("members");
    fs::create_dir_all(&members_dir).map_err(|e| e.to_string())?;

    let saved_path = process_and_save_image(&source_path, &members_dir, "member")?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE members SET photo_path = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![saved_path, member_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(saved_path)
}

#[tauri::command]
pub fn save_church_logo(
    app_handle: AppHandle,
    file_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    fs::create_dir_all(&photos_dir).map_err(|e| e.to_string())?;

    let saved_path = process_and_save_image(&file_path, &photos_dir, "logo")?;
    Ok(saved_path)
}

#[tauri::command]
pub fn delete_photo(
    app_handle: AppHandle,
    photo_path: String,
) -> Result<(), String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let full_path = photos_dir.join(&photo_path);

    if full_path.exists() {
        fs::remove_file(&full_path).map_err(|e| e.to_string())?;
    }

    let thumb_path = get_thumbnail_path(&full_path);
    if thumb_path.exists() {
        fs::remove_file(&thumb_path).map_err(|e| e.to_string())?;
    }

    let full_res_path = get_full_resolution_path(&full_path);
    if full_res_path.exists() {
        fs::remove_file(&full_res_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_photo_path(app_handle: AppHandle, relative_path: String) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let full_path = photos_dir.join(&relative_path);
    Ok(full_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn crop_family_photo_to_member(
    app_handle: AppHandle,
    db: State<'_, Database>,
    family_id: i64,
    member_id: i64,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get the family's photo path
    let family_photo: String = conn
        .query_row(
            "SELECT photo_path FROM families WHERE id = ?",
            rusqlite::params![family_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get family photo: {}", e))?;

    let photos_dir = Database::get_photos_dir(&app_handle);
    let source_path = photos_dir.join("families").join(&family_photo);

    if !source_path.exists() {
        return Err(format!("Family photo not found: {:?}", source_path));
    }

    // Open and crop the image
    let img = image::open(&source_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Ensure crop bounds are within image dimensions
    let (img_width, img_height) = img.dimensions();
    let crop_x = x.min(img_width.saturating_sub(1));
    let crop_y = y.min(img_height.saturating_sub(1));
    let crop_width = width.min(img_width.saturating_sub(crop_x));
    let crop_height = height.min(img_height.saturating_sub(crop_y));

    if crop_width == 0 || crop_height == 0 {
        return Err("Invalid crop dimensions".to_string());
    }

    let cropped = img.crop_imm(crop_x, crop_y, crop_width, crop_height);

    // Save to members directory
    let members_dir = photos_dir.join("members");
    fs::create_dir_all(&members_dir).map_err(|e| e.to_string())?;

    let uuid = Uuid::new_v4();
    let filename = format!("member_{}.jpg", uuid);
    let dest_path = members_dir.join(&filename);

    // Resize to standard member photo size for display
    let resized = cropped.resize(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE, FilterType::Lanczos3);
    resized
        .to_rgb8()
        .save(&dest_path)
        .map_err(|e| format!("Failed to save cropped image: {}", e))?;

    // Save full resolution version for print quality PDFs
    let full_path = get_full_resolution_path(&dest_path);
    cropped
        .to_rgb8()
        .save(&full_path)
        .map_err(|e| format!("Failed to save full resolution cropped image: {}", e))?;

    // Create thumbnail
    let thumbnail = cropped.resize_exact(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);
    let thumb_path = get_thumbnail_path(&dest_path);
    thumbnail
        .to_rgb8()
        .save(&thumb_path)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    // Update member's photo_path in database
    conn.execute(
        "UPDATE members SET photo_path = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![filename, member_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(filename)
}

#[tauri::command]
pub fn get_photo_base64(
    app_handle: AppHandle,
    photo_type: String,
    filename: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let photo_path = photos_dir.join(&photo_type).join(&filename);

    if !photo_path.exists() {
        return Err(format!("Photo not found: {:?}", photo_path));
    }

    let data = fs::read(&photo_path).map_err(|e| format!("Failed to read photo: {}", e))?;
    let base64_data = STANDARD.encode(&data);

    let mime_type = if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".gif") {
        "image/gif"
    } else if filename.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

fn process_and_save_image(
    source_path: &str,
    dest_dir: &Path,
    prefix: &str,
) -> Result<String, String> {
    let img = image::open(source_path).map_err(|e| format!("Failed to open image: {}", e))?;

    let (width, height) = img.dimensions();
    let resized = if width > MAX_IMAGE_SIZE || height > MAX_IMAGE_SIZE {
        img.resize(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE, FilterType::Lanczos3)
    } else {
        img.clone()
    };

    let uuid = Uuid::new_v4();
    let filename = format!("{}_{}.jpg", prefix, uuid);
    let dest_path = dest_dir.join(&filename);

    // Save display version (max 1200px for UI)
    resized
        .to_rgb8()
        .save(&dest_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    // Save full resolution version for print quality PDFs
    let full_path = get_full_resolution_path(&dest_path);
    img.to_rgb8()
        .save(&full_path)
        .map_err(|e| format!("Failed to save full resolution image: {}", e))?;

    // Save thumbnail
    let thumbnail = img.resize_exact(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);
    let thumb_path = get_thumbnail_path(&dest_path);
    thumbnail
        .to_rgb8()
        .save(&thumb_path)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    Ok(filename)
}

fn get_thumbnail_path(image_path: &Path) -> std::path::PathBuf {
    let stem = image_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = image_path.parent().unwrap_or(Path::new(""));
    parent.join(format!("{}_thumb.jpg", stem))
}

fn get_full_resolution_path(image_path: &Path) -> std::path::PathBuf {
    let stem = image_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = image_path.parent().unwrap_or(Path::new(""));
    parent.join(format!("{}_full.jpg", stem))
}
