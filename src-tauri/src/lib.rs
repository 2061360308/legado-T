pub mod commands;
pub mod data;
pub mod domain;
pub mod error;
pub mod platform;
pub mod state;

pub use error::{Error, Result};

use std::sync::Mutex;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&dir).ok();
            let db_path = dir.join("local_reader.db");
            let conn = data::db::open(&db_path.to_string_lossy()).expect("open database");

            if let Ok(res_dir) = app.path().resource_dir() {
                let toc_rule_path = res_dir.join("defaultData").join("txtTocRule.json");
                if let Ok(json) = std::fs::read_to_string(&toc_rule_path) {
                    let _ = data::default_data::load_toc_rules_from_json(&conn, &json);
                    let _ = data::default_data::maybe_upgrade_default_rules(
                        &conn,
                        &json,
                        data::default_data::DEFAULT_DATA_VERSION,
                    );
                }
            }

            app.manage(state::AppState {
                conn: Mutex::new(conn),
                reader_cache: Mutex::new(Default::default()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::version::app_version,
            commands::db::book_list,
            commands::db::book_get,
            commands::db::book_upsert,
            commands::db::book_delete,
            commands::db::chapter_list,
            commands::db::group_list,
            commands::db::txt_toc_rule_list,
            commands::db::kv_get,
            commands::db::kv_set,
            commands::db::replace_rule_list,
            commands::db::dict_rule_list,
            commands::db::http_tts_list,
            commands::bookshelf::bookshelf_list,
            commands::bookshelf::bookshelf_export,
            commands::bookshelf::book_move_to_top,
            commands::bookshelf::book_set_group,
            commands::bookshelf::import_txt,
            commands::bookshelf::import_book,
            commands::bookshelf::re_split_book,
            commands::bookshelf::read_chapter,
            commands::reader::reader_open,
            commands::reader::reader_save_progress,
            commands::reader::reader_effective_rules,
            commands::reader::reader_save_chapter_content,
            commands::rules::txt_toc_rule_save,
            commands::rules::txt_toc_rule_delete,
            commands::rules::txt_toc_rule_reorder,
            commands::rules::txt_toc_rule_restore_default,
            commands::rules::txt_toc_rule_import,
            commands::rules::txt_toc_rule_export,
            commands::rules::replace_rule_save,
            commands::rules::replace_rule_delete,
            commands::rules::replace_rule_import,
            commands::rules::replace_rule_export,
            commands::rules::dict_rule_save,
            commands::rules::dict_rule_delete,
            commands::rules::dict_rule_import,
            commands::rules::dict_rule_export,
            commands::rules::http_tts_save,
            commands::rules::http_tts_delete,
            commands::rules::http_tts_import,
            commands::rules::http_tts_export,
            commands::bookmarks::bookmark_list,
            commands::bookmarks::bookmark_add,
            commands::bookmarks::bookmark_delete,
            commands::backup::backup_export,
            commands::backup::backup_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
