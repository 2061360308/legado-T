use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::read_state::ReaderCache;
use crate::domain::storage::BookStorage;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub reader_cache: Mutex<ReaderCache>,
    pub storage: BookStorage,
    pub explore_cache: Mutex<HashMap<String, Vec<Value>>>,
}
