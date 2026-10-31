const COMMANDS: &[&str] = &[
    "platform_info",
    "get_safe_area_insets",
    "set_keep_screen_on",
    "set_screen_orientation",
    "set_cutout_mode",
    "set_system_bars",
    "set_system_bar_style",
    "set_text_zoom",
    "set_launcher_icon",
    "pick_image",
    "pick_font_file",
    "set_reader_key_mode",
    "pick_books_directory",
    "list_books_directory",
    "prepare_books_import",
    "cleanup_saf_import",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .try_build()
        .unwrap();
}
