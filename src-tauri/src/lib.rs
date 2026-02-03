mod backup;
mod commands;
mod db;
mod import;
mod models;
mod pdf;
mod photos;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let database = Database::new(app.handle())
                .expect("Failed to initialize database");
            app.manage(database);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Family commands
            commands::get_families,
            commands::get_family,
            commands::create_family,
            commands::update_family,
            commands::delete_family,
            // Member commands
            commands::get_members,
            commands::get_member,
            commands::create_member,
            commands::update_member,
            commands::delete_member,
            // Settings commands
            commands::get_settings,
            commands::save_settings,
            // Import commands
            import::preview_import,
            import::execute_import,
            // Photo commands
            photos::save_family_photo,
            photos::save_member_photo,
            photos::save_church_logo,
            photos::delete_photo,
            photos::get_photo_path,
            photos::get_photo_base64,
            photos::crop_family_photo_to_member,
            // PDF commands
            pdf::generate_pdf,
            pdf::get_family_count,
            // Backup commands
            backup::create_backup,
            backup::preview_restore,
            backup::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
