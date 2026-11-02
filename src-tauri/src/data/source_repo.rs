use engine_model::{BookSource, RssSource};
use rusqlite::{params, Connection};
use serde_json::Value;

use crate::{Error, Result};

fn now_seconds() -> i64 {
    chrono::Utc::now().timestamp()
}

fn group_value(value: Option<&String>) -> String {
    value.cloned().unwrap_or_default()
}

pub struct BookSourceRepo;

impl BookSourceRepo {
    pub fn upsert(conn: &Connection, source: &BookSource) -> Result<()> {
        let json = serde_json::to_string(source)?;
        conn.execute(
            r#"INSERT INTO book_sources
               (bookSourceUrl, bookSourceName, bookSourceGroup, bookSourceType, enabled,
                enabledExplore, enabledCookieJar, customOrder, weight, lastUpdateTime,
                respondTime, json, updatedAt)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
               ON CONFLICT(bookSourceUrl) DO UPDATE SET
                 bookSourceName=excluded.bookSourceName,
                 bookSourceGroup=excluded.bookSourceGroup,
                 bookSourceType=excluded.bookSourceType,
                 enabled=excluded.enabled,
                 enabledExplore=excluded.enabledExplore,
                 enabledCookieJar=excluded.enabledCookieJar,
                 customOrder=excluded.customOrder,
                 weight=excluded.weight,
                 lastUpdateTime=excluded.lastUpdateTime,
                 respondTime=excluded.respondTime,
                 json=excluded.json,
                 updatedAt=excluded.updatedAt"#,
            params![
                source.book_source_url,
                source.book_source_name,
                group_value(source.book_source_group.as_ref()),
                source.book_source_type,
                source.enabled as i64,
                source.enabled_explore as i64,
                source.base.enabled_cookie_jar.unwrap_or(false) as i64,
                source.custom_order,
                source.weight,
                source.last_update_time,
                source.respond_time,
                json,
                now_seconds(),
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<BookSource>> {
        let mut stmt = conn.prepare(
            "SELECT json FROM book_sources ORDER BY customOrder ASC, bookSourceName ASC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let raw = row?;
            if let Ok(source) = serde_json::from_str::<BookSource>(&raw) {
                out.push(source);
            }
        }
        Ok(out)
    }

    pub fn get(conn: &Connection, url: &str) -> Result<Option<BookSource>> {
        let mut stmt = conn.prepare("SELECT json FROM book_sources WHERE bookSourceUrl = ?1")?;
        let mut rows = stmt.query_map([url], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(row) => Ok(Some(serde_json::from_str::<BookSource>(&row?)?)),
            None => Ok(None),
        }
    }

    pub fn delete(conn: &Connection, url: &str) -> Result<()> {
        conn.execute("DELETE FROM book_sources WHERE bookSourceUrl = ?1", [url])?;
        Ok(())
    }

    pub fn set_enabled(conn: &Connection, url: &str, enabled: bool) -> Result<()> {
        let Some(mut source) = Self::get(conn, url)? else {
            return Err(Error::NotFound(url.to_string()));
        };
        source.enabled = enabled;
        Self::upsert(conn, &source)
    }

    pub fn set_explore_enabled(conn: &Connection, url: &str, enabled: bool) -> Result<()> {
        let Some(mut source) = Self::get(conn, url)? else {
            return Err(Error::NotFound(url.to_string()));
        };
        source.enabled_explore = enabled;
        Self::upsert(conn, &source)
    }

    pub fn groups(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT bookSourceGroup FROM book_sources WHERE bookSourceGroup <> '' ORDER BY bookSourceGroup",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

pub struct RssSourceRepo;

impl RssSourceRepo {
    pub fn upsert(conn: &Connection, source: &RssSource) -> Result<()> {
        let json = serde_json::to_string(source)?;
        conn.execute(
            r#"INSERT INTO rss_sources
               (sourceUrl, sourceName, sourceGroup, enabled, customOrder, lastUpdateTime, json, updatedAt)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
               ON CONFLICT(sourceUrl) DO UPDATE SET
                 sourceName=excluded.sourceName,
                 sourceGroup=excluded.sourceGroup,
                 enabled=excluded.enabled,
                 customOrder=excluded.customOrder,
                 lastUpdateTime=excluded.lastUpdateTime,
                 json=excluded.json,
                 updatedAt=excluded.updatedAt"#,
            params![
                source.source_url,
                source.source_name,
                group_value(source.source_group.as_ref()),
                source.enabled as i64,
                source.custom_order,
                source.last_update_time,
                json,
                now_seconds(),
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<RssSource>> {
        let mut stmt =
            conn.prepare("SELECT json FROM rss_sources ORDER BY customOrder ASC, sourceName ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let raw = row?;
            if let Ok(source) = serde_json::from_str::<RssSource>(&raw) {
                out.push(source);
            }
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, url: &str) -> Result<()> {
        conn.execute("DELETE FROM rss_sources WHERE sourceUrl = ?1", [url])?;
        Ok(())
    }
}

pub fn save_search_history(conn: &Connection, keyword: &str) -> Result<()> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO search_history (keyword,lastUsed) VALUES (?1,?2)
         ON CONFLICT(keyword) DO UPDATE SET lastUsed=excluded.lastUsed",
        params![keyword, now_seconds()],
    )?;
    Ok(())
}

pub fn list_search_history(conn: &Connection, limit: usize) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT keyword FROM search_history ORDER BY lastUsed DESC LIMIT ?1")?;
    let rows = stmt.query_map([limit as i64], |row| row.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn delete_search_history(conn: &Connection, keyword: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM search_history WHERE keyword = ?1",
        [keyword.trim()],
    )?;
    Ok(())
}

pub fn clear_search_history(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM search_history", [])?;
    Ok(())
}

pub fn save_search_books(conn: &Connection, books: &[Value]) -> Result<()> {
    let now = now_seconds();
    for book in books {
        let book_url = book.get("bookUrl").and_then(Value::as_str).unwrap_or("");
        let source_url = book
            .get("bookSourceUrl")
            .and_then(Value::as_str)
            .unwrap_or("");
        if book_url.is_empty() || source_url.is_empty() || book.get("error").is_some() {
            continue;
        }
        conn.execute(
            "INSERT INTO search_books (bookUrl,bookSourceUrl,json,updatedAt)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT(bookUrl,bookSourceUrl) DO UPDATE SET
               json=excluded.json,
               updatedAt=excluded.updatedAt",
            params![book_url, source_url, serde_json::to_string(book)?, now],
        )?;
    }
    conn.execute(
        "DELETE FROM search_books WHERE updatedAt < ?1",
        [now - 7 * 24 * 60 * 60],
    )?;
    Ok(())
}

pub fn get_search_book(
    conn: &Connection,
    book_url: &str,
    source_url: &str,
) -> Result<Option<Value>> {
    let mut stmt = conn.prepare(
        "SELECT json FROM search_books
         WHERE bookUrl = ?1 AND bookSourceUrl = ?2
         ORDER BY updatedAt DESC LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![book_url, source_url], |row| {
        row.get::<_, String>(0)
    })?;
    match rows.next() {
        Some(row) => Ok(Some(serde_json::from_str(&row?)?)),
        None => Ok(None),
    }
}

pub fn set_source_variable(
    conn: &Connection,
    source_url: &str,
    key: &str,
    value: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO source_variables (bookSourceUrl,key,value) VALUES (?1,?2,?3)
         ON CONFLICT(bookSourceUrl,key) DO UPDATE SET value=excluded.value",
        params![source_url, key, value],
    )?;
    Ok(())
}

pub fn delete_source_variable(
    conn: &Connection,
    source_url: &str,
    key: &str,
) -> Result<()> {
    conn.execute(
        "DELETE FROM source_variables WHERE bookSourceUrl = ?1 AND key = ?2",
        params![source_url, key],
    )?;
    Ok(())
}

pub fn get_source_variables(conn: &Connection, source_url: &str) -> Result<Value> {
    let mut stmt =
        conn.prepare("SELECT key,value FROM source_variables WHERE bookSourceUrl = ?1")?;
    let rows = stmt.query_map([source_url], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = serde_json::Map::new();
    for row in rows {
        let (key, value) = row?;
        map.insert(key, Value::String(value));
    }
    Ok(Value::Object(map))
}
