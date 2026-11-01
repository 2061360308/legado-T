//! 全局 KV 配置存储（T3.4）。
//!
//! 存储全局设置键（`themeMode`/`ttsEngine`/`bookshelfSort`/`password` 等），
//! 并预留 legado `config.xml` 白名单映射（未知键忽略）。

use rusqlite::Connection;

use crate::Result;

/// 全局 KV 存储。
pub struct KeyValue;

impl KeyValue {
    pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare("SELECT value FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |r| r.get::<_, String>(0))?;
        match rows.next() {
            Some(v) => Ok(Some(v?)),
            None => Ok(None),
        }
    }

    pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    pub fn remove(conn: &Connection, key: &str) -> Result<()> {
        conn.execute("DELETE FROM kv WHERE key = ?1", [key])?;
        Ok(())
    }

    pub fn all(conn: &Connection) -> Result<Vec<(String, String)>> {
        let mut stmt = conn.prepare("SELECT key, value FROM kv")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 通用缓存读写（caches 表，含替换去重缓存）。
    pub fn cache_get(conn: &Connection, key: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare("SELECT value FROM caches WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |r| r.get::<_, String>(0))?;
        match rows.next() {
            Some(v) => Ok(Some(v?)),
            None => Ok(None),
        }
    }

    pub fn cache_set(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO caches (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }
}

/// `config.xml` 白名单键（未知键忽略，spec 4.5.2 / design 2.3.2）。
pub const CONFIG_XML_WHITELIST: [&str; 17] = [
    "themeMode",
    "pageAnim",
    "preDownloadNum",
    "useZhLayout",
    "textFullJustify",
    "chineseConverterType",
    "bookshelfSort",
    "password",
    "remoteServers",
    "remoteServerId",
    "remoteProxyUrl",
    "webdavUrl",
    "webdavDir",
    "webdavAccount",
    "webdavUser",
    "webdavPassword",
    "webdavDeviceName",
];

/// 判断某个 `config.xml` 键是否在白名单内。
pub fn is_whitelisted_key(key: &str) -> bool {
    CONFIG_XML_WHITELIST.contains(&key)
        || key.starts_with("remoteServer:password:")
        || key.starts_with("remoteServer:secret:")
}
