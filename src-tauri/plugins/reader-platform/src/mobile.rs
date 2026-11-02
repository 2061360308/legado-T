use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{
    CutoutArgs, KeepScreenOnArgs, LauncherIconArgs, OrientationArgs, PickedImage, PlatformInfo,
    SafDirectoryLocation, SafEntry, SafPrepareResult, SafeAreaInsets, SystemBarStyleArgs,
    SystemBarsArgs, TextZoomArgs,
};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.localreader.platform";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_reader_platform);

pub struct ReaderPlatform<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for ReaderPlatform<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<ReaderPlatform<R>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ReaderPlatformPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_reader_platform)?;
    Ok(ReaderPlatform(handle))
}

impl<R: Runtime> ReaderPlatform<R> {
    pub fn platform_info(&self) -> Result<PlatformInfo, String> {
        self.0
            .run_mobile_plugin("platformInfo", ())
            .map_err(|error| error.to_string())
    }

    pub fn safe_area_insets(&self) -> Result<SafeAreaInsets, String> {
        self.0
            .run_mobile_plugin("getSafeAreaInsets", ())
            .map_err(|error| error.to_string())
    }

    pub fn set_keep_screen_on(
        &self,
        enabled: bool,
        timeout_seconds: Option<i64>,
    ) -> Result<(), String> {
        self.0
            .run_mobile_plugin(
                "setKeepScreenOn",
                KeepScreenOnArgs {
                    enabled,
                    timeout_seconds,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn set_screen_orientation(&self, mode: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin("setScreenOrientation", OrientationArgs { mode })
            .map_err(|error| error.to_string())
    }

    pub fn set_cutout_mode(&self, mode: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin("setCutoutMode", CutoutArgs { mode })
            .map_err(|error| error.to_string())
    }

    pub fn set_system_bars(
        &self,
        hide_status_bar: bool,
        hide_navigation_bar: bool,
        light_status_bar: bool,
        light_navigation_bar: bool,
    ) -> Result<(), String> {
        self.0
            .run_mobile_plugin(
                "setSystemBars",
                SystemBarsArgs {
                    hide_status_bar,
                    hide_navigation_bar,
                    light_status_bar,
                    light_navigation_bar,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn set_system_bar_style(
        &self,
        transparent_status_bar: bool,
        transparent_navigation_bar: bool,
        status_bar_color: String,
        navigation_bar_color: String,
    ) -> Result<(), String> {
        self.0
            .run_mobile_plugin(
                "setSystemBarStyle",
                SystemBarStyleArgs {
                    transparent_status_bar,
                    transparent_navigation_bar,
                    status_bar_color,
                    navigation_bar_color,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn set_text_zoom(&self, percent: i32) -> Result<(), String> {
        self.0
            .run_mobile_plugin("setTextZoom", TextZoomArgs { percent })
            .map_err(|error| error.to_string())
    }

    pub fn set_launcher_icon(&self, icon: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin("setLauncherIcon", LauncherIconArgs { icon })
            .map_err(|error| error.to_string())
    }

    pub fn pick_image(&self) -> Result<PickedImage, String> {
        self.0
            .run_mobile_plugin("pickImage", ())
            .map_err(|error| error.to_string())
    }

    pub fn set_reader_key_mode(
        &self,
        active: bool,
        volume_key_enabled: bool,
        key_page_on_long_press: bool,
        disable_return_key: bool,
    ) -> Result<(), String> {
        self.0
            .run_mobile_plugin(
                "setReaderKeyMode",
                ReaderKeyModeArgs {
                    active,
                    volume_key_enabled,
                    key_page_on_long_press,
                    disable_return_key,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn pick_books_directory(&self) -> Result<SafDirectoryLocation, String> {
        self.0
            .run_mobile_plugin("pickBooksDirectory", ())
            .map_err(|error| error.to_string())
    }

    pub fn list_books_directory(
        &self,
        tree_uri: String,
        document_id: String,
    ) -> Result<Vec<SafEntry>, String> {
        self.0
            .run_mobile_plugin(
                "listBooksDirectory",
                SafListArgs {
                    tree_uri,
                    document_id,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn scan_books_directory(
        &self,
        tree_uri: String,
        document_id: String,
    ) -> Result<Vec<SafEntry>, String> {
        self.0
            .run_mobile_plugin(
                "scanBooksDirectory",
                SafListArgs {
                    tree_uri,
                    document_id,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn delete_saf_documents(
        &self,
        tree_uri: String,
        document_ids: Vec<String>,
    ) -> Result<i64, String> {
        self.0
            .run_mobile_plugin(
                "deleteSafDocuments",
                SafDeleteArgs {
                    tree_uri,
                    document_ids,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn prepare_books_import(
        &self,
        tree_uri: String,
        document_ids: Vec<String>,
    ) -> Result<SafPrepareResult, String> {
        self.0
            .run_mobile_plugin(
                "prepareBooksImport",
                SafPrepareArgs {
                    tree_uri,
                    document_ids,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub fn cleanup_saf_import(&self, session_id: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin("cleanupSafImport", SafCleanupArgs { session_id })
            .map_err(|error| error.to_string())
    }
}

#[cfg(mobile)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReaderKeyModeArgs {
    active: bool,
    volume_key_enabled: bool,
    key_page_on_long_press: bool,
    disable_return_key: bool,
}

#[cfg(mobile)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SafCleanupArgs {
    session_id: String,
}

#[cfg(mobile)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SafListArgs {
    tree_uri: String,
    document_id: String,
}

#[cfg(mobile)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SafPrepareArgs {
    tree_uri: String,
    document_ids: Vec<String>,
}

#[cfg(mobile)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SafDeleteArgs {
    tree_uri: String,
    document_ids: Vec<String>,
}
