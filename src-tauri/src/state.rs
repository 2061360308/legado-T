use std::sync::Mutex;

use rusqlite::Connection;

use crate::domain::read_state::ReaderCache;
use crate::domain::storage::BookStorage;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub reader_cache: Mutex<ReaderCache>,
    pub storage: BookStorage,
}
