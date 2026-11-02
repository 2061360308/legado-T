use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

#[cfg(mobile)]
use tauri::Manager;

#[cfg(mobile)]
mod mobile;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeAreaInsets {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub cutout_left: i32,
    pub cutout_top: i32,
    pub cutout_right: i32,
    pub cutout_bottom: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub api_level: i32,
    pub abi: String,
    pub screen_width: i32,
    pub screen_height: i32,
    pub density: f64,
    pub safe_area: SafeAreaInsets,
}

impl PlatformInfo {
    #[cfg(not(mobile))]
    fn desktop() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            api_level: 0,
            abi: std::env::consts::ARCH.to_string(),
            screen_width: 0,
            screen_height: 0,
            density: 1.0,
            safe_area: SafeAreaInsets::default(),
        }
    }
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeepScreenOnArgs {
    enabled: bool,
    timeout_seconds: Option<i64>,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrientationArgs {
    mode: String,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CutoutArgs {
    mode: String,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemBarsArgs {
    hide_status_bar: bool,
    hide_navigation_bar: bool,
    light_status_bar: bool,
    light_navigation_bar: bool,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemBarStyleArgs {
    transparent_status_bar: bool,
    transparent_navigation_bar: bool,
    status_bar_color: String,
    navigation_bar_color: String,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TextZoomArgs {
    percent: i32,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LauncherIconArgs {
    icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedImage {
    pub cancelled: bool,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafDirectoryLocation {
    pub cancelled: bool,
    pub tree_uri: String,
    pub document_id: String,
    pub root_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafEntry {
    pub document_id: String,
    pub name: String,
    pub is_dir: bool,
    pub size: i64,
    pub modified: i64,
    pub ext: String,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafPrepareFile {
    pub name: String,
    pub path: String,
    pub size: i64,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafPrepareResult {
    pub session_id: String,
    pub files: Vec<SafPrepareFile>,
    pub skipped: i64,
}

#[tauri::command]
fn platform_info<R: Runtime>(app: AppHandle<R>) -> Result<PlatformInfo, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().platform_info()
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(PlatformInfo::desktop())
    }
}

#[tauri::command]
fn get_safe_area_insets<R: Runtime>(app: AppHandle<R>) -> Result<SafeAreaInsets, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().safe_area_insets()
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(SafeAreaInsets::default())
    }
}

#[tauri::command]
fn set_keep_screen_on<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
    timeout_seconds: Option<i64>,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .set_keep_screen_on(enabled, timeout_seconds)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, enabled, timeout_seconds);
        Ok(())
    }
}

#[tauri::command]
fn set_screen_orientation<R: Runtime>(app: AppHandle<R>, mode: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .set_screen_orientation(mode)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, mode);
        Ok(())
    }
}

#[tauri::command]
fn set_cutout_mode<R: Runtime>(app: AppHandle<R>, mode: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_cutout_mode(mode)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, mode);
        Ok(())
    }
}

#[tauri::command]
fn set_system_bars<R: Runtime>(
    app: AppHandle<R>,
    hide_status_bar: bool,
    hide_navigation_bar: bool,
    light_status_bar: bool,
    light_navigation_bar: bool,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_system_bars(
            hide_status_bar,
            hide_navigation_bar,
            light_status_bar,
            light_navigation_bar,
        )
    }
    #[cfg(not(mobile))]
    {
        let _ = (
            app,
            hide_status_bar,
            hide_navigation_bar,
            light_status_bar,
            light_navigation_bar,
        );
        Ok(())
    }
}

#[tauri::command]
fn set_system_bar_style<R: Runtime>(
    app: AppHandle<R>,
    transparent_status_bar: bool,
    transparent_navigation_bar: bool,
    status_bar_color: String,
    navigation_bar_color: String,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_system_bar_style(
            transparent_status_bar,
            transparent_navigation_bar,
            status_bar_color,
            navigation_bar_color,
        )
    }
    #[cfg(not(mobile))]
    {
        let _ = (
            app,
            transparent_status_bar,
            transparent_navigation_bar,
            status_bar_color,
            navigation_bar_color,
        );
        Ok(())
    }
}

#[tauri::command]
fn set_text_zoom<R: Runtime>(app: AppHandle<R>, percent: i32) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_text_zoom(percent)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, percent);
        Ok(())
    }
}

#[tauri::command]
fn set_launcher_icon<R: Runtime>(app: AppHandle<R>, icon: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_launcher_icon(icon)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, icon);
        Ok(())
    }
}

#[tauri::command]
fn pick_image<R: Runtime>(app: AppHandle<R>) -> Result<PickedImage, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().pick_image()
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(PickedImage {
            cancelled: true,
            path: String::new(),
            name: String::new(),
        })
    }
}

#[tauri::command]
fn set_reader_key_mode<R: Runtime>(
    app: AppHandle<R>,
    active: bool,
    volume_key_enabled: bool,
    key_page_on_long_press: bool,
    disable_return_key: bool,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().set_reader_key_mode(
            active,
            volume_key_enabled,
            key_page_on_long_press,
            disable_return_key,
        )
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, active, volume_key_enabled, key_page_on_long_press, disable_return_key);
        Ok(())
    }
}

#[tauri::command]
fn start_background_audio<R: Runtime>(app: AppHandle<R>, title: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .start_background_audio(title)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, title);
        Ok(())
    }
}

#[tauri::command]
fn stop_background_audio<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().stop_background_audio()
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
fn pick_books_directory<R: Runtime>(app: AppHandle<R>) -> Result<SafDirectoryLocation, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>().pick_books_directory()
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Err("SAF directory import is only available on Android".to_string())
    }
}

#[tauri::command]
fn list_books_directory<R: Runtime>(
    app: AppHandle<R>,
    tree_uri: String,
    document_id: String,
) -> Result<Vec<SafEntry>, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .list_books_directory(tree_uri, document_id)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, tree_uri, document_id);
        Err("SAF directory import is only available on Android".to_string())
    }
}

#[tauri::command]
fn scan_books_directory<R: Runtime>(
    app: AppHandle<R>,
    tree_uri: String,
    document_id: String,
) -> Result<Vec<SafEntry>, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .scan_books_directory(tree_uri, document_id)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, tree_uri, document_id);
        Err("SAF directory import is only available on Android".to_string())
    }
}

#[tauri::command]
fn delete_saf_documents<R: Runtime>(
    app: AppHandle<R>,
    tree_uri: String,
    document_ids: Vec<String>,
) -> Result<i64, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .delete_saf_documents(tree_uri, document_ids)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, tree_uri, document_ids);
        Err("SAF directory import is only available on Android".to_string())
    }
}

#[tauri::command]
fn prepare_books_import<R: Runtime>(
    app: AppHandle<R>,
    tree_uri: String,
    document_ids: Vec<String>,
) -> Result<SafPrepareResult, String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .prepare_books_import(tree_uri, document_ids)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, tree_uri, document_ids);
        Err("SAF directory import is only available on Android".to_string())
    }
}

#[tauri::command]
fn cleanup_saf_import<R: Runtime>(app: AppHandle<R>, session_id: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.state::<mobile::ReaderPlatform<R>>()
            .cleanup_saf_import(session_id)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, session_id);
        Ok(())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("reader-platform")
        .invoke_handler(tauri::generate_handler![
            platform_info,
            get_safe_area_insets,
            set_keep_screen_on,
            set_screen_orientation,
            set_cutout_mode,
            set_system_bars,
            set_system_bar_style,
            set_text_zoom,
            set_launcher_icon,
            pick_image,
            set_reader_key_mode,
            start_background_audio,
            stop_background_audio,
            pick_books_directory,
            list_books_directory,
            scan_books_directory,
            delete_saf_documents,
            prepare_books_import,
            cleanup_saf_import,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            {
                let platform = mobile::init(app, api)?;
                app.manage(platform);
            }
            #[cfg(not(mobile))]
            {
                let _ = (app, api);
            }
            Ok(())
        })
        .build()
}
