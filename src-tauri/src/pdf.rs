use crate::db::Database;
use crate::models::{Family, FamilyWithMembers, Member, PdfOptions};
use printpdf::*;
use rusqlite::params;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use tauri::{AppHandle, State};

// Content page dimensions (half of US Legal sheet folded)
const PAGE_WIDTH_MM: f32 = 177.8; // 7 inches
const PAGE_HEIGHT_MM: f32 = 215.9; // 8.5 inches

// Sheet dimensions (US Legal 8.5x14 landscape)
const SHEET_WIDTH_MM: f32 = 355.6; // 14 inches
const SHEET_HEIGHT_MM: f32 = 215.9; // 8.5 inches

// Photo grid layout: 2 columns x 4 rows
const GRID_MARGIN_X: f32 = 8.0;
const GRID_MARGIN_TOP: f32 = 8.0;
const GRID_MARGIN_BOTTOM: f32 = 1.0;
const GRID_COLS: usize = 2;
const GRID_ROWS: usize = 5;
const GRID_CAPTION_HEIGHT: f32 = 9.0; // Space below photo for name+adults line + children line
const GRID_COL_SPACING: f32 = 4.0;
const GRID_ROW_SPACING: f32 = 0.75;
const GRID_CELL_PADDING: f32 = 0.5;

// Text card layout
const CARD_MARGIN: f32 = 15.0;
const CARD_COLS: usize = 2;
const CARD_COL_GAP: f32 = 10.0;
const CARD_SPACING: f32 = 6.0;
const CARD_NAME_SIZE: f32 = 11.0;
const CARD_TEXT_SIZE: f32 = 9.0;
const CARD_LINE_HEIGHT: f32 = 4.5;

// Font sizes
const GRID_NAME_SIZE: f32 = 9.0;

// Celebration page layout
const CELEB_MARGIN_TOP: f32 = 15.0;
const CELEB_MARGIN_BOTTOM: f32 = 17.0;
const CELEB_TITLE_SIZE: f32 = 16.0;
const CELEB_MONTH_SIZE: f32 = 12.0;
const CELEB_SECTION_SIZE: f32 = 8.0;
const CELEB_TEXT_SIZE: f32 = 10.0;
const CELEB_LINE_HEIGHT: f32 = 4.6;
const CELEB_MONTH_SPACING: f32 = 6.0;
const CELEB_SECTION_SPACING: f32 = 3.0;
const CELEB_COL_GAP: f32 = 8.0;

/// Get the full resolution path for a photo (used for print quality PDFs)
fn get_full_resolution_path(image_path: &PathBuf) -> PathBuf {
    let stem = image_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = image_path.parent().unwrap_or(std::path::Path::new(""));
    parent.join(format!("{}_full.jpg", stem))
}

// Content page types for the two-pass approach
enum ContentPage {
    FullImage(PathBuf),
    CoverImage {
        image_path: PathBuf,
        title_line1: String,
        title_line2: String,
        title_color: String,
    },
    PhotoGrid { entries: Vec<PhotoGridEntry>, grid_rows: usize },
    TextCards([Vec<FamilyCardData>; 2]),
    CelebrationPages(CelebPage),
    MarkdownContent(String),
    FirstInsidePage {
        pastor_letter: Option<String>,
        mission_statement: Option<String>,
    },
    StaffPage(Vec<StaffEntry>),
    LeadershipPage {
        entries: Vec<LeadershipEntry>,
        contact_left: Vec<String>,
        contact_right: Vec<String>,
    },
    Blank,
}

struct LeadershipEntry {
    ministry: String,
    names: String,
}

struct StaffEntry {
    name: String,
    title: String,
    role: String,
    photo_path: Option<PathBuf>,
}

impl ContentPage {
    /// All content pages get footers except cover/back images and blank pages.
    fn needs_footer(&self) -> bool {
        !matches!(self, ContentPage::FullImage(_) | ContentPage::CoverImage { .. } | ContentPage::Blank)
    }
}

struct CelebrationEntry {
    name: String,
    day: u32,
    display_date: String, // e.g., "March 15"
}

struct CelebrationMonth {
    month_name: String,
    birthdays: Vec<CelebrationEntry>,
    anniversaries: Vec<CelebrationEntry>,
}

/// A single renderable item in the celebration flow layout
#[derive(Clone)]
enum CelebItem {
    /// Month heading, with flag indicating if it's a continuation
    MonthHeader { name: String, continued: bool },
    /// "Birthdays" or "Anniversaries" section label
    SectionHeader(String),
    /// A single date + name entry
    Entry { display_date: String, name: String },
    /// Vertical spacing between months
    MonthGap,
}

impl CelebItem {
    fn height(&self) -> f32 {
        match self {
            CelebItem::MonthHeader { .. } => 2.0 + 5.0, // text + accent line + gap
            CelebItem::SectionHeader(_) => CELEB_SECTION_SIZE * 0.4 + CELEB_SECTION_SPACING * 0.5,
            CelebItem::Entry { .. } => CELEB_LINE_HEIGHT,
            CelebItem::MonthGap => CELEB_MONTH_SPACING,
        }
    }
}

/// A page of celebrations: two columns of items, plus first-page flag and image
struct CelebPage {
    left: Vec<CelebItem>,
    right: Vec<CelebItem>,
    is_first: bool,
    image_path: Option<PathBuf>,
}

struct PhotoGridEntry {
    photo_path: PathBuf,
    family_name: String,
    display_name: String,
    directory_adults: Option<String>,
    directory_children: Option<String>,
}

#[derive(Clone)]
struct MemberCardInfo {
    name: String,
    phone: Option<String>,
    email: Option<String>,
    photo_path: Option<PathBuf>,
}

#[derive(Clone)]
struct FamilyCardData {
    name: String,
    mailing_name: Option<String>,
    address: Option<String>,
    city_state_zip: Option<String>,
    alt_address: Option<String>,
    alt_city_state_zip: Option<String>,
    phone: Option<String>,
    members: Vec<MemberCardInfo>,
}

impl FamilyCardData {
    fn height(&self) -> f32 {
        // Base height from family info lines
        let mut h = CARD_NAME_SIZE * 0.6;
        let mut family_lines = 0;
        if self.mailing_name.is_some() { family_lines += 1; }
        if self.address.is_some() { family_lines += 1; }
        if self.city_state_zip.is_some() { family_lines += 1; }
        if self.alt_address.is_some() || self.alt_city_state_zip.is_some() {
            family_lines += 1;
            if self.alt_address.is_some() { family_lines += 1; }
            if self.alt_city_state_zip.is_some() { family_lines += 1; }
        }
        if self.phone.is_some() { family_lines += 1; }
        h += family_lines as f32 * CARD_LINE_HEIGHT;

        // If any member has a photo, all members get photo-height spacing for alignment
        let any_member_has_photo = self.members.iter().any(|m| m.photo_path.is_some());
        for member in &self.members {
            let mut member_text_h = CARD_LINE_HEIGHT; // name line
            if member.phone.is_some() { member_text_h += CARD_LINE_HEIGHT; }
            if member.email.is_some() { member_text_h += CARD_LINE_HEIGHT; }
            if any_member_has_photo {
                h += member_text_h.max(MEMBER_PHOTO_SIZE_MM + 1.0);
            } else {
                h += member_text_h;
            }
        }

        h + CARD_SPACING
    }
}

#[tauri::command]
pub fn generate_pdf(
    app_handle: AppHandle,
    db: State<'_, Database>,
    options: PdfOptions,
    output_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);

    // Query all families with members
    let families_with_members: Vec<FamilyWithMembers> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let mut family_stmt = conn
            .prepare(
                "SELECT id, family_id, name, mailing_name, address, city, state, zip, phone,
                        photo_path, notes, children, alt_address, alt_city, alt_state, alt_zip,
                        directory_adults, directory_children, include_photo_in_directory, created_at, updated_at
                 FROM families ORDER BY name",
            )
            .map_err(|e| e.to_string())?;

        let families = family_stmt
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

        let mut result = Vec::new();
        for family in families {
            let mut member_stmt = conn
                .prepare(
                    "SELECT id, family_id, first_name, last_name, role, birth_date, wedding_date, phone, email,
                            photo_path, notes, sort_order, created_at, updated_at
                     FROM members WHERE family_id = ? ORDER BY sort_order, last_name, first_name",
                )
                .map_err(|e| e.to_string())?;

            let members = member_stmt
                .query_map(params![family.id], |row| {
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

            result.push(FamilyWithMembers { family, members });
        }
        result
    };

    // Load staff data
    let staff_entries: Vec<StaffEntry> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT name, title, role, photo_path FROM staff ORDER BY sort_order, name",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, String, String, Option<String>)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        rows.into_iter()
            .map(|(name, title, role, photo_filename)| {
                let photo_path = photo_filename.and_then(|f| {
                    let base = photos_dir.join("staff").join(&f);
                    let full_res = get_full_resolution_path(&base);
                    if full_res.exists() {
                        Some(full_res)
                    } else if base.exists() {
                        Some(base)
                    } else {
                        None
                    }
                });
                StaffEntry { name, title, role, photo_path }
            })
            .collect()
    };

    // ========================================
    // Pass 1: Build content pages in reading order
    // ========================================
    let mut content_pages: Vec<ContentPage> = Vec::new();

    // Page 1: Cover image with title text
    {
        let cover_base = resolve_image_page(&options.cover_image_path, &photos_dir);
        let line1 = options.cover_title_line1.clone().unwrap_or_default();
        let line2 = options.cover_title_line2.clone().unwrap_or_default();
        let color = options.cover_title_color.clone().unwrap_or_else(|| "#FFFFFF".to_string());
        if let ContentPage::FullImage(path) = cover_base {
            content_pages.push(ContentPage::CoverImage {
                image_path: path,
                title_line1: line1,
                title_line2: line2,
                title_color: color,
            });
        } else if !line1.is_empty() || !line2.is_empty() {
            content_pages.push(ContentPage::CoverImage {
                image_path: PathBuf::new(),
                title_line1: line1,
                title_line2: line2,
                title_color: color,
            });
        } else {
            content_pages.push(cover_base);
        }
    }

    // Pastor letter + mission statement (inside front cover)
    let has_pastor = options.pastor_letter.as_ref().map_or(false, |s| !s.is_empty());
    let has_mission = options.mission_statement.as_ref().map_or(false, |s| !s.is_empty());
    if has_pastor || has_mission {
        content_pages.push(ContentPage::FirstInsidePage {
            pastor_letter: options.pastor_letter.clone(),
            mission_statement: options.mission_statement.clone(),
        });
    } else if let Some(ref markdown) = options.first_page_markdown {
        if !markdown.is_empty() {
            content_pages.push(ContentPage::MarkdownContent(markdown.clone()));
        }
    }

    // Pastor, Elders & Staff
    if !staff_entries.is_empty() {
        content_pages.push(ContentPage::StaffPage(staff_entries));
    }

    // Page 4: Ministry Team Leadership
    let leadership_entries: Vec<LeadershipEntry> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT ministry, names FROM leadership ORDER BY ministry COLLATE NOCASE")
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        rows.into_iter()
            .map(|(ministry, names)| LeadershipEntry { ministry, names })
            .collect()
    };

    if !leadership_entries.is_empty() {
        // Build contact info lines from settings
        let (contact_left, contact_right) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let mut left_lines = Vec::new();
            let mut right_lines = Vec::new();

            // Left column: address + phone
            for key in &["church_address", "church_phone"] {
                if let Ok(value) = conn.query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    params![key],
                    |row| row.get::<_, String>(0),
                ) {
                    if !value.is_empty() {
                        for line in value.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                left_lines.push(trimmed.to_string());
                            }
                        }
                    }
                }
            }

            // Right column: email + website
            for key in &["church_email", "church_website"] {
                if let Ok(value) = conn.query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    params![key],
                    |row| row.get::<_, String>(0),
                ) {
                    if !value.is_empty() {
                        right_lines.push(value);
                    }
                }
            }

            (left_lines, right_lines)
        };

        content_pages.push(ContentPage::LeadershipPage {
            entries: leadership_entries,
            contact_left,
            contact_right,
        });
    }

    // Pages 5+: Photo grid (only families with photos and include_photo_in_directory enabled)
    let mut grid_entries: Vec<PhotoGridEntry> = Vec::new();
    for fwm in &families_with_members {
        if !fwm.family.include_photo_in_directory {
            continue;
        }
        if let Some(ref photo_path) = fwm.family.photo_path {
            let base_path = photos_dir.join("families").join(photo_path);
            let full_res_path = get_full_resolution_path(&base_path);
            let photo_file = if full_res_path.exists() {
                full_res_path
            } else {
                base_path
            };

            if photo_file.exists() {
                let display_name = fwm
                    .family
                    .mailing_name
                    .clone()
                    .unwrap_or_else(|| fwm.family.name.clone());
                grid_entries.push(PhotoGridEntry {
                    photo_path: photo_file,
                    family_name: fwm.family.name.clone(),
                    display_name,
                    directory_adults: fwm.family.directory_adults.clone(),
                    directory_children: fwm.family.directory_children.clone(),
                });
            }
        }
    }

    // Chunk grid entries into pages, laid out column-first (down left column, then right)
    // so alphabetical order reads top-to-bottom within each column.
    let grid_rows = options.photo_grid_rows.unwrap_or(4);
    let entries_per_page = GRID_COLS * grid_rows;
    for chunk in grid_entries.chunks(entries_per_page) {
        let page_entries: Vec<PhotoGridEntry> = chunk
            .iter()
            .map(|e| PhotoGridEntry {
                photo_path: e.photo_path.clone(),
                family_name: e.family_name.clone(),
                display_name: e.display_name.clone(),
                directory_adults: e.directory_adults.clone(),
                directory_children: e.directory_children.clone(),
            })
            .collect();

        content_pages.push(ContentPage::PhotoGrid { entries: page_entries, grid_rows });
    }

    // Text-only card pages for ALL families
    let card_data: Vec<FamilyCardData> = families_with_members
        .iter()
        .map(|fwm| {
            let mut city_state_zip_parts: Vec<String> = Vec::new();
            if let Some(ref city) = fwm.family.city.as_ref().filter(|s| !s.trim().is_empty()) {
                city_state_zip_parts.push(city.to_string());
            }
            if let Some(ref state) = fwm.family.state.as_ref().filter(|s| !s.trim().is_empty()) {
                if let Some(last) = city_state_zip_parts.pop() {
                    city_state_zip_parts.push(format!("{}, {}", last, state));
                } else {
                    city_state_zip_parts.push(state.to_string());
                }
            }
            if let Some(ref zip) = fwm.family.zip.as_ref().filter(|s| !s.trim().is_empty()) {
                city_state_zip_parts.push(zip.to_string());
            }
            let city_state_zip = if city_state_zip_parts.is_empty() {
                None
            } else {
                let joined = city_state_zip_parts.join(" ");
                let trimmed = joined.trim().trim_matches(',').trim();
                if trimmed.is_empty() { None } else { Some(joined) }
            };

            // Build alt city/state/zip
            let mut alt_csz_parts: Vec<String> = Vec::new();
            if let Some(ref city) = fwm.family.alt_city.as_ref().filter(|s| !s.trim().is_empty()) {
                alt_csz_parts.push(city.to_string());
            }
            if let Some(ref state) = fwm.family.alt_state.as_ref().filter(|s| !s.trim().is_empty()) {
                if let Some(last) = alt_csz_parts.pop() {
                    alt_csz_parts.push(format!("{}, {}", last, state));
                } else {
                    alt_csz_parts.push(state.to_string());
                }
            }
            if let Some(ref zip) = fwm.family.alt_zip.as_ref().filter(|s| !s.trim().is_empty()) {
                alt_csz_parts.push(zip.to_string());
            }
            let alt_city_state_zip = if alt_csz_parts.is_empty() {
                None
            } else {
                let joined = alt_csz_parts.join(" ");
                let trimmed = joined.trim().trim_matches(',').trim();
                if trimmed.is_empty() { None } else { Some(joined) }
            };

            let members: Vec<MemberCardInfo> = fwm.members.iter().map(|m| {
                let photo_path = m.photo_path.as_ref().and_then(|pp| {
                    let base = photos_dir.join("members").join(pp);
                    let full = get_full_resolution_path(&base);
                    let resolved = if full.exists() { full } else { base };
                    if resolved.exists() { Some(resolved) } else { None }
                });
                MemberCardInfo {
                    name: format!("{} {}", m.first_name, m.last_name),
                    phone: m.phone.clone().filter(|s| !s.trim().is_empty()),
                    email: m.email.clone().filter(|s| !s.trim().is_empty()),
                    photo_path,
                }
            }).collect();

            FamilyCardData {
                name: fwm.family.name.clone(),
                mailing_name: fwm.family.mailing_name.clone().filter(|s| !s.trim().is_empty() && s.trim() != ","),
                address: fwm.family.address.clone().filter(|s| !s.trim().is_empty()),
                city_state_zip,
                alt_address: fwm.family.alt_address.clone().filter(|s| !s.trim().is_empty()),
                alt_city_state_zip,
                phone: fwm.family.phone.clone().filter(|s| !s.trim().is_empty()),
                members,
            }
        })
        .collect();

    // Paginate cards into pages
    let card_pages = paginate_cards(&card_data);
    for page_cards in card_pages {
        content_pages.push(ContentPage::TextCards(page_cards));
    }

    // Build birthday & anniversary celebration pages
    let month_names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let mut celebration_months: Vec<CelebrationMonth> = (0..12)
        .map(|i| CelebrationMonth {
            month_name: month_names[i].to_string(),
            birthdays: Vec::new(),
            anniversaries: Vec::new(),
        })
        .collect();

    for fwm in &families_with_members {
        for member in &fwm.members {
            if let Some(ref bd) = member.birth_date {
                if let Some((month_idx, day, display)) = parse_month_day(bd) {
                    celebration_months[month_idx].birthdays.push(CelebrationEntry {
                        name: format!("{} {}", member.first_name, member.last_name),
                        day,
                        display_date: display,
                    });
                }
            }
            if let Some(ref wd) = member.wedding_date {
                if let Some((month_idx, day, display)) = parse_month_day(wd) {
                    // Use family name for anniversaries (couple event)
                    let ann_name = fwm.family.mailing_name.clone()
                        .unwrap_or_else(|| fwm.family.name.clone());
                    celebration_months[month_idx].anniversaries.push(CelebrationEntry {
                        name: ann_name,
                        day,
                        display_date: display,
                    });
                }
            }
        }
    }

    // Sort entries by day within each month and deduplicate anniversaries
    for month in &mut celebration_months {
        month.birthdays.sort_by_key(|e| e.day);
        month.anniversaries.sort_by_key(|e| e.day);
        month.anniversaries.dedup_by(|a, b| a.name == b.name && a.day == b.day);
    }

    // Add celebration pages immediately after directory content (no blank gap)
    let has_celebrations = celebration_months.iter().any(|m| !m.birthdays.is_empty() || !m.anniversaries.is_empty());

    if has_celebrations {
        let celeb_image = options.celebration_image_path.as_ref().and_then(|filename| {
            let full_path = photos_dir.join("directory").join(filename);
            let full_res = get_full_resolution_path(&full_path);
            let path = if full_res.exists() { full_res } else { full_path };
            if path.exists() { Some(path) } else { None }
        });

        let celeb_pages = paginate_celebrations(&celebration_months, celeb_image);
        for page in celeb_pages {
            content_pages.push(ContentPage::CelebrationPages(page));
        }
    }

    // Pad with blanks at the end to reach a multiple of 4
    while content_pages.len() % 4 != 0 {
        content_pages.push(ContentPage::Blank);
    }

    // ========================================
    // Pass 2: Imposition and PDF rendering
    // ========================================
    let total = content_pages.len();
    let num_sheets = total / 4;
    let current_year = chrono::Utc::now().format("%Y").to_string();

    let (doc, page1, layer1) = PdfDocument::new(
        &options.church_name,
        Mm(SHEET_WIDTH_MM),
        Mm(SHEET_HEIGHT_MM),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Font error: {:?}", e))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Font error: {:?}", e))?;

    for s in 0..num_sheets {
        // Front side of sheet
        let left_idx = total - 1 - 2 * s;
        let right_idx = 2 * s;

        let front_layer = if s == 0 {
            doc.get_page(page1).get_layer(layer1)
        } else {
            let (new_page, new_layer) =
                doc.add_page(Mm(SHEET_WIDTH_MM), Mm(SHEET_HEIGHT_MM), "Sheet");
            doc.get_page(new_page).get_layer(new_layer)
        };

        render_content_page(
            &doc,
            &front_layer,
            &content_pages[left_idx],
            0.0,
            true,
            &font,
            &font_bold,
        );
        if content_pages[left_idx].needs_footer() {
            render_footer(&front_layer, 0.0, &options.church_name, &current_year, &font);
        }
        render_content_page(
            &doc,
            &front_layer,
            &content_pages[right_idx],
            PAGE_WIDTH_MM,
            false,
            &font,
            &font_bold,
        );
        if content_pages[right_idx].needs_footer() {
            render_footer(&front_layer, PAGE_WIDTH_MM, &options.church_name, &current_year, &font);
        }

        // Back side of sheet
        let left_idx = 2 * s + 1;
        let right_idx = total - 2 - 2 * s;

        let (back_page, back_layer_id) =
            doc.add_page(Mm(SHEET_WIDTH_MM), Mm(SHEET_HEIGHT_MM), "Sheet");
        let back_layer = doc.get_page(back_page).get_layer(back_layer_id);

        render_content_page(
            &doc,
            &back_layer,
            &content_pages[left_idx],
            0.0,
            true,
            &font,
            &font_bold,
        );
        if content_pages[left_idx].needs_footer() {
            render_footer(&back_layer, 0.0, &options.church_name, &current_year, &font);
        }
        render_content_page(
            &doc,
            &back_layer,
            &content_pages[right_idx],
            PAGE_WIDTH_MM,
            false,
            &font,
            &font_bold,
        );
        if content_pages[right_idx].needs_footer() {
            render_footer(&back_layer, PAGE_WIDTH_MM, &options.church_name, &current_year, &font);
        }
    }

    let file = File::create(&output_path).map_err(|e| e.to_string())?;
    let mut buf_writer = BufWriter::new(file);
    doc.save(&mut buf_writer)
        .map_err(|e| format!("Save error: {:?}", e))?;

    Ok(output_path)
}

// Page border (1/4 inch = 6.35mm)
const PAGE_BORDER_INSET: f32 = 6.35;
const FOOTER_MARGIN: f32 = 10.0;
const FOOTER_FONT_SIZE: f32 = 7.0;
const FOOTER_Y: f32 = 10.0;

fn render_footer(
    layer: &PdfLayerReference,
    x_offset: f32,
    church_name: &str,
    year: &str,
    font: &IndirectFontRef,
) {
    layer.set_fill_color(Color::Greyscale(Greyscale::new(0.35, None)));
    // Left: church name
    layer.use_text(
        church_name,
        FOOTER_FONT_SIZE,
        Mm(x_offset + FOOTER_MARGIN),
        Mm(FOOTER_Y),
        font,
    );
    // Right: year — estimate text width for right-alignment
    let year_width = year.len() as f32 * FOOTER_FONT_SIZE * 0.22;
    layer.use_text(
        year,
        FOOTER_FONT_SIZE,
        Mm(x_offset + PAGE_WIDTH_MM - FOOTER_MARGIN - year_width),
        Mm(FOOTER_Y),
        font,
    );
    // Reset fill color to black
    layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
}

fn ordinal_suffix(day: u32) -> &'static str {
    match (day % 10, day % 100) {
        (1, 11) => "th",
        (2, 12) => "th",
        (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

/// Parse a date string into (month_index 0-11, day, display_string like "March 15th")
fn parse_month_day(date_str: &str) -> Option<(usize, u32, String)> {
    let parts: Vec<&str> = date_str.split('-').collect();
    match parts.len() {
        2 => {
            // MM-DD format
            let month: u32 = parts[0].parse().ok()?;
            let day: u32 = parts[1].parse().ok()?;
            if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                let month_idx = (month - 1) as usize;
                Some((month_idx, day, format!("{}{}", day, ordinal_suffix(day))))
            } else {
                None
            }
        }
        3 => {
            // YYYY-MM-DD format — ignore year
            let month: u32 = parts[1].parse().ok()?;
            let day: u32 = parts[2].parse().ok()?;
            if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                let month_idx = (month - 1) as usize;
                Some((month_idx, day, format!("{}{}", day, ordinal_suffix(day))))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Estimate the height of a celebration month block in mm

/// Flatten celebration months into a linear stream of renderable items
fn flatten_celebrations(months: &[CelebrationMonth]) -> Vec<CelebItem> {
    let mut items: Vec<CelebItem> = Vec::new();
    for month in months {
        if month.birthdays.is_empty() && month.anniversaries.is_empty() {
            continue;
        }
        items.push(CelebItem::MonthHeader { name: month.month_name.clone(), continued: false });
        if !month.birthdays.is_empty() {
            items.push(CelebItem::SectionHeader("Birthdays".to_string()));
            for entry in &month.birthdays {
                items.push(CelebItem::Entry {
                    display_date: entry.display_date.clone(),
                    name: entry.name.clone(),
                });
            }
        }
        if !month.anniversaries.is_empty() {
            items.push(CelebItem::SectionHeader("Anniversaries".to_string()));
            for entry in &month.anniversaries {
                items.push(CelebItem::Entry {
                    display_date: entry.display_date.clone(),
                    name: entry.name.clone(),
                });
            }
        }
        items.push(CelebItem::MonthGap);
    }
    // Remove trailing MonthGap
    if matches!(items.last(), Some(CelebItem::MonthGap)) {
        items.pop();
    }
    items
}

/// Check if there are any Entry items remaining for the current month
/// starting from the given index (before the next MonthHeader or end).
fn has_remaining_entries(items: &[CelebItem], from_idx: usize) -> bool {
    for item in &items[from_idx..] {
        match item {
            CelebItem::MonthHeader { .. } => return false,
            CelebItem::Entry { .. } => return true,
            _ => {}
        }
    }
    false
}

/// Calculate the minimum height needed to show a useful start of the next content:
/// the current item plus enough room for at least one Entry line.
fn min_useful_height(items: &[CelebItem], from_idx: usize) -> f32 {
    let mut h = 0.0;
    for item in &items[from_idx..] {
        h += item.height();
        if matches!(item, CelebItem::Entry { .. }) {
            return h;
        }
    }
    h
}

/// Fill a column with items, returning how many items were consumed.
/// Ensures we never end a column with just headers and no entries beneath them.
fn fill_column(
    items: &[CelebItem],
    start_idx: usize,
    max_height: f32,
    column: &mut Vec<CelebItem>,
    current_month_name: &mut Option<String>,
) -> usize {
    let mut idx = start_idx;
    let mut h: f32 = column.iter().map(|i| i.height()).sum();

    while idx < items.len() {
        let item = &items[idx];
        let item_h = item.height();

        // If column is non-empty and this item won't fit, stop
        if h + item_h > max_height && !column.is_empty() {
            break;
        }

        // Before adding a MonthHeader or SectionHeader, ensure there's room
        // for it PLUS at least one Entry line. Otherwise, defer to next column.
        if matches!(item, CelebItem::MonthHeader { .. } | CelebItem::SectionHeader(_)) && !column.is_empty() {
            let needed = min_useful_height(items, idx);
            if h + needed > max_height {
                break;
            }
        }

        if let CelebItem::MonthHeader { name, .. } = item {
            *current_month_name = Some(name.clone());
        }
        column.push(item.clone());
        h += item_h;
        idx += 1;
    }

    // Remove trailing MonthGap
    if matches!(column.last(), Some(CelebItem::MonthGap)) {
        column.pop();
    }

    idx
}

/// Paginate celebration items into pages with two flowing columns per page.
/// When a month spans into a new column, a "(Continued)" header is inserted.
fn paginate_celebrations(months: &[CelebrationMonth], image_path: Option<PathBuf>) -> Vec<CelebPage> {
    let title_block_height = CELEB_TITLE_SIZE * 0.4 + 8.0; // title + line + gap
    let col_height_first = PAGE_HEIGHT_MM - CELEB_MARGIN_TOP - CELEB_MARGIN_BOTTOM - title_block_height;
    let col_height_rest = PAGE_HEIGHT_MM - CELEB_MARGIN_TOP - CELEB_MARGIN_BOTTOM;

    let items = flatten_celebrations(months);
    let mut pages: Vec<CelebPage> = Vec::new();
    let mut idx = 0;
    let mut is_first = true;

    while idx < items.len() {
        let mut left: Vec<CelebItem> = Vec::new();
        let mut right: Vec<CelebItem> = Vec::new();

        let max_h = if is_first { col_height_first } else { col_height_rest };

        // Fill left column
        let mut current_month_name: Option<String> = None;
        idx = fill_column(&items, idx, max_h, &mut left, &mut current_month_name);

        // Fill right column if there are more items
        if idx < items.len() {
            // Check if we need a "(Continued)" header — only if the current month
            // still has entries remaining (not just a MonthGap)
            if let Some(ref month_name) = current_month_name {
                if !matches!(items[idx], CelebItem::MonthHeader { .. })
                    && has_remaining_entries(&items, idx)
                {
                    right.push(CelebItem::MonthHeader {
                        name: month_name.clone(),
                        continued: true,
                    });
                }
            }

            // Skip a MonthGap at the boundary (it's just spacing, not content)
            if matches!(items.get(idx), Some(CelebItem::MonthGap)) {
                idx += 1;
            }

            idx = fill_column(&items, idx, max_h, &mut right, &mut current_month_name);
        }

        pages.push(CelebPage {
            left,
            right,
            is_first,
            image_path: if is_first { image_path.clone() } else { None },
        });
        is_first = false;
    }

    // Post-process: for pages after the first, if the left column doesn't start
    // with a MonthHeader, prepend a "(Continued)" header from the previous page.
    for i in 1..pages.len() {
        if !pages[i].left.is_empty() && !matches!(pages[i].left[0], CelebItem::MonthHeader { .. }) {
            let mut last_month = None;
            for item in pages[i - 1].right.iter().rev().chain(pages[i - 1].left.iter().rev()) {
                if let CelebItem::MonthHeader { name, .. } = item {
                    last_month = Some(name.clone());
                    break;
                }
            }
            if let Some(month_name) = last_month {
                pages[i].left.insert(0, CelebItem::MonthHeader {
                    name: month_name,
                    continued: true,
                });
            }
        }
    }

    pages
}

/// Render a single column of celebration items
fn render_celeb_column(
    layer: &PdfLayerReference,
    items: &[CelebItem],
    col_x: f32,
    start_y: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let mut y = start_y;
    for item in items {
        match item {
            CelebItem::MonthHeader { name, continued } => {
                let heading = if *continued {
                    format!("{} (Continued)", name)
                } else {
                    name.clone()
                };
                layer.use_text(&heading, CELEB_MONTH_SIZE, Mm(col_x), Mm(y), font_bold);
                y -= 2.0;
                // Short accent line under month name
                let accent_points = vec![
                    (Point::new(Mm(col_x), Mm(y)), false),
                    (Point::new(Mm(col_x + 20.0), Mm(y)), false),
                ];
                let accent_line = Line {
                    points: accent_points,
                    is_closed: false,
                };
                layer.set_outline_color(Color::Greyscale(Greyscale::new(0.7, None)));
                layer.set_outline_thickness(0.25);
                layer.add_line(accent_line);
                y -= 5.0;
            }
            CelebItem::SectionHeader(label) => {
                layer.use_text(label, CELEB_SECTION_SIZE, Mm(col_x), Mm(y), font_bold);
                y -= CELEB_SECTION_SIZE * 0.4 + CELEB_SECTION_SPACING * 0.5;
            }
            CelebItem::Entry { display_date, name } => {
                let date_text = format!("{}  ", display_date);
                layer.use_text(&date_text, CELEB_TEXT_SIZE, Mm(col_x), Mm(y), font);
                layer.use_text(name, CELEB_TEXT_SIZE, Mm(col_x + 8.0), Mm(y), font);
                y -= CELEB_LINE_HEIGHT;
            }
            CelebItem::MonthGap => {
                y -= CELEB_MONTH_SPACING;
            }
        }
    }
}

/// Render celebration pages with flowing two-column layout
fn render_celebrations(
    _doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    celeb_page: &CelebPage,
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let usable_width = PAGE_WIDTH_MM - 2.0 * CELEB_MARGIN_TOP;
    let col_width = (usable_width - CELEB_COL_GAP) / 2.0;

    let mut y = PAGE_HEIGHT_MM - CELEB_MARGIN_TOP;

    if celeb_page.is_first {
        // Centered title
        let title = "Birthdays & Anniversaries";
        let title_width = measure_helvetica_width(title, CELEB_TITLE_SIZE) * 1.07;
        let title_x = x_offset + CELEB_MARGIN_TOP + (usable_width - title_width) / 2.0;
        layer.use_text(title, CELEB_TITLE_SIZE, Mm(title_x), Mm(y), font_bold);

        // Header image placed just to the right of the title text
        if let Some(ref img_path) = celeb_page.image_path {
            if let Ok(img) = image_crate::open(img_path) {
                use image_crate::GenericImageView;
                let (img_w, img_h) = img.dimensions();
                let img_aspect = img_w as f32 / img_h as f32;
                let img_render_h = CELEB_TITLE_SIZE * 1.8;
                let img_render_w = img_render_h * img_aspect;
                let img_x = title_x + title_width + 6.0;
                let img_y = y - img_render_h + CELEB_TITLE_SIZE * 0.5 + 6.0;
                let dpi = (img_w as f32 * 25.4) / img_render_w;
                let image_xobject = image_to_xobject_with_transparency(&img);
                let image = Image::from(image_xobject);
                image.add_to_layer(
                    layer.clone(),
                    ImageTransform {
                        translate_x: Some(Mm(img_x)),
                        translate_y: Some(Mm(img_y)),
                        dpi: Some(dpi),
                        ..Default::default()
                    },
                );
            }
        }
        y -= CELEB_TITLE_SIZE * 0.4;

        // Decorative line centered under title
        let line_y = y;
        let line_width = title_width * 0.8;
        let line_start = x_offset + CELEB_MARGIN_TOP + (usable_width - line_width) / 2.0;
        let line_end = line_start + line_width;
        let points = vec![
            (Point::new(Mm(line_start), Mm(line_y)), false),
            (Point::new(Mm(line_end), Mm(line_y)), false),
        ];
        let line = Line {
            points,
            is_closed: false,
        };
        layer.set_outline_color(Color::Greyscale(Greyscale::new(0.6, None)));
        layer.set_outline_thickness(0.3);
        layer.add_line(line);
        y -= 8.0;
    }

    // Render left column
    let left_x = x_offset + CELEB_MARGIN_TOP;
    render_celeb_column(layer, &celeb_page.left, left_x, y, font, font_bold);

    // Render right column — starts at same y as left (below title on first page)
    let right_x = x_offset + CELEB_MARGIN_TOP + col_width + CELEB_COL_GAP;
    render_celeb_column(layer, &celeb_page.right, right_x, y, font, font_bold);
}

fn resolve_image_page(path: &Option<String>, photos_dir: &PathBuf) -> ContentPage {
    if let Some(ref filename) = path {
        let full_path = photos_dir.join("directory").join(filename);
        let full_res = get_full_resolution_path(&full_path);
        let image_path = if full_res.exists() {
            full_res
        } else {
            full_path
        };
        if image_path.exists() {
            return ContentPage::FullImage(image_path);
        }
    }
    ContentPage::Blank
}

fn render_content_page(
    doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    page: &ContentPage,
    x_offset: f32,
    is_left_half: bool,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    match page {
        ContentPage::FullImage(path) => {
            render_full_image(doc, layer, path, x_offset, is_left_half);
        }
        ContentPage::CoverImage { image_path, title_line1, title_line2, title_color } => {
            if image_path.exists() {
                render_full_image(doc, layer, image_path, x_offset, is_left_half);
            }
            render_cover_title(layer, title_line1, title_line2, title_color, x_offset, font_bold);
        }
        ContentPage::PhotoGrid { entries, grid_rows } => {
            render_photo_grid(doc, layer, entries, x_offset, font, font_bold, *grid_rows);
        }
        ContentPage::TextCards(cards) => {
            render_text_cards(layer, cards, x_offset, font, font_bold);
        }
        ContentPage::CelebrationPages(celeb_page) => {
            render_celebrations(doc, layer, celeb_page, x_offset, font, font_bold);
        }
        ContentPage::MarkdownContent(text) => {
            render_markdown(layer, text, x_offset, font, font_bold);
        }
        ContentPage::FirstInsidePage { pastor_letter, mission_statement } => {
            render_first_inside_page(layer, pastor_letter.as_deref(), mission_statement.as_deref(), x_offset, font, font_bold);
        }
        ContentPage::StaffPage(entries) => {
            render_staff_page(doc, layer, entries, x_offset, font, font_bold);
        }
        ContentPage::LeadershipPage { entries, contact_left, contact_right } => {
            render_leadership_page(layer, entries, contact_left, contact_right, x_offset, font, font_bold);
        }
        ContentPage::Blank => {}
    }
}

// Markdown rendering constants
const MD_MARGIN_X: f32 = 15.0;
const MD_MARGIN_TOP: f32 = 20.0;
const MD_H1_SIZE: f32 = 20.0;
const MD_H2_SIZE: f32 = 16.0;
const MD_H3_SIZE: f32 = 13.0;
const MD_BODY_SIZE: f32 = 11.0;
const MD_H1_LINE_HEIGHT: f32 = 9.0;
const MD_H2_LINE_HEIGHT: f32 = 7.5;
const MD_H3_LINE_HEIGHT: f32 = 6.5;
const MD_BODY_LINE_HEIGHT: f32 = 5.5;
const MD_PARAGRAPH_SPACING: f32 = 4.0;
const MD_HEADING_SPACING_BEFORE: f32 = 6.0;
const MD_LIST_INDENT: f32 = 5.0;

fn render_markdown(
    layer: &PdfLayerReference,
    markdown: &str,
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let left = x_offset + MD_MARGIN_X;
    let right_bound = x_offset + PAGE_WIDTH_MM - MD_MARGIN_X;
    let max_width = right_bound - left;
    let mut y = PAGE_HEIGHT_MM - MD_MARGIN_TOP;

    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;
    let mut prev_was_blank = false;
    let mut centered = false; // Content after --- is centered
    let mut has_rendered_content = false; // Track if we've rendered any content yet

    while i < lines.len() {
        let line = lines[i].trim_end();

        // Blank line = paragraph break
        if line.is_empty() {
            if !prev_was_blank {
                y -= MD_PARAGRAPH_SPACING;
            }
            prev_was_blank = true;
            i += 1;
            continue;
        }
        prev_was_blank = false;

        // Horizontal rule: --- or more dashes
        if line.trim().len() >= 3 && line.trim().chars().all(|c| c == '-') {
            y -= MD_PARAGRAPH_SPACING;
            let rule_inset = max_width * 0.15;
            let line_start = left + rule_inset;
            let line_end = left + max_width - rule_inset;
            let points = vec![
                (Point::new(Mm(line_start), Mm(y)), false),
                (Point::new(Mm(line_end), Mm(y)), false),
            ];
            let hr = Line {
                points,
                is_closed: false,
            };
            layer.set_outline_color(Color::Greyscale(Greyscale::new(0.4, None)));
            layer.set_outline_thickness(0.75);
            layer.add_line(hr);
            y -= MD_PARAGRAPH_SPACING + 2.0;
            centered = true;
            i += 1;
            continue;
        }

        if let Some(heading) = line.strip_prefix("### ") {
            y -= MD_HEADING_SPACING_BEFORE;
            let wrapped = wrap_text(heading.trim(), MD_H3_SIZE, max_width);
            for wl in &wrapped {
                let text_x = if centered {
                    let tw = wl.len() as f32 * MD_H3_SIZE * 0.19;
                    left + (max_width - tw) / 2.0
                } else { left };
                layer.use_text(wl, MD_H3_SIZE, Mm(text_x), Mm(y), font_bold);
                y -= MD_H3_LINE_HEIGHT;
            }
        } else if let Some(heading) = line.strip_prefix("## ") {
            y -= MD_HEADING_SPACING_BEFORE;
            let wrapped = wrap_text(heading.trim(), MD_H2_SIZE, max_width);
            for wl in &wrapped {
                let text_x = if centered {
                    let tw = wl.len() as f32 * MD_H2_SIZE * 0.19;
                    left + (max_width - tw) / 2.0
                } else { left };
                layer.use_text(wl, MD_H2_SIZE, Mm(text_x), Mm(y), font_bold);
                y -= MD_H2_LINE_HEIGHT;
            }
        } else if let Some(heading) = line.strip_prefix("# ") {
            // Draw a decorative line before any H1 that isn't the first content
            if has_rendered_content {
                y -= MD_PARAGRAPH_SPACING;
                let rule_inset = max_width * 0.15;
                let line_start = left + rule_inset;
                let line_end = left + max_width - rule_inset;
                let points = vec![
                    (Point::new(Mm(line_start), Mm(y)), false),
                    (Point::new(Mm(line_end), Mm(y)), false),
                ];
                let hr = Line {
                    points,
                    is_closed: false,
                };
                layer.set_outline_color(Color::Greyscale(Greyscale::new(0.4, None)));
                layer.set_outline_thickness(0.75);
                layer.add_line(hr);
                y -= MD_PARAGRAPH_SPACING;
                centered = true;
            }
            y -= MD_HEADING_SPACING_BEFORE;
            let wrapped = wrap_text(heading.trim(), MD_H1_SIZE, max_width);
            for wl in &wrapped {
                let text_x = if centered {
                    let tw = wl.len() as f32 * MD_H1_SIZE * 0.19;
                    left + (max_width - tw) / 2.0
                } else { left };
                layer.use_text(wl, MD_H1_SIZE, Mm(text_x), Mm(y), font_bold);
                y -= MD_H1_LINE_HEIGHT;
            }
        } else if line.starts_with("- ") || line.starts_with("* ") {
            let content = &line[2..];
            let bullet_x = left + MD_LIST_INDENT;
            let text_x = bullet_x + 4.0;
            let text_max_width = max_width - MD_LIST_INDENT - 4.0;

            // Render inline markdown (bold/italic) for list items
            let segments = parse_inline_markdown(content);
            let wrapped = wrap_inline_segments(&segments, MD_BODY_SIZE, text_max_width);

            for (li, segs) in wrapped.iter().enumerate() {
                if li == 0 {
                    layer.use_text("\u{2022}", MD_BODY_SIZE, Mm(bullet_x), Mm(y), font);
                }
                render_inline_segments(layer, segs, MD_BODY_SIZE, text_x, y, font, font_bold);
                y -= MD_BODY_LINE_HEIGHT;
            }
        } else {
            // Regular paragraph text with inline markdown
            let segments = parse_inline_markdown(line);
            let wrapped = wrap_inline_segments(&segments, MD_BODY_SIZE, max_width);

            for segs in &wrapped {
                if centered {
                    let total_width = estimate_inline_width(segs, MD_BODY_SIZE);
                    let text_x = left + (max_width - total_width) / 2.0;
                    render_inline_segments(layer, segs, MD_BODY_SIZE, text_x, y, font, font_bold);
                } else {
                    render_inline_segments(layer, segs, MD_BODY_SIZE, left, y, font, font_bold);
                }
                y -= MD_BODY_LINE_HEIGHT;
            }
        }

        has_rendered_content = true;
        i += 1;
    }
}

/// Render the first inside page with pastor letter and mission statement
fn render_first_inside_page(
    layer: &PdfLayerReference,
    pastor_letter: Option<&str>,
    mission_statement: Option<&str>,
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let left = x_offset + MD_MARGIN_X;
    let max_width = PAGE_WIDTH_MM - 2.0 * MD_MARGIN_X;
    let mut y = PAGE_HEIGHT_MM - MD_MARGIN_TOP;

    // "Letter From the Pastor" heading
    if let Some(letter) = pastor_letter {
        if !letter.is_empty() {
            layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
            layer.use_text("Letter From the Pastor", MD_H1_SIZE, Mm(left), Mm(y), font_bold);
            y -= MD_H1_LINE_HEIGHT + MD_PARAGRAPH_SPACING;

            // Render letter body as wrapped paragraphs
            for paragraph in letter.split("\n\n") {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Join single newlines within a paragraph into one block
                let block = trimmed.lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                let wrapped = wrap_text(&block, MD_BODY_SIZE, max_width);
                for wl in &wrapped {
                    layer.use_text(wl, MD_BODY_SIZE, Mm(left), Mm(y), font);
                    y -= MD_BODY_LINE_HEIGHT;
                }
                y -= MD_PARAGRAPH_SPACING;
            }
        }
    }

    // Decorative line — centered in the gap, full width minus margins
    if pastor_letter.map_or(false, |s| !s.is_empty()) && mission_statement.map_or(false, |s| !s.is_empty()) {
        y += MD_PARAGRAPH_SPACING; // reclaim trailing paragraph spacing
        let gap_above = 6.0;
        let gap_below = 12.0;
        y -= gap_above;
        let line_start = left;
        let line_end = left + max_width;
        let points = vec![
            (Point::new(Mm(line_start), Mm(y)), false),
            (Point::new(Mm(line_end), Mm(y)), false),
        ];
        let hr = Line {
            points,
            is_closed: false,
        };
        layer.set_outline_color(Color::Greyscale(Greyscale::new(0.4, None)));
        layer.set_outline_thickness(0.75);
        layer.add_line(hr);
        y -= gap_below;
    }

    // "Mission Statement" heading (centered) + body text (each line centered)
    if let Some(mission) = mission_statement {
        if !mission.is_empty() {
            layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));

            // Centered heading
            let heading = "Mission Statement";
            let heading_width = measure_helvetica_width(heading, MD_H1_SIZE);
            let heading_x = left + (max_width - heading_width) / 2.0;
            layer.use_text(heading, MD_H1_SIZE, Mm(heading_x), Mm(y), font_bold);
            y -= MD_H1_LINE_HEIGHT + MD_PARAGRAPH_SPACING;

            // Each line individually centered using accurate font metrics
            let lines: Vec<&str> = mission.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();

            for line in &lines {
                let line_width = measure_helvetica_width(line, MD_BODY_SIZE);
                let line_x = left + (max_width - line_width) / 2.0;
                layer.use_text(*line, MD_BODY_SIZE, Mm(line_x), Mm(y), font);
                y -= MD_BODY_LINE_HEIGHT;
            }
        }
    }
}

/// Estimate the rendered width of inline segments
fn estimate_inline_width(segments: &[InlineSegment], font_size: f32) -> f32 {
    let char_width = font_size * 0.19;
    segments.iter().map(|seg| {
        let text = match seg {
            InlineSegment::Plain(t) | InlineSegment::Bold(t) | InlineSegment::Italic(t) => t,
        };
        text.len() as f32 * char_width
    }).sum()
}

/// Measure text width using standard Helvetica character widths.
/// Widths are in 1/1000 of a unit; multiply by font_size and convert to mm.
fn measure_helvetica_width(text: &str, font_size: f32) -> f32 {
    let width_units: f32 = text.chars().map(|c| helvetica_char_width(c) as f32).sum();
    // 1 PDF point = 1/72 inch = 25.4/72 mm ≈ 0.3528 mm
    // font_size is in points, char widths are in 1/1000 of font_size
    width_units / 1000.0 * font_size * 25.4 / 72.0
}

/// Standard Helvetica character widths (in 1/1000 units of font size)
fn helvetica_char_width(c: char) -> u32 {
    match c {
        ' ' => 278, '!' => 278, '"' => 355, '#' => 556, '$' => 556,
        '%' => 889, '&' => 667, '\'' => 191, '(' => 333, ')' => 333,
        '*' => 389, '+' => 584, ',' => 278, '-' => 333, '.' => 278,
        '/' => 278, '0' => 556, '1' => 556, '2' => 556, '3' => 556,
        '4' => 556, '5' => 556, '6' => 556, '7' => 556, '8' => 556,
        '9' => 556, ':' => 278, ';' => 278, '<' => 584, '=' => 584,
        '>' => 584, '?' => 556, '@' => 1015, 'A' => 667, 'B' => 667,
        'C' => 722, 'D' => 722, 'E' => 611, 'F' => 556, 'G' => 778,
        'H' => 722, 'I' => 278, 'J' => 500, 'K' => 667, 'L' => 556,
        'M' => 833, 'N' => 722, 'O' => 778, 'P' => 667, 'Q' => 778,
        'R' => 722, 'S' => 667, 'T' => 611, 'U' => 722, 'V' => 667,
        'W' => 944, 'X' => 667, 'Y' => 667, 'Z' => 611, '[' => 278,
        '\\' => 278, ']' => 278, '^' => 469, '_' => 556, '`' => 333,
        'a' => 556, 'b' => 556, 'c' => 500, 'd' => 556, 'e' => 556,
        'f' => 278, 'g' => 556, 'h' => 556, 'i' => 222, 'j' => 222,
        'k' => 500, 'l' => 222, 'm' => 833, 'n' => 556, 'o' => 556,
        'p' => 556, 'q' => 556, 'r' => 333, 's' => 500, 't' => 278,
        'u' => 556, 'v' => 500, 'w' => 722, 'x' => 500, 'y' => 500,
        'z' => 500, '{' => 334, '|' => 260, '}' => 334, '~' => 584,
        _ => 556, // default for unknown chars
    }
}

#[derive(Clone, Debug)]
enum InlineSegment {
    Plain(String),
    Bold(String),
    Italic(String),
}

fn parse_inline_markdown(text: &str) -> Vec<InlineSegment> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Look for **bold**
        if let Some(start) = remaining.find("**") {
            if start > 0 {
                segments.push(InlineSegment::Plain(remaining[..start].to_string()));
            }
            let after_start = &remaining[start + 2..];
            if let Some(end) = after_start.find("**") {
                segments.push(InlineSegment::Bold(after_start[..end].to_string()));
                remaining = &after_start[end + 2..];
            } else {
                segments.push(InlineSegment::Plain(remaining[start..].to_string()));
                break;
            }
        } else if let Some(start) = remaining.find('*') {
            if start > 0 {
                segments.push(InlineSegment::Plain(remaining[..start].to_string()));
            }
            let after_start = &remaining[start + 1..];
            if let Some(end) = after_start.find('*') {
                segments.push(InlineSegment::Italic(after_start[..end].to_string()));
                remaining = &after_start[end + 1..];
            } else {
                segments.push(InlineSegment::Plain(remaining[start..].to_string()));
                break;
            }
        } else {
            segments.push(InlineSegment::Plain(remaining.to_string()));
            break;
        }
    }

    segments
}

fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    // Per-character width estimates for Helvetica, as fraction of font size in mm.
    // font_size is in pt; 1pt = 0.3528mm. Helvetica average char ≈ 0.5em.
    // So 1 char ≈ 0.5 * font_size * 0.3528 mm ≈ font_size * 0.176 mm.
    // Use per-character widths for better accuracy.
    let width: f32 = text
        .chars()
        .map(|c| match c {
            ' ' | 'i' | 'l' | '!' | '|' | '.' | ',' | ':' | ';' | '\'' => 0.10,
            'f' | 'j' | 'r' | 't' | '(' | ')' | '[' | ']' => 0.13,
            'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q'
            | 's' | 'u' | 'v' | 'x' | 'y' | 'z' | '-' => 0.18,
            'w' | 'm' => 0.25,
            'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P'
            | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 0.22,
            'M' | 'W' => 0.27,
            'I' | 'J' | 'L' => 0.15,
            '0'..='9' => 0.18,
            _ => 0.18,
        })
        .sum();
    width * font_size
}

fn wrap_text(text: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        let test = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if estimate_text_width(&test, font_size) > max_width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line = test;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Represents a style type for merging adjacent segments.
#[derive(Clone, Copy, PartialEq)]
enum InlineStyle {
    Plain,
    Bold,
    Italic,
}

fn segment_style(seg: &InlineSegment) -> InlineStyle {
    match seg {
        InlineSegment::Plain(_) => InlineStyle::Plain,
        InlineSegment::Bold(_) => InlineStyle::Bold,
        InlineSegment::Italic(_) => InlineStyle::Italic,
    }
}

fn wrap_inline_segments(
    segments: &[InlineSegment],
    font_size: f32,
    max_width: f32,
) -> Vec<Vec<InlineSegment>> {
    // Flatten segments into words with style info, then re-wrap
    let mut styled_words: Vec<(String, InlineStyle)> = Vec::new();

    for seg in segments {
        let (text, style) = match seg {
            InlineSegment::Plain(t) => (t.as_str(), InlineStyle::Plain),
            InlineSegment::Bold(t) => (t.as_str(), InlineStyle::Bold),
            InlineSegment::Italic(t) => (t.as_str(), InlineStyle::Italic),
        };
        for word in text.split_whitespace() {
            styled_words.push((word.to_string(), style));
        }
    }

    let mut lines: Vec<Vec<(String, InlineStyle)>> = Vec::new();
    let mut current_words: Vec<(String, InlineStyle)> = Vec::new();
    let mut current_width: f32 = 0.0;
    let space_width = estimate_text_width(" ", font_size);

    for (word, style) in &styled_words {
        let word_width = estimate_text_width(word, font_size);
        let needed = if current_words.is_empty() {
            word_width
        } else {
            space_width + word_width
        };

        if current_width + needed > max_width && !current_words.is_empty() {
            lines.push(current_words);
            current_words = Vec::new();
            current_width = 0.0;
        }

        current_words.push((word.clone(), *style));
        current_width += needed;
    }

    if !current_words.is_empty() {
        lines.push(current_words);
    }

    if lines.is_empty() {
        lines.push(Vec::new());
    }

    // Merge adjacent words of the same style into single segments per line
    lines
        .into_iter()
        .map(|words| {
            let mut merged: Vec<InlineSegment> = Vec::new();
            for (word, style) in words {
                let can_merge = merged.last().map_or(false, |last| segment_style(last) == style);
                if can_merge {
                    // Append word (with space) to the last segment
                    match merged.last_mut().unwrap() {
                        InlineSegment::Plain(ref mut t)
                        | InlineSegment::Bold(ref mut t)
                        | InlineSegment::Italic(ref mut t) => {
                            t.push(' ');
                            t.push_str(&word);
                        }
                    }
                } else {
                    // Start a new segment; add leading space if not first
                    let text = if merged.is_empty() {
                        word
                    } else {
                        format!(" {}", word)
                    };
                    merged.push(match style {
                        InlineStyle::Plain => InlineSegment::Plain(text),
                        InlineStyle::Bold => InlineSegment::Bold(text),
                        InlineStyle::Italic => InlineSegment::Italic(text),
                    });
                }
            }
            merged
        })
        .collect()
}

fn render_inline_segments(
    layer: &PdfLayerReference,
    segments: &[InlineSegment],
    font_size: f32,
    start_x: f32,
    y: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let mut x = start_x;

    for seg in segments {
        let (text, used_font) = match seg {
            InlineSegment::Plain(t) => (t.as_str(), font),
            InlineSegment::Bold(t) => (t.as_str(), font_bold),
            // Italic falls back to regular font since we only have regular and bold
            InlineSegment::Italic(t) => (t.as_str(), font),
        };

        if !text.is_empty() {
            layer.use_text(text, font_size, Mm(x), Mm(y), used_font);
            x += estimate_text_width(text, font_size);
        }
    }
}

// Staff page layout constants
const STAFF_MARGIN_X: f32 = 12.0;
const STAFF_MARGIN_TOP: f32 = 20.0;
const STAFF_SECTION_TITLE_SIZE: f32 = 18.0;
const STAFF_NAME_SIZE: f32 = 11.0;
const STAFF_TITLE_SIZE: f32 = 10.0;
const STAFF_PASTOR_PHOTO_W: f32 = 40.0;
const STAFF_PASTOR_PHOTO_H: f32 = 60.0; // 4:6 aspect
const STAFF_ELDER_PHOTO_W: f32 = 30.0;
const STAFF_ELDER_PHOTO_H: f32 = 45.0; // 4:6 aspect

fn render_staff_page(
    _doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    entries: &[StaffEntry],
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let left = x_offset + STAFF_MARGIN_X;
    let mut y = PAGE_HEIGHT_MM - STAFF_MARGIN_TOP;

    let pastor: Vec<&StaffEntry> = entries.iter().filter(|e| e.role == "pastor").collect();
    let elders: Vec<&StaffEntry> = entries.iter().filter(|e| e.role == "elder").collect();
    let staff: Vec<&StaffEntry> = entries.iter().filter(|e| e.role == "staff").collect();

    let usable_width = PAGE_WIDTH_MM - 2.0 * STAFF_MARGIN_X;

    // Pastor takes left 1/3, Staff takes right 2/3 — rendered side by side
    let staff_col_left = left + usable_width * 0.4;
    let section_top_y = y;

    // === PASTOR SECTION (left 1/3) ===
    if !pastor.is_empty() {
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.use_text("Pastor", STAFF_SECTION_TITLE_SIZE, Mm(left), Mm(y), font_bold);
        y -= 8.0;

        for p in &pastor {
            let photo_x = left;
            let photo_y = y - STAFF_ELDER_PHOTO_H;

            if let Some(ref path) = p.photo_path {
                render_staff_photo(layer, path, photo_x, photo_y, STAFF_ELDER_PHOTO_W, STAFF_ELDER_PHOTO_H);
            }

            // Name centered below photo in title case
            let caption = to_title_case(&p.name);
            let caption_width = measure_helvetica_width(&caption, STAFF_NAME_SIZE);
            let caption_x = photo_x + (STAFF_ELDER_PHOTO_W - caption_width) / 2.0;
            layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
            layer.use_text(&caption, STAFF_NAME_SIZE, Mm(caption_x), Mm(photo_y - 5.0), font);

            y = photo_y - 18.0;
        }
    }

    let pastor_bottom_y = y;

    // === STAFF SECTION (right 2/3, same vertical start as Pastor) ===
    if !staff.is_empty() {
        let mut staff_y = section_top_y;
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.use_text("Staff", STAFF_SECTION_TITLE_SIZE, Mm(staff_col_left), Mm(staff_y), font_bold);
        staff_y -= 10.0;

        for s in &staff {
            layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
            let title_upper = s.title.to_uppercase();
            layer.use_text(&title_upper, STAFF_TITLE_SIZE, Mm(staff_col_left), Mm(staff_y), font_bold);
            let title_w = estimate_text_width(&title_upper, STAFF_TITLE_SIZE);
            layer.use_text(&s.name, STAFF_TITLE_SIZE, Mm(staff_col_left + title_w + 6.0), Mm(staff_y), font);
            staff_y -= 7.0;
        }
    }

    // Continue y from whichever section went lower
    y = pastor_bottom_y;

    // === ELDERS SECTION ===
    if !elders.is_empty() {
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.use_text("Elders", STAFF_SECTION_TITLE_SIZE, Mm(left), Mm(y), font_bold);
        y -= 8.0;

        // Layout elders in a row, up to 3 per row
        let max_per_row = 3;
        let col_width = usable_width / max_per_row as f32;

        for row_elders in elders.chunks(max_per_row) {
            let photo_y = y - STAFF_ELDER_PHOTO_H;
            let row_count = row_elders.len();

            for (i, elder) in row_elders.iter().enumerate() {
                // First photo left-justified, last right-justified, middle centered
                let photo_x = if row_count == 1 {
                    left
                } else if i == 0 {
                    left
                } else if i == row_count - 1 {
                    left + usable_width - STAFF_ELDER_PHOTO_W
                } else {
                    left + (usable_width - STAFF_ELDER_PHOTO_W) / 2.0
                };

                if let Some(ref path) = elder.photo_path {
                    render_staff_photo(layer, path, photo_x, photo_y, STAFF_ELDER_PHOTO_W, STAFF_ELDER_PHOTO_H);
                }

                // Name centered below photo in title case
                let name_tc = to_title_case(&elder.name);
                let name_width = measure_helvetica_width(&name_tc, STAFF_NAME_SIZE);
                let name_x = photo_x + (STAFF_ELDER_PHOTO_W - name_width) / 2.0;
                layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
                layer.use_text(&name_tc, STAFF_NAME_SIZE, Mm(name_x), Mm(photo_y - 5.0), font);
            }

            y = photo_y - 14.0;
        }
    }
}

// Leadership page layout constants
const LEAD_MARGIN_X: f32 = 12.0;
const LEAD_MARGIN_TOP: f32 = 22.0;
const LEAD_TITLE_SIZE: f32 = 18.0;
const LEAD_MINISTRY_SIZE: f32 = 14.0;
const LEAD_NAMES_SIZE: f32 = 11.0;
const LEAD_ROW_SPACING: f32 = 10.0;
const LEAD_NAMES_INDENT: f32 = 75.0;

fn render_leadership_page(
    layer: &PdfLayerReference,
    entries: &[LeadershipEntry],
    contact_left: &[String],
    contact_right: &[String],
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let left = x_offset + LEAD_MARGIN_X;
    let usable_width = PAGE_WIDTH_MM - 2.0 * LEAD_MARGIN_X;
    let mut y = PAGE_HEIGHT_MM - LEAD_MARGIN_TOP;

    // Title
    layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
    let lead_heading = "Ministry Team Leadership";
    let lead_heading_width = measure_helvetica_width(lead_heading, LEAD_TITLE_SIZE) * 1.07;
    let lead_heading_x = left + (usable_width - lead_heading_width) / 2.0;
    layer.use_text(lead_heading, LEAD_TITLE_SIZE, Mm(lead_heading_x), Mm(y), font_bold);
    y -= 14.0;

    let gap_between = 4.0;
    let names_x = left + LEAD_NAMES_INDENT;
    let ministry_right_edge = names_x - gap_between;
    let names_max_width = PAGE_WIDTH_MM - LEAD_MARGIN_X * 2.0 - LEAD_NAMES_INDENT;

    for entry in entries {
        let ministry_upper = entry.ministry.to_uppercase();
        let ministry_width = measure_helvetica_width(&ministry_upper, LEAD_MINISTRY_SIZE) * 1.07;
        let ministry_x = ministry_right_edge - ministry_width;
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.use_text(&ministry_upper, LEAD_MINISTRY_SIZE, Mm(ministry_x), Mm(y), font_bold);

        // Names on the same line, wrapping if needed
        // Convert newlines to comma-separated
        let names_text = entry.names
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        let wrapped = wrap_text(&names_text, LEAD_NAMES_SIZE, names_max_width);
        for (i, line) in wrapped.iter().enumerate() {
            let line_y = y - (i as f32 * 4.5);
            layer.use_text(line, LEAD_NAMES_SIZE, Mm(names_x), Mm(line_y), font);
        }

        let extra_lines = if wrapped.len() > 1 { (wrapped.len() - 1) as f32 * 4.5 } else { 0.0 };
        y -= LEAD_ROW_SPACING + extra_lines;
    }

    // Decorative line and contact info
    if !contact_left.is_empty() || !contact_right.is_empty() {
        y -= 4.0;
        let line_start = left;
        let line_end = left + usable_width;
        let points = vec![
            (Point::new(Mm(line_start), Mm(y)), false),
            (Point::new(Mm(line_end), Mm(y)), false),
        ];
        let hr = Line {
            points,
            is_closed: false,
        };
        layer.set_outline_color(Color::Greyscale(Greyscale::new(0.4, None)));
        layer.set_outline_thickness(0.75);
        layer.add_line(hr);
        y -= 10.0;

        // "Contact Info" heading
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.use_text("Contact Info", LEAD_TITLE_SIZE, Mm(left), Mm(y), font_bold);
        y -= 12.0;

        // Two-column contact info
        let contact_size = 11.0;
        let contact_line_height = 5.5;
        let col2_x = left + usable_width / 2.0;
        layer.set_fill_color(Color::Greyscale(Greyscale::new(0.0, None)));

        let max_lines = contact_left.len().max(contact_right.len());
        for i in 0..max_lines {
            if let Some(line) = contact_left.get(i) {
                layer.use_text(line, contact_size, Mm(left), Mm(y), font);
            }
            if let Some(line) = contact_right.get(i) {
                layer.use_text(line, contact_size, Mm(col2_x), Mm(y), font);
            }
            y -= contact_line_height;
        }
    }
}

/// Apply rounded corners to an image by painting corner pixels white.
fn round_image_corners(img: &mut image_crate::RgbImage, radius_fraction: f32) {
    let (w, h) = (img.width(), img.height());
    let radius = (w.min(h) as f32 * radius_fraction) as u32;
    if radius == 0 {
        return;
    }
    let white = image_crate::Rgb([255u8, 255, 255]);
    let r = radius as f32;

    for corner_y in 0..radius {
        for corner_x in 0..radius {
            let dx = r - corner_x as f32 - 0.5;
            let dy = r - corner_y as f32 - 0.5;
            if dx * dx + dy * dy > r * r {
                // Top-left
                img.put_pixel(corner_x, corner_y, white);
                // Top-right
                img.put_pixel(w - 1 - corner_x, corner_y, white);
                // Bottom-left
                img.put_pixel(corner_x, h - 1 - corner_y, white);
                // Bottom-right
                img.put_pixel(w - 1 - corner_x, h - 1 - corner_y, white);
            }
        }
    }
}

/// Draw a rounded rectangle border on the PDF layer.
fn draw_rounded_rect_border(
    layer: &PdfLayerReference,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    line_width: f32,
) {
    layer.set_outline_color(Color::Greyscale(Greyscale::new(0.0, None)));
    layer.set_outline_thickness(line_width);

    // Kappa for cubic Bezier approximation of quarter circle
    let k = radius * 0.5523;

    let points = vec![
        // Start at bottom-left, just above the corner
        (Point::new(Mm(x), Mm(y + radius)), false),
        // Bottom-left corner
        (Point::new(Mm(x), Mm(y + radius - k)), true),
        (Point::new(Mm(x + radius - k), Mm(y)), true),
        (Point::new(Mm(x + radius), Mm(y)), false),
        // Bottom edge to bottom-right corner
        (Point::new(Mm(x + w - radius), Mm(y)), false),
        // Bottom-right corner
        (Point::new(Mm(x + w - radius + k), Mm(y)), true),
        (Point::new(Mm(x + w), Mm(y + radius - k)), true),
        (Point::new(Mm(x + w), Mm(y + radius)), false),
        // Right edge to top-right corner
        (Point::new(Mm(x + w), Mm(y + h - radius)), false),
        // Top-right corner
        (Point::new(Mm(x + w), Mm(y + h - radius + k)), true),
        (Point::new(Mm(x + w - radius + k), Mm(y + h)), true),
        (Point::new(Mm(x + w - radius), Mm(y + h)), false),
        // Top edge to top-left corner
        (Point::new(Mm(x + radius), Mm(y + h)), false),
        // Top-left corner
        (Point::new(Mm(x + radius - k), Mm(y + h)), true),
        (Point::new(Mm(x), Mm(y + h - radius + k)), true),
        (Point::new(Mm(x), Mm(y + h - radius)), false),
    ];

    let line = Line {
        points,
        is_closed: true,
    };

    layer.add_line(line);
}

fn render_staff_photo(
    layer: &PdfLayerReference,
    image_path: &PathBuf,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    if let Ok(img) = image_crate::open(image_path) {
        let (img_width, img_height) = image_crate::GenericImageView::dimensions(&img);

        // Crop to 4:6 aspect ratio (center crop)
        let target_aspect = 4.0 / 6.0;
        let img_aspect = img_width as f32 / img_height as f32;

        let cropped = if (img_aspect - target_aspect).abs() > 0.01 {
            let (crop_w, crop_h) = if img_aspect > target_aspect {
                let cw = (img_height as f32 * target_aspect) as u32;
                (cw, img_height)
            } else {
                let ch = (img_width as f32 / target_aspect) as u32;
                (img_width, ch)
            };
            let cx = (img_width - crop_w) / 2;
            let cy = (img_height - crop_h) / 2;
            img.crop_imm(cx, cy, crop_w, crop_h)
        } else {
            img
        };

        // Round the corners on the image pixels
        let mut rgb = cropped.to_rgb8();
        round_image_corners(&mut rgb, 0.06);

        let (cw, _ch) = (rgb.width(), rgb.height());
        let rounded_img = image_crate::DynamicImage::ImageRgb8(rgb);
        let image_xobject = image_to_xobject(&rounded_img);
        let image = Image::from(image_xobject);
        let dpi = (cw as f32 * 25.4) / w;

        image.add_to_layer(
            layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                dpi: Some(dpi),
                ..Default::default()
            },
        );

        // Draw rounded rectangle border
        let corner_radius = w * 0.06;
        draw_rounded_rect_border(layer, x, y, w, h, corner_radius, 0.5);
    }
}

/// Convert a DynamicImage to an ImageXObject with correct RGB channel ordering.
/// printpdf 0.7's `from_dynamic_image` can swap R and B channels, producing a pink tint.
fn image_to_xobject(img: &image_crate::DynamicImage) -> ImageXObject {
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());
    let data = rgb.into_raw();

    ImageXObject {
        width: Px(w as usize),
        height: Px(h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    }
}

/// Convert a DynamicImage to an ImageXObject, trimming transparent edges
/// and compositing alpha onto a white background.
fn image_to_xobject_with_transparency(img: &image_crate::DynamicImage) -> ImageXObject {
    let rgba = img.to_rgba8();
    let (orig_w, orig_h) = (rgba.width(), rgba.height());

    // Find bounding box of non-transparent pixels (alpha > 10)
    let mut min_x = orig_w;
    let mut min_y = orig_h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    for py in 0..orig_h {
        for px in 0..orig_w {
            if rgba.get_pixel(px, py).0[3] > 10 {
                min_x = min_x.min(px);
                min_y = min_y.min(py);
                max_x = max_x.max(px);
                max_y = max_y.max(py);
            }
        }
    }

    // If no opaque pixels found, use full image
    let (crop_x, crop_y, w, h) = if max_x >= min_x && max_y >= min_y {
        (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    } else {
        (0, 0, orig_w, orig_h)
    };

    let cropped = img.crop_imm(crop_x, crop_y, w, h);
    let cropped_rgba = cropped.to_rgba8();
    let mut rgb_data = Vec::with_capacity((w * h * 3) as usize);

    for pixel in cropped_rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = a as f32 / 255.0;
        let inv = 1.0 - alpha;
        rgb_data.push((r as f32 * alpha + 255.0 * inv) as u8);
        rgb_data.push((g as f32 * alpha + 255.0 * inv) as u8);
        rgb_data.push((b as f32 * alpha + 255.0 * inv) as u8);
    }

    ImageXObject {
        width: Px(w as usize),
        height: Px(h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: rgb_data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    }
}

fn render_full_image(
    _doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    image_path: &PathBuf,
    x_offset: f32,
    _is_left_half: bool,
) {
    if let Ok(img) = image_crate::open(image_path) {
        let (img_width, img_height) = image_crate::GenericImageView::dimensions(&img);
        let img_aspect = img_width as f32 / img_height as f32;

        // 1/4 inch inset on all sides
        let inset = PAGE_BORDER_INSET;
        let avail_width = PAGE_WIDTH_MM - 2.0 * inset;
        let avail_height = PAGE_HEIGHT_MM - 2.0 * inset;
        let avail_aspect = avail_width / avail_height;

        // Scale to fit within available area, preserving aspect ratio (contain)
        let (render_width_mm, render_height_mm) = if img_aspect > avail_aspect {
            (avail_width, avail_width / img_aspect)
        } else {
            (avail_height * img_aspect, avail_height)
        };

        let image_xobject = image_to_xobject(&img);
        let image = Image::from(image_xobject);

        // Center within the inset area
        let offset_x = x_offset + inset + (avail_width - render_width_mm) / 2.0;
        let offset_y = inset + (avail_height - render_height_mm) / 2.0;

        let dpi = (img_width as f32 * 25.4) / render_width_mm;

        image.add_to_layer(
            layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(offset_x)),
                translate_y: Some(Mm(offset_y)),
                dpi: Some(dpi),
                ..Default::default()
            },
        );
    }
}

fn parse_hex_color(hex: &str) -> (f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        (r, g, b)
    } else {
        (1.0, 1.0, 1.0) // default white
    }
}

fn render_cover_title(
    layer: &PdfLayerReference,
    title_line1: &str,
    title_line2: &str,
    title_color: &str,
    x_offset: f32,
    font_bold: &IndirectFontRef,
) {
    const COVER_TITLE_SIZE: f32 = 22.0;
    const COVER_DATE_SIZE: f32 = 16.0;
    const LINE_SPACING: f32 = 10.0;

    // Build the current month and year string
    let now = chrono::Local::now();
    let date_line = now.format("%B %Y").to_string();

    // Collect non-empty lines
    let mut lines: Vec<(&str, f32)> = Vec::new();
    if !title_line1.is_empty() {
        lines.push((title_line1, COVER_TITLE_SIZE));
    }
    if !title_line2.is_empty() {
        lines.push((title_line2, COVER_TITLE_SIZE));
    }
    lines.push((&date_line, COVER_DATE_SIZE));

    if lines.is_empty() {
        return;
    }

    // Position title near the top of the page, below the inset
    let start_y = PAGE_HEIGHT_MM - PAGE_BORDER_INSET - 40.4; // ~1 inch lower

    // Parse and set the title color
    let (r, g, b) = parse_hex_color(title_color);
    layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));

    let page_center = x_offset + PAGE_WIDTH_MM / 2.0;
    let mut y = start_y;

    for (text, size) in &lines {
        let text_width = estimate_text_width(text, *size) * 1.07; // bold correction
        let text_x = page_center - text_width / 2.0;
        layer.use_text(*text, *size, Mm(text_x), Mm(y), font_bold);
        y -= LINE_SPACING;
    }
}

fn render_photo_grid(
    _doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    entries: &[PhotoGridEntry],
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    grid_rows: usize,
) {
    let usable_width = PAGE_WIDTH_MM - 2.0 * GRID_MARGIN_X;
    let usable_height = PAGE_HEIGHT_MM - GRID_MARGIN_TOP - GRID_MARGIN_BOTTOM - FOOTER_Y - FOOTER_FONT_SIZE;

    let cell_width =
        (usable_width - (GRID_COLS as f32 - 1.0) * GRID_COL_SPACING) / GRID_COLS as f32;
    let cell_height =
        (usable_height - (grid_rows as f32 - 1.0) * GRID_ROW_SPACING) / grid_rows as f32;
    // Photo area is the cell minus caption below and padding
    let photo_area_width = cell_width - 2.0 * GRID_CELL_PADDING;
    let photo_area_height =
        cell_height - GRID_CAPTION_HEIGHT - 2.0 * GRID_CELL_PADDING;

    for (i, entry) in entries.iter().enumerate() {
        // Column-major layout: fill down left column first, then right
        let col = i / grid_rows;
        let row = i % grid_rows;

        let cell_x = x_offset + GRID_MARGIN_X + col as f32 * (cell_width + GRID_COL_SPACING);
        let cell_top =
            PAGE_HEIGHT_MM - GRID_MARGIN_TOP - row as f32 * (cell_height + GRID_ROW_SPACING);

        // Track rendered photo position for aligning text
        let mut rendered_img_x = cell_x + GRID_CELL_PADDING;

        // 1. Render photo — fit within cell (contain, not cover)
        if let Ok(img) = image_crate::open(&entry.photo_path) {
            let (img_width, img_height) = image_crate::GenericImageView::dimensions(&img);
            let img_aspect = img_width as f32 / img_height as f32;
            let target_aspect = photo_area_width / photo_area_height;

            let (render_w, render_h) = if img_aspect > target_aspect {
                (photo_area_width, photo_area_width / img_aspect)
            } else {
                (photo_area_height * img_aspect, photo_area_height)
            };

            // Photo area starts at top of cell
            let photo_area_x = cell_x + GRID_CELL_PADDING;
            let photo_area_top = cell_top - GRID_CELL_PADDING;

            let img_x = photo_area_x + (photo_area_width - render_w) / 2.0;
            let img_y = photo_area_top - render_h - (photo_area_height - render_h) / 2.0;

            rendered_img_x = img_x;

            let dpi = (img_width as f32 * 25.4) / render_w;

            let image_xobject = image_to_xobject(&img);
            let image = Image::from(image_xobject);

            image.add_to_layer(
                layer.clone(),
                ImageTransform {
                    translate_x: Some(Mm(img_x)),
                    translate_y: Some(Mm(img_y)),
                    dpi: Some(dpi),
                    ..Default::default()
                },
            );
        }

        // 2. Render caption lines below photo: "LastName, Adults" then children
        let caption_top = cell_top - cell_height + GRID_CAPTION_HEIGHT;
        let caption_size = GRID_NAME_SIZE;
        let line_spacing = 3.5;
        let mut caption_y = caption_top - 2.5;

        // Family last name (bold) followed by adults on the same line
        let name_width = estimate_text_width(&entry.family_name, caption_size) * 1.07;
        layer.use_text(
            &entry.family_name,
            caption_size,
            Mm(rendered_img_x),
            Mm(caption_y),
            font_bold,
        );

        if let Some(ref adults) = entry.directory_adults {
            if !adults.is_empty() {
                let adults_text = format!(", {}", adults);
                layer.use_text(
                    &adults_text,
                    caption_size,
                    Mm(rendered_img_x + name_width),
                    Mm(caption_y),
                    font,
                );
            }
        }
        caption_y -= line_spacing;

        // Children line
        if let Some(ref children) = entry.directory_children {
            if !children.is_empty() {
                layer.use_text(children, caption_size, Mm(rendered_img_x), Mm(caption_y), font);
            }
        }
    }
}

/// Crop an image to a circle: resize to a square, then composite onto a white background.
fn crop_to_circle(img: &image_crate::DynamicImage, size: u32) -> image_crate::DynamicImage {
    use image_crate::GenericImageView;
    let (w, h) = img.dimensions();
    // Center-crop to square first
    let side = w.min(h);
    let crop_x = (w - side) / 2;
    let crop_y = (h - side) / 2;
    let cropped = img.crop_imm(crop_x, crop_y, side, side);
    let resized = cropped.resize_exact(size, size, image_crate::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let center = size as f32 / 2.0;
    let radius_sq = center * center;
    // Composite onto white background as RGB (PDF doesn't support alpha well)
    let mut rgb = image_crate::RgbImage::new(size, size);
    for py in 0..size {
        for px in 0..size {
            let dx = px as f32 - center;
            let dy = py as f32 - center;
            if dx * dx + dy * dy <= radius_sq {
                let p = rgba.get_pixel(px, py);
                rgb.put_pixel(px, py, image_crate::Rgb([p.0[0], p.0[1], p.0[2]]));
            } else {
                rgb.put_pixel(px, py, image_crate::Rgb([255, 255, 255]));
            }
        }
    }
    image_crate::DynamicImage::ImageRgb8(rgb)
}

/// Size of the circular member photo in mm
const MEMBER_PHOTO_SIZE_MM: f32 = 8.0;
/// Pixel size for the circular crop (higher = better quality)
const MEMBER_PHOTO_PIXELS: u32 = 192;

fn render_text_cards(
    layer: &PdfLayerReference,
    columns: &[Vec<FamilyCardData>; 2],
    x_offset: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let usable_width = PAGE_WIDTH_MM - 2.0 * CARD_MARGIN;
    let col_width = (usable_width - CARD_COL_GAP) / CARD_COLS as f32;

    for (col_idx, col_cards) in columns.iter().enumerate() {
        let col_x = x_offset + CARD_MARGIN + col_idx as f32 * (col_width + CARD_COL_GAP);
        let mut y = PAGE_HEIGHT_MM - CARD_MARGIN;

        for card in col_cards.iter() {
            // Name (bold)
            layer.use_text(&card.name, CARD_NAME_SIZE, Mm(col_x), Mm(y), font_bold);
            y -= CARD_NAME_SIZE * 0.6;

            if let Some(ref mailing_name) = card.mailing_name {
                layer.use_text(mailing_name, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                y -= CARD_LINE_HEIGHT;
            }
            if let Some(ref address) = card.address {
                layer.use_text(address, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                y -= CARD_LINE_HEIGHT;
            }
            if let Some(ref csz) = card.city_state_zip {
                layer.use_text(csz, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                y -= CARD_LINE_HEIGHT;
            }
            if card.alt_address.is_some() || card.alt_city_state_zip.is_some() {
                layer.use_text("Alt Address:", CARD_TEXT_SIZE, Mm(col_x), Mm(y), font_bold);
                y -= CARD_LINE_HEIGHT;
                if let Some(ref alt_address) = card.alt_address {
                    layer.use_text(alt_address, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                    y -= CARD_LINE_HEIGHT;
                }
                if let Some(ref alt_csz) = card.alt_city_state_zip {
                    layer.use_text(alt_csz, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                    y -= CARD_LINE_HEIGHT;
                }
            }
            if let Some(ref phone) = card.phone {
                let home_phone = format!("H: {}", phone);
                layer.use_text(&home_phone, CARD_TEXT_SIZE, Mm(col_x), Mm(y), font);
                y -= CARD_LINE_HEIGHT;
            }
            // Check if any member in this family has a photo — if so, indent all names
            let any_member_has_photo = card.members.iter().any(|m| m.photo_path.is_some());
            let indented_x = col_x + MEMBER_PHOTO_SIZE_MM + 1.0;

            // Member info
            for member in &card.members {
                let member_start_y = y;
                let text_x = if any_member_has_photo { indented_x } else { col_x };

                // Render photo if this member has one
                if let Some(ref photo_path) = member.photo_path {
                    if let Ok(img) = image_crate::open(photo_path) {
                        let circle_img = crop_to_circle(&img, MEMBER_PHOTO_PIXELS);
                        let dpi = (MEMBER_PHOTO_PIXELS as f32 * 25.4) / MEMBER_PHOTO_SIZE_MM;
                        let image_xobject = image_to_xobject(&circle_img);
                        let image = Image::from(image_xobject);
                        let photo_y = y - MEMBER_PHOTO_SIZE_MM + CARD_TEXT_SIZE * 0.35;
                        image.add_to_layer(
                            layer.clone(),
                            ImageTransform {
                                translate_x: Some(Mm(col_x)),
                                translate_y: Some(Mm(photo_y)),
                                dpi: Some(dpi),
                                ..Default::default()
                            },
                        );
                    }
                }

                // Name (bold)
                layer.use_text(&member.name, CARD_TEXT_SIZE, Mm(text_x), Mm(y), font_bold);
                y -= CARD_LINE_HEIGHT;

                // Cell phone and email
                if let Some(ref phone) = member.phone {
                    let cell_phone = format!("C: {}", phone);
                    layer.use_text(&cell_phone, CARD_TEXT_SIZE, Mm(text_x), Mm(y), font);
                    y -= CARD_LINE_HEIGHT;
                }
                if let Some(ref email) = member.email {
                    layer.use_text(email, CARD_TEXT_SIZE, Mm(text_x), Mm(y), font);
                    y -= CARD_LINE_HEIGHT;
                }

                // If family has photos, ensure y advances enough for photo height
                if any_member_has_photo {
                    let text_used = member_start_y - y;
                    let photo_height = MEMBER_PHOTO_SIZE_MM + 1.0;
                    if text_used < photo_height {
                        y = member_start_y - photo_height;
                    }
                }
            }

            y -= CARD_SPACING;
        }
    }
}

/// Paginate family cards into pages, with two columns per page.
/// Returns Vec of [left_column, right_column] per page.
fn paginate_cards(cards: &[FamilyCardData]) -> Vec<[Vec<FamilyCardData>; 2]> {
    if cards.is_empty() {
        return Vec::new();
    }

    // Reserve space for footer
    let usable_height = PAGE_HEIGHT_MM - CARD_MARGIN - FOOTER_Y - FOOTER_FONT_SIZE - 2.0;
    let mut pages: Vec<[Vec<FamilyCardData>; 2]> = Vec::new();
    let mut current_cols: [Vec<FamilyCardData>; 2] = [Vec::new(), Vec::new()];
    let mut col_heights = [0.0f32; 2];
    let mut current_col = 0; // Start with left column

    for card in cards {
        let card_h = card.height();

        // Check if card fits in the current column
        if col_heights[current_col] + card_h > usable_height {
            if current_col == 0 {
                // Left column full — move to right column
                current_col = 1;
                // Check if card fits in the right column
                if col_heights[1] + card_h > usable_height {
                    // Right column also full — start a new page
                    if !current_cols[0].is_empty() || !current_cols[1].is_empty() {
                        pages.push(current_cols);
                    }
                    current_cols = [Vec::new(), Vec::new()];
                    col_heights = [0.0, 0.0];
                    current_col = 0;
                }
            } else {
                // Right column full — start a new page
                if !current_cols[0].is_empty() || !current_cols[1].is_empty() {
                    pages.push(current_cols);
                }
                current_cols = [Vec::new(), Vec::new()];
                col_heights = [0.0, 0.0];
                current_col = 0;
            }
        }

        col_heights[current_col] += card_h;
        current_cols[current_col].push(card.clone());
    }

    if !current_cols[0].is_empty() || !current_cols[1].is_empty() {
        pages.push(current_cols);
    }

    pages
}

#[tauri::command]
pub fn get_family_count(db: State<'_, Database>) -> Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM families", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count as usize)
}
