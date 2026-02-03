use crate::db::Database;
use crate::models::{Family, FamilyWithMembers, Member, PdfOptions};
use printpdf::*;
use rusqlite::params;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use tauri::{AppHandle, State};

const LETTER_WIDTH_MM: f32 = 215.9;
const LETTER_HEIGHT_MM: f32 = 279.4;
const A4_WIDTH_MM: f32 = 210.0;
const A4_HEIGHT_MM: f32 = 297.0;
const MARGIN: f32 = 20.0;
const LINE_HEIGHT: f32 = 5.0;
const TITLE_SIZE: f32 = 24.0;
const HEADING_SIZE: f32 = 14.0;
const TEXT_SIZE: f32 = 10.0;
const PHOTO_WIDTH_MM: f32 = 40.0;
const PHOTO_HEIGHT_MM: f32 = 50.0;

/// Get the full resolution path for a photo (used for print quality PDFs)
fn get_full_resolution_path(image_path: &PathBuf) -> PathBuf {
    let stem = image_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = image_path.parent().unwrap_or(std::path::Path::new(""));
    parent.join(format!("{}_full.jpg", stem))
}

#[tauri::command]
pub fn generate_pdf(
    app_handle: AppHandle,
    db: State<'_, Database>,
    options: PdfOptions,
    output_path: String,
) -> Result<String, String> {
    let photos_dir = Database::get_photos_dir(&app_handle);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut families_with_members: Vec<FamilyWithMembers> = Vec::new();

    let mut family_stmt = conn
        .prepare(
            "SELECT id, family_id, name, mailing_name, address, city, state, zip, phone, email,
                    photo_path, notes, created_at, updated_at
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
                email: row.get(9)?,
                photo_path: row.get(10)?,
                notes: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

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

        families_with_members.push(FamilyWithMembers { family, members });
    }

    // conn is dropped automatically after this point

    let (page_width, page_height) = if options.page_size == "a4" {
        (Mm(A4_WIDTH_MM), Mm(A4_HEIGHT_MM))
    } else {
        (Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM))
    };

    let (doc, page1, layer1) = PdfDocument::new(
        &options.church_name,
        page_width,
        page_height,
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| format!("Font error: {:?}", e))?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| format!("Font error: {:?}", e))?;

    let mut current_page = page1;
    let mut current_layer = layer1;
    let mut y_position = page_height.0 - MARGIN;

    if options.include_cover {
        {
            let layer = doc.get_page(current_page).get_layer(current_layer);
            layer.use_text(
                &options.church_name,
                TITLE_SIZE,
                Mm(MARGIN),
                Mm(y_position),
                &font_bold,
            );
            y_position -= 15.0;

            layer.use_text(
                "Photo Directory",
                HEADING_SIZE,
                Mm(MARGIN),
                Mm(y_position),
                &font,
            );
        }

        let (new_page, new_layer) = doc.add_page(page_width, page_height, "Directory");
        current_page = new_page;
        current_layer = new_layer;
        y_position = page_height.0 - MARGIN;
    }

    if options.include_toc && !families_with_members.is_empty() {
        {
            let layer = doc.get_page(current_page).get_layer(current_layer);
            layer.use_text(
                "Table of Contents",
                HEADING_SIZE,
                Mm(MARGIN),
                Mm(y_position),
                &font_bold,
            );
        }
        y_position -= LINE_HEIGHT * 3.0;

        for family in &families_with_members {
            if y_position < MARGIN + LINE_HEIGHT {
                let (new_page, new_layer) = doc.add_page(page_width, page_height, "TOC");
                current_page = new_page;
                current_layer = new_layer;
                y_position = page_height.0 - MARGIN;
            }

            {
                let layer = doc.get_page(current_page).get_layer(current_layer);
                layer.use_text(
                    &family.family.name,
                    TEXT_SIZE,
                    Mm(MARGIN),
                    Mm(y_position),
                    &font,
                );
            }
            y_position -= LINE_HEIGHT;
        }

        let (new_page, new_layer) = doc.add_page(page_width, page_height, "Directory");
        current_page = new_page;
        current_layer = new_layer;
        y_position = page_height.0 - MARGIN;
    }

    if options.layout == "grid" {
        let col_width = (page_width.0 - MARGIN * 3.0) / 2.0;
        let mut col = 0;

        for family in &families_with_members {
            let entry_height = calculate_entry_height(family, &options);

            if y_position - entry_height < MARGIN {
                if col == 0 {
                    col = 1;
                    y_position = page_height.0 - MARGIN;
                } else {
                    let (new_page, new_layer) = doc.add_page(page_width, page_height, "Directory");
                    current_page = new_page;
                    current_layer = new_layer;
                    y_position = page_height.0 - MARGIN;
                    col = 0;
                }
            }

            let x_offset = MARGIN + (col as f32 * (col_width + MARGIN));
            {
                let layer = doc.get_page(current_page).get_layer(current_layer);
                y_position = render_family_entry(
                    &doc,
                    &layer,
                    &font,
                    &font_bold,
                    family,
                    &options,
                    x_offset,
                    y_position,
                    &photos_dir,
                );
            }
            y_position -= LINE_HEIGHT * 2.0;
        }
    } else {
        for family in &families_with_members {
            let entry_height = calculate_entry_height(family, &options);

            if y_position - entry_height < MARGIN {
                let (new_page, new_layer) = doc.add_page(page_width, page_height, "Directory");
                current_page = new_page;
                current_layer = new_layer;
                y_position = page_height.0 - MARGIN;
            }

            {
                let layer = doc.get_page(current_page).get_layer(current_layer);
                y_position = render_family_entry(
                    &doc,
                    &layer,
                    &font,
                    &font_bold,
                    family,
                    &options,
                    MARGIN,
                    y_position,
                    &photos_dir,
                );
            }
            y_position -= LINE_HEIGHT * 2.0;
        }
    }

    let file = File::create(&output_path).map_err(|e| e.to_string())?;
    let mut buf_writer = BufWriter::new(file);
    doc.save(&mut buf_writer).map_err(|e| format!("Save error: {:?}", e))?;

    Ok(output_path)
}

fn calculate_entry_height(family: &FamilyWithMembers, options: &PdfOptions) -> f32 {
    let mut height = LINE_HEIGHT * 2.0;

    if options.include_address {
        if family.family.mailing_name.is_some() {
            height += LINE_HEIGHT;
        }
        if family.family.address.is_some() {
            height += LINE_HEIGHT * 2.0;
        }
    }

    if options.include_contact_info {
        if family.family.phone.is_some() {
            height += LINE_HEIGHT;
        }
        if family.family.email.is_some() {
            height += LINE_HEIGHT;
        }
    }

    height += LINE_HEIGHT * family.members.len() as f32;

    if options.include_photos {
        height = height.max(PHOTO_HEIGHT_MM + LINE_HEIGHT);
    }

    height
}

fn render_family_entry(
    _doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    family: &FamilyWithMembers,
    options: &PdfOptions,
    x: f32,
    mut y: f32,
    photos_dir: &PathBuf,
) -> f32 {
    let text_x = if options.include_photos {
        x + PHOTO_WIDTH_MM + 5.0
    } else {
        x
    };

    // Render photo if enabled and available
    if options.include_photos {
        if let Some(ref photo_path) = family.family.photo_path {
            // Prefer full resolution version for print quality
            let base_path = photos_dir.join("families").join(photo_path);
            let full_res_path = get_full_resolution_path(&base_path);
            let photo_file = if full_res_path.exists() {
                full_res_path
            } else {
                base_path
            };

            if photo_file.exists() {
                // Use printpdf's re-exported image crate for compatibility
                if let Ok(img) = image_crate::open(&photo_file) {
                    let (img_width, img_height) = image_crate::GenericImageView::dimensions(&img);

                    // Calculate aspect ratio and dimensions to fit within PHOTO_WIDTH x PHOTO_HEIGHT
                    let aspect = img_width as f32 / img_height as f32;
                    let (render_width_mm, render_height_mm) = if aspect > PHOTO_WIDTH_MM / PHOTO_HEIGHT_MM {
                        // Image is wider than target area - constrain by width
                        (PHOTO_WIDTH_MM, PHOTO_WIDTH_MM / aspect)
                    } else {
                        // Image is taller than target area - constrain by height
                        (PHOTO_HEIGHT_MM * aspect, PHOTO_HEIGHT_MM)
                    };

                    // Create image for PDF
                    let image_xobject = ImageXObject::from_dynamic_image(&img);
                    let image = Image::from(image_xobject);

                    // Position: x is left edge, y is bottom edge of image in PDF coordinates
                    let image_y = y - render_height_mm;

                    // Calculate DPI needed to render image at desired size
                    // DPI = pixels / inches, and we want render_width_mm
                    // render_width_inches = render_width_mm / 25.4
                    // dpi = img_width / render_width_inches = img_width * 25.4 / render_width_mm
                    let dpi = (img_width as f32 * 25.4) / render_width_mm;

                    image.add_to_layer(
                        layer.clone(),
                        ImageTransform {
                            translate_x: Some(Mm(x)),
                            translate_y: Some(Mm(image_y)),
                            dpi: Some(dpi),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    layer.use_text(
        &family.family.name,
        HEADING_SIZE,
        Mm(text_x),
        Mm(y),
        font_bold,
    );
    y -= LINE_HEIGHT * 1.5;

    if options.include_address {
        if let Some(ref mailing_name) = family.family.mailing_name {
            layer.use_text(mailing_name, TEXT_SIZE, Mm(text_x), Mm(y), font);
            y -= LINE_HEIGHT;
        }
        if let Some(ref address) = family.family.address {
            layer.use_text(address, TEXT_SIZE, Mm(text_x), Mm(y), font);
            y -= LINE_HEIGHT;

            let mut city_state_zip = Vec::new();
            if let Some(ref city) = family.family.city {
                city_state_zip.push(city.clone());
            }
            if let Some(ref state) = family.family.state {
                if !city_state_zip.is_empty() {
                    let last = city_state_zip.pop().unwrap();
                    city_state_zip.push(format!("{}, {}", last, state));
                } else {
                    city_state_zip.push(state.clone());
                }
            }
            if let Some(ref zip) = family.family.zip {
                city_state_zip.push(zip.clone());
            }
            if !city_state_zip.is_empty() {
                layer.use_text(&city_state_zip.join(" "), TEXT_SIZE, Mm(text_x), Mm(y), font);
                y -= LINE_HEIGHT;
            }
        }
    }

    if options.include_contact_info {
        if let Some(ref phone) = family.family.phone {
            layer.use_text(phone, TEXT_SIZE, Mm(text_x), Mm(y), font);
            y -= LINE_HEIGHT;
        }
        if let Some(ref email) = family.family.email {
            layer.use_text(email, TEXT_SIZE, Mm(text_x), Mm(y), font);
            y -= LINE_HEIGHT;
        }
    }

    y -= LINE_HEIGHT * 0.5;

    for member in &family.members {
        let member_text = if let Some(ref role) = member.role {
            format!("{} {} ({})", member.first_name, member.last_name, role)
        } else {
            format!("{} {}", member.first_name, member.last_name)
        };
        layer.use_text(&member_text, TEXT_SIZE, Mm(text_x + 5.0), Mm(y), font);
        y -= LINE_HEIGHT;
    }

    // Ensure we move down at least the photo height if photos are included
    if options.include_photos {
        let start_y = y + LINE_HEIGHT * (family.members.len() as f32 + 3.0);
        let photo_bottom = start_y - PHOTO_HEIGHT_MM;
        if y > photo_bottom {
            y = photo_bottom;
        }
    }

    y
}

#[tauri::command]
pub fn get_family_count(db: State<'_, Database>) -> Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM families", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count as usize)
}
