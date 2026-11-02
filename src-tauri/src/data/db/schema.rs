//! 数据库建表与迁移（T3.1）。
//!
//! 表结构仅内部一致、不参与交换；交换格式由 `rule_io`/`BackupService` 负责。

use rusqlite::Connection;

use crate::Result;

/// 初始化全部表结构（幂等）。
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS books (
            bookUrl           TEXT PRIMARY KEY,
            name              TEXT NOT NULL,
            author            TEXT NOT NULL DEFAULT '',
            origin            TEXT NOT NULL DEFAULT 'local',
            originName        TEXT NOT NULL DEFAULT '',
            charset           TEXT,
            type              INTEGER NOT NULL DEFAULT 0,
            "group"           INTEGER NOT NULL DEFAULT 0,
            tocUrl            TEXT NOT NULL DEFAULT '',
            intro             TEXT,
            kind              TEXT,
            customTag         TEXT,
            coverUrl          TEXT,
            customCoverUrl    TEXT,
            latestChapterTitle TEXT,
            durChapterIndex   INTEGER NOT NULL DEFAULT 0,
            durChapterPos     INTEGER NOT NULL DEFAULT 0,
            durChapterTitle   TEXT,
            durChapterTime    INTEGER NOT NULL DEFAULT 0,
            totalChapterNum   INTEGER NOT NULL DEFAULT 0,
            canUpdate         INTEGER NOT NULL DEFAULT 0,
            "order"           INTEGER NOT NULL DEFAULT 0,
            syncTime          INTEGER NOT NULL DEFAULT 0,
            readConfig        TEXT,
            customIntro       TEXT,
            sourcePath        TEXT,
            sourceHash        TEXT,
            originalPath      TEXT
        );
        DROP INDEX IF EXISTS idx_books_name_author;
        CREATE INDEX IF NOT EXISTS idx_books_name_author ON books(name, author);

        CREATE TABLE IF NOT EXISTS chapters (
            url           TEXT NOT NULL,
            bookUrl       TEXT NOT NULL,
            title         TEXT NOT NULL DEFAULT '',
            "index"       INTEGER NOT NULL DEFAULT 0,
            isVolume      INTEGER NOT NULL DEFAULT 0,
            start         INTEGER NOT NULL DEFAULT 0,
            end           INTEGER NOT NULL DEFAULT 0,
            isVip         INTEGER NOT NULL DEFAULT 0,
            isPay         INTEGER NOT NULL DEFAULT 0,
            tag           TEXT,
            resourceUrl   TEXT,
            tocLevel      INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (url, bookUrl),
            FOREIGN KEY (bookUrl) REFERENCES books(bookUrl) ON DELETE CASCADE
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_chapters_book_index ON chapters(bookUrl, "index");

        CREATE TABLE IF NOT EXISTS book_groups (
            groupId       INTEGER PRIMARY KEY,
            groupName     TEXT NOT NULL DEFAULT '',
            "order"       INTEGER NOT NULL DEFAULT 0,
            bookSort      INTEGER NOT NULL DEFAULT 0,
            show          INTEGER NOT NULL DEFAULT 1,
            enableRefresh INTEGER NOT NULL DEFAULT 1,
            coverUrl      TEXT
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            time          INTEGER PRIMARY KEY,
            bookName      TEXT NOT NULL DEFAULT '',
            bookAuthor    TEXT NOT NULL DEFAULT '',
            bookUrl       TEXT NOT NULL DEFAULT '',
            chapterIndex  INTEGER NOT NULL DEFAULT 0,
            chapterPos    INTEGER NOT NULL DEFAULT 0,
            chapterName   TEXT NOT NULL DEFAULT '',
            bookText      TEXT NOT NULL DEFAULT '',
            content       TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_bookmarks_book ON bookmarks(bookName, bookAuthor);

        CREATE TABLE IF NOT EXISTS readRecord (
            deviceId        TEXT NOT NULL,
            bookName        TEXT NOT NULL,
            author          TEXT NOT NULL DEFAULT '',
            readTime        INTEGER NOT NULL DEFAULT 0,
            lastRead        INTEGER NOT NULL DEFAULT 0,
            durChapterIndex INTEGER NOT NULL DEFAULT 0,
            durChapterPos   INTEGER NOT NULL DEFAULT 0,
            durChapterTitle TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (deviceId, bookName)
        );

        CREATE TABLE IF NOT EXISTS txtTocRules (
            id           INTEGER PRIMARY KEY,
            name         TEXT NOT NULL DEFAULT '',
            rule         TEXT NOT NULL DEFAULT '',
            example      TEXT NOT NULL DEFAULT '',
            serialNumber INTEGER NOT NULL DEFAULT 0,
            enable       INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS replace_rules (
            id                 INTEGER PRIMARY KEY,
            name               TEXT NOT NULL DEFAULT '',
            "group"            TEXT NOT NULL DEFAULT '',
            pattern            TEXT NOT NULL DEFAULT '',
            replacement        TEXT NOT NULL DEFAULT '',
            scope              TEXT NOT NULL DEFAULT '',
            excludeScope       TEXT NOT NULL DEFAULT '',
            scopeTitle         INTEGER NOT NULL DEFAULT 0,
            scopeContent       INTEGER NOT NULL DEFAULT 1,
            isEnabled          INTEGER NOT NULL DEFAULT 1,
            isRegex            INTEGER NOT NULL DEFAULT 1,
            timeoutMillisecond INTEGER NOT NULL DEFAULT 0,
            sortOrder          INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS dictRules (
            name       TEXT PRIMARY KEY,
            urlRule    TEXT NOT NULL DEFAULT '',
            showRule   TEXT NOT NULL DEFAULT '',
            enabled    INTEGER NOT NULL DEFAULT 1,
            sortNumber INTEGER NOT NULL DEFAULT 0,
            jsLib      TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS httpTTS (
            id                INTEGER PRIMARY KEY,
            name              TEXT NOT NULL DEFAULT '',
            url               TEXT NOT NULL DEFAULT '',
            contentType       TEXT NOT NULL DEFAULT '',
            concurrentRate    TEXT NOT NULL DEFAULT '',
            loginUrl          TEXT NOT NULL DEFAULT '',
            loginUi           TEXT NOT NULL DEFAULT '',
            header            TEXT NOT NULL DEFAULT '',
            jsLib             TEXT NOT NULL DEFAULT '',
            enabledCookieJar  INTEGER NOT NULL DEFAULT 0,
            loginCheckJs      TEXT NOT NULL DEFAULT '',
            lastUpdateTime    INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS caches (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS themes (
            themeName       TEXT PRIMARY KEY,
            isNightTheme    INTEGER NOT NULL DEFAULT 0,
            primaryColor    TEXT NOT NULL DEFAULT '',
            accentColor     TEXT NOT NULL DEFAULT '',
            backgroundColor TEXT NOT NULL DEFAULT '',
            bottomBackground TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS read_style_schemes (
            seqNo   INTEGER PRIMARY KEY,
            name    TEXT NOT NULL DEFAULT '',
            payload TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS book_sources (
            bookSourceUrl       TEXT PRIMARY KEY,
            bookSourceName      TEXT NOT NULL DEFAULT '',
            bookSourceGroup     TEXT NOT NULL DEFAULT '',
            bookSourceType      INTEGER NOT NULL DEFAULT 0,
            enabled             INTEGER NOT NULL DEFAULT 1,
            enabledExplore      INTEGER NOT NULL DEFAULT 1,
            enabledCookieJar    INTEGER NOT NULL DEFAULT 0,
            customOrder         INTEGER NOT NULL DEFAULT 0,
            weight              INTEGER NOT NULL DEFAULT 0,
            lastUpdateTime      INTEGER NOT NULL DEFAULT 0,
            respondTime         INTEGER NOT NULL DEFAULT 0,
            json                TEXT NOT NULL,
            updatedAt           INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_book_sources_name ON book_sources(bookSourceName);
        CREATE INDEX IF NOT EXISTS idx_book_sources_group ON book_sources(bookSourceGroup);

        CREATE TABLE IF NOT EXISTS book_source_groups (
            name       TEXT PRIMARY KEY,
            sortNumber INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS rss_sources (
            sourceUrl       TEXT PRIMARY KEY,
            sourceName      TEXT NOT NULL DEFAULT '',
            sourceGroup     TEXT NOT NULL DEFAULT '',
            enabled         INTEGER NOT NULL DEFAULT 1,
            customOrder     INTEGER NOT NULL DEFAULT 0,
            lastUpdateTime  INTEGER NOT NULL DEFAULT 0,
            json            TEXT NOT NULL,
            updatedAt       INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS rss_articles (
            sourceUrl   TEXT NOT NULL,
            link        TEXT NOT NULL,
            sort        TEXT NOT NULL DEFAULT '',
            title       TEXT NOT NULL DEFAULT '',
            pubDate     TEXT,
            description TEXT,
            content     TEXT,
            image       TEXT,
            "group"     TEXT NOT NULL DEFAULT '默认分组',
            read        INTEGER NOT NULL DEFAULT 0,
            favorite    INTEGER NOT NULL DEFAULT 0,
            "order"     INTEGER NOT NULL DEFAULT 0,
            variables   TEXT NOT NULL DEFAULT '{}',
            PRIMARY KEY (sourceUrl, link)
        );
        CREATE INDEX IF NOT EXISTS idx_rss_articles_sort ON rss_articles(sourceUrl, sort, "order");

        CREATE TABLE IF NOT EXISTS rss_stars (
            origin      TEXT NOT NULL,
            link        TEXT NOT NULL,
            sort        TEXT NOT NULL DEFAULT '',
            title       TEXT NOT NULL DEFAULT '',
            starTime    INTEGER NOT NULL DEFAULT 0,
            pubDate     TEXT,
            description TEXT,
            content     TEXT,
            image       TEXT,
            "group"     TEXT NOT NULL DEFAULT '默认分组',
            variables   TEXT NOT NULL DEFAULT '{}',
            PRIMARY KEY (origin, link)
        );
        CREATE INDEX IF NOT EXISTS idx_rss_stars_group ON rss_stars("group", starTime);

        CREATE TABLE IF NOT EXISTS rss_read_records (
            record      TEXT PRIMARY KEY,
            title       TEXT,
            readTime    INTEGER NOT NULL DEFAULT 0,
            read        INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_rss_read_records_time ON rss_read_records(readTime);

        CREATE TABLE IF NOT EXISTS source_variables (
            bookSourceUrl TEXT NOT NULL,
            key           TEXT NOT NULL,
            value         TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (bookSourceUrl, key)
        );

        CREATE TABLE IF NOT EXISTS search_history (
            keyword  TEXT PRIMARY KEY,
            lastUsed INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS search_books (
            bookUrl       TEXT NOT NULL,
            bookSourceUrl TEXT NOT NULL,
            json          TEXT NOT NULL,
            updatedAt     INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (bookUrl, bookSourceUrl)
        );
        CREATE INDEX IF NOT EXISTS idx_search_books_updated ON search_books(updatedAt);

        CREATE TABLE IF NOT EXISTS chapter_contents (
            bookUrl      TEXT NOT NULL,
            chapterIndex INTEGER NOT NULL,
            content      TEXT NOT NULL DEFAULT '',
            updatedAt    INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (bookUrl, chapterIndex)
        );

        CREATE TABLE IF NOT EXISTS app_logs (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            time    INTEGER NOT NULL DEFAULT 0,
            level   TEXT NOT NULL DEFAULT 'info',
            source  TEXT NOT NULL DEFAULT '',
            message TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_app_logs_time ON app_logs(time);
        CREATE INDEX IF NOT EXISTS idx_app_logs_source ON app_logs(source);

        CREATE TABLE IF NOT EXISTS kv (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );
        "#,
    )?;
    ensure_column(conn, "books", "customIntro", "TEXT")?;
    ensure_column(conn, "books", "sourcePath", "TEXT")?;
    ensure_column(conn, "books", "sourceHash", "TEXT")?;
    ensure_column(conn, "books", "originalPath", "TEXT")?;
    ensure_column(conn, "books", "variables", "TEXT")?;
    ensure_column(conn, "books", "wordCount", "TEXT")?;
    ensure_column(conn, "books", "imageStyle", "TEXT")?;
    ensure_column(conn, "chapters", "tocLevel", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "chapters", "baseUrl", "TEXT")?;
    ensure_column(conn, "chapters", "bookUrl", "TEXT")?;
    ensure_column(conn, "chapters", "variables", "TEXT")?;
    ensure_column(conn, "rss_articles", "group", "TEXT NOT NULL DEFAULT '默认分组'")?;
    ensure_column(conn, "rss_articles", "order", "INTEGER NOT NULL DEFAULT 0")?;
    Ok(())
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn ensure_column(conn: &Connection, table: &str, column: &str, kind: &str) -> Result<()> {
    let table_sql = quote_identifier(table);
    let column_sql = quote_identifier(column);
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_sql})"))?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(std::result::Result::ok)
        .any(|name| name == column);
    if !exists {
        conn.execute(
            &format!("ALTER TABLE {table_sql} ADD COLUMN {column_sql} {kind}"),
            [],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_reserved_column_names_idempotently() {
        let conn = Connection::open_in_memory().expect("open memory database");
        conn.execute_batch(
            r#"
            CREATE TABLE rss_articles (
                sourceUrl   TEXT NOT NULL,
                link        TEXT NOT NULL,
                sort        TEXT NOT NULL DEFAULT '',
                title       TEXT NOT NULL DEFAULT '',
                pubDate     TEXT,
                description TEXT,
                content     TEXT,
                image       TEXT,
                read        INTEGER NOT NULL DEFAULT 0,
                favorite    INTEGER NOT NULL DEFAULT 0,
                variables   TEXT NOT NULL DEFAULT '{}',
                PRIMARY KEY (sourceUrl, link)
            );
            "#,
        )
        .expect("create legacy table");

        init_schema(&conn).expect("first migration");
        init_schema(&conn).expect("second migration");

        let mut stmt = conn
            .prepare("PRAGMA table_info(rss_articles)")
            .expect("prepare table info");
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query columns")
            .filter_map(std::result::Result::ok)
            .collect();
        assert!(columns.iter().any(|name| name == "group"));
        assert!(columns.iter().any(|name| name == "order"));
    }
}
