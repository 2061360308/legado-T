//! 仓储层（T3.3）：对象 ↔ 表映射，含类型标志/位掩码拆解与级联。

use rusqlite::{params, Connection, Row};

use crate::domain::model::{
    Book, BookChapter, BookGroup, Bookmark, DictRule, HttpTts, ReplaceRule, TxtTocRule,
};
use crate::{Error, Result};

/// 分组位掩码运算：加入某分组。
pub fn group_add(current: u64, bit: u64) -> u64 {
    current | bit
}

/// 分组位掩码运算：移除某分组。
pub fn group_remove(current: u64, bit: u64) -> u64 {
    current & !bit
}

/// 分组位掩码运算：是否属于某分组。
pub fn group_has(current: u64, bit: u64) -> bool {
    current & bit != 0
}

fn read_config_to_json(b: &Book) -> Option<String> {
    b.read_config
        .as_ref()
        .and_then(|c| serde_json::to_string(c).ok())
}

fn json_to_read_config(s: Option<String>) -> Option<crate::domain::model::ReadConfig> {
    s.and_then(|s| serde_json::from_str(&s).ok())
}

/// 书籍仓储。
pub struct BookRepo;

impl BookRepo {
    pub fn upsert(conn: &Connection, b: &Book) -> Result<()> {
        conn.execute(
            r#"INSERT INTO books (
                bookUrl,name,author,origin,originName,charset,type,"group",tocUrl,intro,kind,
                customTag,coverUrl,customCoverUrl,latestChapterTitle,durChapterIndex,durChapterPos,
                durChapterTitle,durChapterTime,totalChapterNum,canUpdate,"order",syncTime,readConfig)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)
               ON CONFLICT(bookUrl) DO UPDATE SET
                name=excluded.name, author=excluded.author, origin=excluded.origin,
                originName=excluded.originName, charset=excluded.charset, type=excluded.type,
                "group"=excluded."group", tocUrl=excluded.tocUrl, intro=excluded.intro,
                kind=excluded.kind, customTag=excluded.customTag, coverUrl=excluded.coverUrl,
                customCoverUrl=excluded.customCoverUrl, latestChapterTitle=excluded.latestChapterTitle,
                durChapterIndex=excluded.durChapterIndex, durChapterPos=excluded.durChapterPos,
                durChapterTitle=excluded.durChapterTitle, durChapterTime=excluded.durChapterTime,
                totalChapterNum=excluded.totalChapterNum, canUpdate=excluded.canUpdate,
                "order"=excluded."order", syncTime=excluded.syncTime, readConfig=excluded.readConfig"#,
            params![
                b.book_url, b.name, b.author, b.origin, b.origin_name, b.charset, b.r#type as i64,
                b.group as i64, b.toc_url, b.intro, b.kind, b.custom_tag, b.cover_url,
                b.custom_cover_url, b.latest_chapter_title, b.dur_chapter_index, b.dur_chapter_pos,
                b.dur_chapter_title, b.dur_chapter_time, b.total_chapter_num, b.can_update as i64,
                b.order, b.sync_time, read_config_to_json(b),
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, book_url: &str) -> Result<Option<Book>> {
        let mut stmt = conn.prepare(
            r#"SELECT bookUrl,name,author,origin,originName,charset,type,"group",tocUrl,intro,kind,
                      customTag,coverUrl,customCoverUrl,latestChapterTitle,durChapterIndex,durChapterPos,
                      durChapterTitle,durChapterTime,totalChapterNum,canUpdate,"order",syncTime,readConfig
               FROM books WHERE bookUrl = ?1"#,
        )?;
        let mut rows = stmt.query_map([book_url], map_book)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn list(conn: &Connection) -> Result<Vec<Book>> {
        let mut stmt = conn.prepare(
            r#"SELECT bookUrl,name,author,origin,originName,charset,type,"group",tocUrl,intro,kind,
                      customTag,coverUrl,customCoverUrl,latestChapterTitle,durChapterIndex,durChapterPos,
                      durChapterTitle,durChapterTime,totalChapterNum,canUpdate,"order",syncTime,readConfig
               FROM books ORDER BY "order" ASC"#,
        )?;
        let rows = stmt.query_map([], map_book)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 删除书籍；章节由外键 CASCADE 一并删除（spec 6.2-2）。
    pub fn delete(conn: &Connection, book_url: &str) -> Result<()> {
        conn.execute("DELETE FROM books WHERE bookUrl = ?1", [book_url])?;
        Ok(())
    }

    pub fn count(conn: &Connection) -> Result<i64> {
        Ok(conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))?)
    }
}

fn map_book(row: &Row<'_>) -> rusqlite::Result<Book> {
    let read_config: Option<String> = row.get(23)?;
    Ok(Book {
        book_url: row.get(0)?,
        name: row.get(1)?,
        author: row.get(2)?,
        origin: row.get(3)?,
        origin_name: row.get(4)?,
        charset: row.get(5)?,
        r#type: row.get::<_, i64>(6)? as u64,
        group: row.get::<_, i64>(7)? as u64,
        toc_url: row.get(8)?,
        intro: row.get(9)?,
        kind: row.get(10)?,
        custom_tag: row.get(11)?,
        cover_url: row.get(12)?,
        custom_cover_url: row.get(13)?,
        latest_chapter_title: row.get(14)?,
        dur_chapter_index: row.get(15)?,
        dur_chapter_pos: row.get(16)?,
        dur_chapter_title: row.get(17)?,
        dur_chapter_time: row.get(18)?,
        total_chapter_num: row.get(19)?,
        can_update: row.get::<_, i64>(20)? != 0,
        order: row.get(21)?,
        sync_time: row.get(22)?,
        read_config: json_to_read_config(read_config),
    })
}

/// 章节仓储。
pub struct ChapterRepo;

impl ChapterRepo {
    pub fn replace_all(conn: &Connection, book_url: &str, chapters: &[BookChapter]) -> Result<()> {
        let tx = conn.unchecked_transaction()?;
        tx.execute("DELETE FROM chapters WHERE bookUrl = ?1", [book_url])?;
        {
            let mut stmt = tx.prepare(
                r#"INSERT OR REPLACE INTO chapters
                   (url,bookUrl,title,"index",isVolume,start,end,isVip,isPay,tag,resourceUrl)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
            )?;
            for c in chapters {
                stmt.execute(params![
                    c.url,
                    book_url,
                    c.title,
                    c.index,
                    c.is_volume as i64,
                    c.start,
                    c.end,
                    c.is_vip as i64,
                    c.is_pay as i64,
                    c.tag,
                    c.resource_url,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list(conn: &Connection, book_url: &str) -> Result<Vec<BookChapter>> {
        let mut stmt = conn.prepare(
            r#"SELECT url,title,"index",isVolume,start,end,isVip,isPay,tag,resourceUrl
               FROM chapters WHERE bookUrl = ?1 ORDER BY "index" ASC"#,
        )?;
        let rows = stmt.query_map([book_url], |row| {
            Ok(BookChapter {
                url: row.get(0)?,
                title: row.get(1)?,
                index: row.get(2)?,
                is_volume: row.get::<_, i64>(3)? != 0,
                start: row.get(4)?,
                end: row.get(5)?,
                is_vip: row.get::<_, i64>(6)? != 0,
                is_pay: row.get::<_, i64>(7)? != 0,
                tag: row.get(8)?,
                resource_url: row.get(9)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn count(conn: &Connection, book_url: &str) -> Result<i64> {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM chapters WHERE bookUrl = ?1",
            [book_url],
            |r| r.get(0),
        )?)
    }
}

/// 分组仓储。
pub struct GroupRepo;

impl GroupRepo {
    pub fn upsert(conn: &Connection, g: &BookGroup) -> Result<()> {
        conn.execute(
            r#"INSERT INTO book_groups (groupId,groupName,"order",bookSort,show,enableRefresh,coverUrl)
               VALUES (?1,?2,?3,?4,?5,?6,?7)
               ON CONFLICT(groupId) DO UPDATE SET groupName=excluded.groupName, "order"=excluded."order",
                 bookSort=excluded.bookSort, show=excluded.show, enableRefresh=excluded.enableRefresh,
                 coverUrl=excluded.coverUrl"#,
            params![
                g.group_id as i64,
                g.group_name,
                g.order,
                g.book_sort,
                g.show as i64,
                g.enable_refresh as i64,
                g.cover_url
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<BookGroup>> {
        let mut stmt = conn.prepare(
            r#"SELECT groupId,groupName,"order",bookSort,show,enableRefresh,coverUrl FROM book_groups"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BookGroup {
                group_id: row.get::<_, i64>(0)? as u64,
                group_name: row.get(1)?,
                order: row.get(2)?,
                book_sort: row.get(3)?,
                show: row.get::<_, i64>(4)? != 0,
                enable_refresh: row.get::<_, i64>(5)? != 0,
                cover_url: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, group_id: u64) -> Result<()> {
        conn.execute(
            "DELETE FROM book_groups WHERE groupId = ?1",
            [group_id as i64],
        )?;
        Ok(())
    }
}

/// 书签仓储（弱关联，不级联）。
pub struct BookmarkRepo;

impl BookmarkRepo {
    pub fn upsert(conn: &Connection, m: &Bookmark) -> Result<()> {
        conn.execute(
            r#"INSERT OR REPLACE INTO bookmarks
               (time,bookName,bookAuthor,bookUrl,chapterIndex,chapterPos,chapterName,bookText,content)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
            params![
                m.time, m.book_name, m.book_author, m.book_url, m.chapter_index, m.chapter_pos,
                m.chapter_name, m.book_text, m.content
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<Bookmark>> {
        let mut stmt = conn.prepare(
            r#"SELECT time,bookName,bookAuthor,bookUrl,chapterIndex,chapterPos,chapterName,bookText,content
               FROM bookmarks ORDER BY time DESC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Bookmark {
                time: row.get(0)?,
                book_name: row.get(1)?,
                book_author: row.get(2)?,
                book_url: row.get(3)?,
                chapter_index: row.get(4)?,
                chapter_pos: row.get(5)?,
                chapter_name: row.get(6)?,
                book_text: row.get(7)?,
                content: row.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, time: i64) -> Result<()> {
        conn.execute("DELETE FROM bookmarks WHERE time = ?1", [time])?;
        Ok(())
    }
}

/// 目录规则仓储。
pub struct TxtTocRuleRepo;

impl TxtTocRuleRepo {
    pub fn upsert(conn: &Connection, r: &TxtTocRule) -> Result<()> {
        conn.execute(
            r#"INSERT INTO txtTocRules (id,name,rule,example,serialNumber,enable)
               VALUES (?1,?2,?3,?4,?5,?6)
               ON CONFLICT(id) DO UPDATE SET name=excluded.name, rule=excluded.rule,
                 example=excluded.example, serialNumber=excluded.serialNumber, enable=excluded.enable"#,
            params![r.id, r.name, r.rule, r.example, r.serial_number, r.enable as i64],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<TxtTocRule>> {
        let mut stmt = conn.prepare(
            "SELECT id,name,rule,example,serialNumber,enable FROM txtTocRules ORDER BY serialNumber DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TxtTocRule {
                id: row.get(0)?,
                name: row.get(1)?,
                rule: row.get(2)?,
                example: row.get(3)?,
                serial_number: row.get(4)?,
                enable: row.get::<_, i64>(5)? != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn count(conn: &Connection) -> Result<i64> {
        Ok(conn.query_row("SELECT COUNT(*) FROM txtTocRules", [], |r| r.get(0))?)
    }

    pub fn get(conn: &Connection, id: i64) -> Result<Option<TxtTocRule>> {
        Ok(Self::list(conn)?.into_iter().find(|r| r.id == id))
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM txtTocRules WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn delete_all(conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM txtTocRules", [])?;
        Ok(())
    }
}

/// 替换净化规则仓储。
pub struct ReplaceRuleRepo;

impl ReplaceRuleRepo {
    pub fn upsert(conn: &Connection, r: &ReplaceRule) -> Result<()> {
        conn.execute(
            r#"INSERT INTO replace_rules
               (id,name,"group",pattern,replacement,scope,excludeScope,scopeTitle,scopeContent,
                isEnabled,isRegex,timeoutMillisecond,sortOrder)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
               ON CONFLICT(id) DO UPDATE SET name=excluded.name, "group"=excluded."group",
                 pattern=excluded.pattern, replacement=excluded.replacement, scope=excluded.scope,
                 excludeScope=excluded.excludeScope, scopeTitle=excluded.scopeTitle,
                 scopeContent=excluded.scopeContent, isEnabled=excluded.isEnabled,
                 isRegex=excluded.isRegex, timeoutMillisecond=excluded.timeoutMillisecond,
                 sortOrder=excluded.sortOrder"#,
            params![
                r.id,
                r.name,
                r.group,
                r.pattern,
                r.replacement,
                r.scope,
                r.exclude_scope,
                r.scope_title as i64,
                r.scope_content as i64,
                r.is_enabled as i64,
                r.is_regex as i64,
                r.timeout_millisecond,
                r.sort_order
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<ReplaceRule>> {
        let mut stmt = conn.prepare(
            r#"SELECT id,name,"group",pattern,replacement,scope,excludeScope,scopeTitle,scopeContent,
                      isEnabled,isRegex,timeoutMillisecond,sortOrder
               FROM replace_rules ORDER BY sortOrder ASC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ReplaceRule {
                id: row.get(0)?,
                name: row.get(1)?,
                group: row.get(2)?,
                pattern: row.get(3)?,
                replacement: row.get(4)?,
                scope: row.get(5)?,
                exclude_scope: row.get(6)?,
                scope_title: row.get::<_, i64>(7)? != 0,
                scope_content: row.get::<_, i64>(8)? != 0,
                is_enabled: row.get::<_, i64>(9)? != 0,
                is_regex: row.get::<_, i64>(10)? != 0,
                timeout_millisecond: row.get(11)?,
                sort_order: row.get(12)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM replace_rules WHERE id = ?1", [id])?;
        Ok(())
    }
}

/// 字典规则仓储（以 name 为唯一键，同名覆盖）。
pub struct DictRuleRepo;

impl DictRuleRepo {
    pub fn upsert(conn: &Connection, r: &DictRule) -> Result<()> {
        conn.execute(
            r#"INSERT INTO dictRules (name,urlRule,showRule,enabled,sortNumber,jsLib)
               VALUES (?1,?2,?3,?4,?5,?6)
               ON CONFLICT(name) DO UPDATE SET urlRule=excluded.urlRule, showRule=excluded.showRule,
                 enabled=excluded.enabled, sortNumber=excluded.sortNumber, jsLib=excluded.jsLib"#,
            params![
                r.name,
                r.url_rule,
                r.show_rule,
                r.enabled as i64,
                r.sort_number,
                r.js_lib
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<DictRule>> {
        let mut stmt =
            conn.prepare("SELECT name,urlRule,showRule,enabled,sortNumber,jsLib FROM dictRules ORDER BY sortNumber DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(DictRule {
                name: row.get(0)?,
                url_rule: row.get(1)?,
                show_rule: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                sort_number: row.get(4)?,
                js_lib: row.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, name: &str) -> Result<()> {
        conn.execute("DELETE FROM dictRules WHERE name = ?1", [name])?;
        Ok(())
    }
}

/// 在线语音引擎仓储。
pub struct HttpTtsRepo;

impl HttpTtsRepo {
    pub fn upsert(conn: &Connection, r: &HttpTts) -> Result<()> {
        conn.execute(
            r#"INSERT INTO httpTTS
               (id,name,url,contentType,concurrentRate,loginUrl,loginUi,header,jsLib,enabledCookieJar,
                loginCheckJs,lastUpdateTime)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
               ON CONFLICT(id) DO UPDATE SET name=excluded.name, url=excluded.url,
                 contentType=excluded.contentType, concurrentRate=excluded.concurrentRate,
                 loginUrl=excluded.loginUrl, loginUi=excluded.loginUi, header=excluded.header,
                 jsLib=excluded.jsLib, enabledCookieJar=excluded.enabledCookieJar,
                 loginCheckJs=excluded.loginCheckJs, lastUpdateTime=excluded.lastUpdateTime"#,
            params![
                r.id, r.name, r.url, r.content_type, r.concurrent_rate, r.login_url, r.login_ui,
                r.header, r.js_lib, r.enabled_cookie_jar as i64, r.login_check_js, r.last_update_time
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<HttpTts>> {
        let mut stmt = conn.prepare(
            r#"SELECT id,name,url,contentType,concurrentRate,loginUrl,loginUi,header,jsLib,
                      enabledCookieJar,loginCheckJs,lastUpdateTime FROM httpTTS ORDER BY name ASC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HttpTts {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                content_type: row.get(3)?,
                concurrent_rate: row.get(4)?,
                login_url: row.get(5)?,
                login_ui: row.get(6)?,
                header: row.get(7)?,
                js_lib: row.get(8)?,
                enabled_cookie_jar: row.get::<_, i64>(9)? != 0,
                login_check_js: row.get(10)?,
                last_update_time: row.get(11)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM httpTTS WHERE id = ?1", [id])?;
        Ok(())
    }
}

/// 便捷错误构造（供上层使用）。
pub fn not_found(what: &str) -> Error {
    Error::NotFound(what.to_string())
}
