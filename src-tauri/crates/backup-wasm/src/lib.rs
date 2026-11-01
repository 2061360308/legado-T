use backup_core::BackupFile;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct TextFile {
    name: String,
    text: String,
}

#[derive(Serialize)]
struct TextFileOut {
    name: String,
    text: String,
}

#[derive(Deserialize)]
struct ConfigPair {
    key: String,
    value: String,
}

#[wasm_bindgen]
pub fn pack_backup_json(files_json: &str) -> Result<Vec<u8>, JsValue> {
    let files: Vec<TextFile> =
        serde_json::from_str(files_json).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let files = files
        .into_iter()
        .map(|file| BackupFile {
            name: file.name,
            bytes: file.text.into_bytes(),
        })
        .collect::<Vec<_>>();
    backup_core::pack(&files).map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn unpack_backup_json(bytes: &[u8]) -> Result<String, JsValue> {
    let files = backup_core::unpack(bytes).map_err(|error| JsValue::from_str(&error))?;
    let files = files
        .into_iter()
        .map(|file| TextFileOut {
            name: file.name,
            text: String::from_utf8_lossy(&file.bytes).to_string(),
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&files).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn write_config_xml_json(
    entries_json: &str,
    password: Option<String>,
) -> Result<String, JsValue> {
    let entries: Vec<ConfigPair> = serde_json::from_str(entries_json)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let entries = entries
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect::<Vec<_>>();
    Ok(backup_core::write_config_xml(&entries, password.as_deref()))
}

#[wasm_bindgen]
pub fn parse_config_xml_json(xml: &str, password: Option<String>) -> Result<String, JsValue> {
    let entries = backup_core::parse_config_xml(xml, password.as_deref())
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "key": entry.key,
                "value": entry.value,
                "encrypted": entry.encrypted,
                "decrypted": entry.decrypted,
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&entries).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn decrypt_backup_text(value: &str, password: String) -> Option<String> {
    backup_core::decrypt_text(value, &password)
}
