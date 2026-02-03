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
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
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
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
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
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
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
    pub layout: String,
    pub page_size: String,
    pub include_photos: bool,
    pub include_contact_info: bool,
    pub include_address: bool,
    pub include_cover: bool,
    pub include_toc: bool,
    pub church_name: String,
    pub church_logo_path: Option<String>,
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
