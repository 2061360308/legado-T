//! 数据库连接与 schema 初始化。

pub mod schema;

use rusqlite::Connection;

use crate::{Error, Result};

/// 打开（或创建）数据库并初始化表结构。
pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path).map_err(|e| Error::Io(e.to_string()))?;
    schema::init_schema(&conn)?;
    Ok(conn)
}

/// 打开内存数据库（测试用）。
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory().map_err(|e| Error::Io(e.to_string()))?;
    schema::init_schema(&conn)?;
    Ok(conn)
}
