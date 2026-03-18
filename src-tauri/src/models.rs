use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Family {
    pub id: i64,
    pub family_id: String,
    pub name: String,
    pub mailing_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub children: Option<String>,
    pub alt_address: Option<String>,
    pub alt_city: Option<String>,
    pub alt_state: Option<String>,
    pub alt_zip: Option<String>,
    pub directory_adults: Option<String>,
    pub directory_children: Option<String>,
    pub include_photo_in_directory: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyInput {
    pub family_id: String,
    pub name: String,
    pub mailing_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub children: Option<String>,
    pub alt_address: Option<String>,
    pub alt_city: Option<String>,
    pub alt_state: Option<String>,
    pub alt_zip: Option<String>,
    pub directory_adults: Option<String>,
    pub directory_children: Option<String>,
    #[serde(default = "default_include_photo")]
    pub include_photo_in_directory: bool,
}

fn default_include_photo() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyUpdate {
    pub family_id: Option<String>,
    pub name: Option<String>,
    pub mailing_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub photo_path: Option<Option<String>>,
    pub notes: Option<String>,
    pub children: Option<String>,
    pub alt_address: Option<String>,
    pub alt_city: Option<String>,
    pub alt_state: Option<String>,
    pub alt_zip: Option<String>,
    pub directory_adults: Option<String>,
    pub directory_children: Option<String>,
    pub include_photo_in_directory: Option<bool>,
}

fn deserialize_nullable_field<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // If the field is present, deserialize its value (which may be null)
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: i64,
    pub family_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub birth_date: Option<String>,
    pub wedding_date: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInput {
    pub family_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub birth_date: Option<String>,
    pub wedding_date: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub birth_date: Option<String>,
    pub wedding_date: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyWithMembers {
    #[serde(flatten)]
    pub family: Family,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub church_name: String,
    pub church_logo_path: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub default_layout: String,
    pub page_size: String,
    pub include_photos: bool,
    pub include_contact_info: bool,
    pub include_address: bool,
    pub cover_image_path: Option<String>,
    pub cover_title_line1: Option<String>,
    pub cover_title_line2: Option<String>,
    pub cover_title_color: Option<String>,
    pub pastor_letter: Option<String>,
    pub mission_statement: Option<String>,
    pub first_page_markdown: Option<String>,
    pub back_cover_image_path: Option<String>,
    pub celebration_image_path: Option<String>,
    pub church_address: Option<String>,
    pub church_phone: Option<String>,
    pub church_email: Option<String>,
    pub church_website: Option<String>,
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            church_name: "Our Church".to_string(),
            church_logo_path: None,
            theme: "system".to_string(),
            default_layout: "grid".to_string(),
            page_size: "letter".to_string(),
            include_photos: true,
            include_contact_info: true,
            include_address: true,
            cover_image_path: None,
            cover_title_line1: None,
            cover_title_line2: None,
            cover_title_color: Some("#FFFFFF".to_string()),
            pastor_letter: None,
            mission_statement: None,
            first_page_markdown: None,
            back_cover_image_path: None,
            celebration_image_path: None,
            church_address: None,
            church_phone: None,
            church_email: None,
            church_website: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub families: Vec<ImportFamilyPreview>,
    pub total_families: usize,
    pub total_members: usize,
    pub duplicates: Vec<DuplicateMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportFamilyPreview {
    pub family_id: String,
    pub name: String,
    pub address: Option<String>,
    pub members: Vec<ImportMemberPreview>,
    pub is_duplicate: bool,
    pub existing_family_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMemberPreview {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMatch {
    pub import_family_id: String,
    pub import_name: String,
    pub existing_id: i64,
    pub existing_name: String,
    pub match_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub families_created: usize,
    pub families_updated: usize,
    pub members_created: usize,
    pub members_updated: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfOptions {
    pub church_name: String,
    pub cover_image_path: Option<String>,
    pub cover_title_line1: Option<String>,
    pub cover_title_line2: Option<String>,
    pub cover_title_color: Option<String>,
    pub pastor_letter: Option<String>,
    pub mission_statement: Option<String>,
    pub first_page_markdown: Option<String>,
    pub back_cover_image_path: Option<String>,
    pub celebration_image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leadership {
    pub id: i64,
    pub ministry: String,
    pub names: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadershipInput {
    pub ministry: String,
    pub names: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub role: String,
    pub photo_path: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffInput {
    pub name: String,
    #[serde(default)]
    pub title: String,
    pub role: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: String,
    pub created_at: String,
    pub app_version: String,
    pub family_count: usize,
    pub member_count: usize,
    pub photo_count: usize,
}
