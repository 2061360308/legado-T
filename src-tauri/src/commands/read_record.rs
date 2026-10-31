use rusqlite::params;
use tauri::State;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::BookRepo;
use crate::domain::model::ReadRecord;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

const DEVICE_ID_KEY: &str = "readRecordDeviceId";

fn device_id(state: &State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(value) = KeyValue::get(&conn, DEVICE_ID_KEY).map_err(|e| e.to_string())? {
        return Ok(value);
    }
    let value = format!("local-{}", chrono::Utc::now().timestamp_millis());
    KeyValue::set(&conn, DEVICE_ID_KEY, &value).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command]
pub fn read_record_add_time(
    state: State<AppState>,
    book_url: String,
    seconds: i64,
) -> CmdResult<()> {
    if seconds <= 0 {
        return Ok(());
    }
    let device = device_id(&state)?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let Some(book) = BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())? else {
        return Ok(());
    };
    conn.execute(
        r#"INSERT INTO readRecord
           (deviceId,bookName,author,readTime,lastRead,durChapterIndex,durChapterPos,durChapterTitle)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
           ON CONFLICT(deviceId,bookName) DO UPDATE SET
             author=excluded.author,
             readTime=readRecord.readTime+excluded.readTime,
             lastRead=excluded.lastRead,
             durChapterIndex=excluded.durChapterIndex,
             durChapterPos=excluded.durChapterPos,
             durChapterTitle=excluded.durChapterTitle"#,
        params![
            device,
            book.name,
            book.author,
            seconds,
            chrono::Utc::now().timestamp(),
            book.dur_chapter_index,
            book.dur_chapter_pos,
            book.dur_chapter_title.unwrap_or_default(),
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn read_record_list(state: State<AppState>) -> CmdResult<Vec<ReadRecord>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            r#"SELECT deviceId,bookName,author,readTime,lastRead,durChapterIndex,durChapterPos,durChapterTitle
               FROM readRecord ORDER BY lastRead DESC"#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ReadRecord {
                device_id: row.get(0)?,
                book_name: row.get(1)?,
                author: row.get(2)?,
                read_time: row.get(3)?,
                last_read: row.get(4)?,
                dur_chapter_index: row.get(5)?,
                dur_chapter_pos: row.get(6)?,
                dur_chapter_title: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_record_delete(state: State<AppState>, device_id: String, book_name: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM readRecord WHERE deviceId=?1 AND bookName=?2",
        params![device_id, book_name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn read_record_clear(state: State<AppState>) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM readRecord", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
