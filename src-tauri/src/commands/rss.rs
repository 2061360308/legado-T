use engine_model::{RssArticle, RssSource};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use tauri::{AppHandle, State};

use crate::data::source_repo::RssSourceRepo;
use crate::engine::create_pipeline;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn value_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn value_opt_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .filter(|text| !text.is_empty())
}

fn value_variables(value: &Value) -> String {
    match value.get("variable") {
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => "{}".to_string(),
    }
}

fn article_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let variables: String = row.get(11)?;
    let read: i64 = row.get(12)?;
    Ok(json!({
        "origin": row.get::<_, String>(0)?,
        "link": row.get::<_, String>(1)?,
        "sort": row.get::<_, String>(2)?,
        "title": row.get::<_, String>(3)?,
        "pubDate": row.get::<_, Option<String>>(4)?,
        "description": row.get::<_, Option<String>>(5)?,
        "content": row.get::<_, Option<String>>(6)?,
        "image": row.get::<_, Option<String>>(7)?,
        "group": row.get::<_, String>(8)?,
        "order": row.get::<_, i64>(9)?,
        "variable": serde_json::from_str::<Value>(&variables).unwrap_or_else(|_| json!({})),
        "read": read != 0,
    }))
}

fn list_articles(
    conn: &rusqlite::Connection,
    source_url: &str,
    sort_name: &str,
) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT a.sourceUrl, a.link, a.sort, a.title, a.pubDate, a.description,
                      a.content, a.image, a."group", a."order", a.variables,
                      CASE WHEN r.record IS NULL THEN 0 ELSE 1 END AS read
               FROM rss_articles a
               LEFT JOIN rss_read_records r ON r.record = a.link
               WHERE a.sourceUrl = ?1 AND a.sort = ?2
               ORDER BY a."order" DESC"#,
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params![source_url, sort_name], article_from_row)
        .map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }
    Ok(out)
}

fn star_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let variables: String = row.get(10)?;
    Ok(json!({
        "origin": row.get::<_, String>(0)?,
        "link": row.get::<_, String>(1)?,
        "sort": row.get::<_, String>(2)?,
        "title": row.get::<_, String>(3)?,
        "starTime": row.get::<_, i64>(4)?,
        "pubDate": row.get::<_, Option<String>>(5)?,
        "description": row.get::<_, Option<String>>(6)?,
        "content": row.get::<_, Option<String>>(7)?,
        "image": row.get::<_, Option<String>>(8)?,
        "group": row.get::<_, String>(9)?,
        "variable": serde_json::from_str::<Value>(&variables).unwrap_or_else(|_| json!({})),
    }))
}

fn list_stars(
    conn: &rusqlite::Connection,
    group: Option<&str>,
) -> Result<Vec<Value>, String> {
    let (sql, param): (&str, Option<&str>) = match group {
        Some(group) => (
            r#"SELECT origin, link, sort, title, starTime, pubDate, description, content,
                      image, "group", variables
               FROM rss_stars WHERE "group" = ?1 ORDER BY starTime DESC"#,
            Some(group),
        ),
        None => (
            r#"SELECT origin, link, sort, title, starTime, pubDate, description, content,
                      image, "group", variables
               FROM rss_stars ORDER BY starTime DESC"#,
            None,
        ),
    };
    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = match param {
        Some(group) => stmt
            .query_map(params![group], star_from_row)
            .map_err(|error| error.to_string())?,
        None => stmt
            .query_map([], star_from_row)
            .map_err(|error| error.to_string())?,
    };
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }
    Ok(out)
}

fn load_sources(state: &State<'_, AppState>) -> Result<Vec<RssSource>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    RssSourceRepo::list(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_list(state: State<'_, AppState>) -> CmdResult<Vec<Value>> {
    load_sources(&state)?
        .into_iter()
        .map(|source| serde_json::to_value(source).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn rss_source_save(
    state: State<'_, AppState>,
    source: Value,
    old_source_url: Option<String>,
) -> CmdResult<()> {
    let source: RssSource = serde_json::from_value(source).map_err(|error| error.to_string())?;
    if source.source_url.trim().is_empty() {
        return Err("RSS 源 URL 不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    if let Some(old_url) = old_source_url.filter(|url| !url.trim().is_empty()) {
        if old_url != source.source_url {
            conn.execute(
                "UPDATE rss_stars SET origin = ?1 WHERE origin = ?2",
                params![source.source_url, old_url],
            )
            .map_err(|error| error.to_string())?;
            conn.execute(
                "UPDATE rss_articles SET sourceUrl = ?1 WHERE sourceUrl = ?2",
                params![source.source_url, old_url],
            )
            .map_err(|error| error.to_string())?;
            conn.execute(
                "UPDATE source_variables SET bookSourceUrl = ?1 WHERE bookSourceUrl = ?2",
                params![source.source_url, old_url],
            )
            .map_err(|error| error.to_string())?;
            conn.execute(
                "DELETE FROM rss_sources WHERE sourceUrl = ?1",
                params![old_url],
            )
            .map_err(|error| error.to_string())?;
        }
    }
    RssSourceRepo::upsert(&conn, &source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_delete(state: State<'_, AppState>, source_url: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    RssSourceRepo::delete(&conn, &source_url).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_import(state: State<'_, AppState>, json: String) -> CmdResult<usize> {
    let value: Value = serde_json::from_str(&json).map_err(|error| error.to_string())?;
    let values = value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value]);
    let mut sources = Vec::with_capacity(values.len());
    for value in values {
        sources.push(serde_json::from_value::<RssSource>(value).map_err(|error| error.to_string())?);
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    for source in &sources {
        RssSourceRepo::upsert(&conn, source).map_err(|error| error.to_string())?;
    }
    Ok(sources.len())
}

#[tauri::command]
pub fn rss_source_export(state: State<'_, AppState>) -> CmdResult<String> {
    let sources = load_sources(&state)?;
    serde_json::to_string_pretty(&sources).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_article_list(
    state: State<'_, AppState>,
    source_url: String,
    sort_name: String,
) -> CmdResult<Vec<Value>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    list_articles(&conn, &source_url, &sort_name)
}

#[tauri::command]
pub fn rss_article_clear(
    state: State<'_, AppState>,
    source_url: Option<String>,
    sort_name: Option<String>,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    match (source_url, sort_name) {
        (Some(source_url), Some(sort_name)) => conn
            .execute(
                "DELETE FROM rss_articles WHERE sourceUrl = ?1 AND sort = ?2",
                params![source_url, sort_name],
            )
            .map(|_| ())
            .map_err(|error| error.to_string()),
        (Some(source_url), None) => conn
            .execute(
                "DELETE FROM rss_articles WHERE sourceUrl = ?1",
                params![source_url],
            )
            .map(|_| ())
            .map_err(|error| error.to_string()),
        _ => conn
            .execute("DELETE FROM rss_articles", [])
            .map(|_| ())
            .map_err(|error| error.to_string()),
    }
}

#[tauri::command]
pub fn rss_star_list(
    state: State<'_, AppState>,
    group: Option<String>,
) -> CmdResult<Vec<Value>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    list_stars(&conn, group.as_deref())
}

#[tauri::command]
pub fn rss_star_groups(state: State<'_, AppState>) -> CmdResult<Vec<String>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut stmt = conn
        .prepare(r#"SELECT "group" FROM rss_stars GROUP BY "group" ORDER BY "group""#)
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn rss_star_get(
    state: State<'_, AppState>,
    origin: String,
    link: String,
) -> CmdResult<Option<Value>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.query_row(
        r#"SELECT origin, link, sort, title, starTime, pubDate, description, content,
                  image, "group", variables
           FROM rss_stars WHERE origin = ?1 AND link = ?2"#,
        params![origin, link],
        star_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_star_save(
    state: State<'_, AppState>,
    article: Value,
    group: Option<String>,
) -> CmdResult<()> {
    let origin = value_string(&article, "origin");
    let link = value_string(&article, "link");
    if origin.is_empty() || link.is_empty() {
        return Err("收藏文章缺少 origin 或 link".to_string());
    }
    let group = group
        .filter(|value| !value.trim().is_empty())
        .or_else(|| value_opt_string(&article, "group"))
        .unwrap_or_else(|| "默认分组".to_string());
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        r#"INSERT INTO rss_stars
           (origin, link, sort, title, starTime, pubDate, description, content, image, "group", variables)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
           ON CONFLICT(origin, link) DO UPDATE SET
             sort=excluded.sort, title=excluded.title, starTime=excluded.starTime,
             pubDate=excluded.pubDate, description=excluded.description,
             content=excluded.content, image=excluded.image, "group"=excluded."group",
             variables=excluded.variables"#,
        params![
            origin,
            link,
            value_string(&article, "sort"),
            value_string(&article, "title"),
            chrono::Utc::now().timestamp_millis(),
            value_opt_string(&article, "pubDate"),
            value_opt_string(&article, "description"),
            value_opt_string(&article, "content"),
            value_opt_string(&article, "image"),
            group,
            value_variables(&article),
        ],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_star_update(
    state: State<'_, AppState>,
    origin: String,
    link: String,
    title: Option<String>,
    group: Option<String>,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    if let Some(title) = title.filter(|value| !value.trim().is_empty()) {
        conn.execute(
            "UPDATE rss_stars SET title = ?1 WHERE origin = ?2 AND link = ?3",
            params![title, origin, link],
        )
        .map_err(|error| error.to_string())?;
    }
    if let Some(group) = group.filter(|value| !value.trim().is_empty()) {
        conn.execute(
            r#"UPDATE rss_stars SET "group" = ?1 WHERE origin = ?2 AND link = ?3"#,
            params![group, origin, link],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn rss_star_delete(
    state: State<'_, AppState>,
    origin: String,
    link: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        "DELETE FROM rss_stars WHERE origin = ?1 AND link = ?2",
        params![origin, link],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_star_delete_group(state: State<'_, AppState>, group: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        r#"DELETE FROM rss_stars WHERE "group" = ?1"#,
        params![group],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_star_clear(state: State<'_, AppState>) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute("DELETE FROM rss_stars", [])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_read_record_add(
    state: State<'_, AppState>,
    record: String,
    title: Option<String>,
) -> CmdResult<()> {
    if record.trim().is_empty() {
        return Ok(());
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        r#"INSERT INTO rss_read_records (record, title, readTime, read)
           VALUES (?1, ?2, ?3, 1)
           ON CONFLICT(record) DO UPDATE SET title=excluded.title, readTime=excluded.readTime"#,
        params![record, title, chrono::Utc::now().timestamp_millis()],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_read_record_list(state: State<'_, AppState>) -> CmdResult<Vec<Value>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut stmt = conn
        .prepare("SELECT record, title, readTime, read FROM rss_read_records ORDER BY readTime DESC")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(json!({
                "record": row.get::<_, String>(0)?,
                "title": row.get::<_, Option<String>>(1)?,
                "readTime": row.get::<_, i64>(2)?,
                "read": row.get::<_, i64>(3)? != 0,
            }))
        })
        .map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn rss_read_record_clear(state: State<'_, AppState>) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute("DELETE FROM rss_read_records", [])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command(async)]
pub async fn rss_articles(
    app: AppHandle,
    state: State<'_, AppState>,
    source_url: String,
    sort_name: String,
    sort_url: String,
    page: Option<i32>,
) -> CmdResult<Value> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        RssSourceRepo::list(&conn)
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|source| source.source_url == source_url)
            .ok_or_else(|| "RSS 源不存在".to_string())?
    };
    let sort_url = if sort_url.trim().is_empty() {
        source.source_url.clone()
    } else {
        sort_url
    };
    let page_number = page.unwrap_or(1).max(1);
    let source_url_for_db = source_url.clone();
    let sort_name_for_db = sort_name.clone();
    let (articles, next) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<RssArticle>, Option<String>), String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let pipeline = create_pipeline(&app)?;
            runtime
                .block_on(pipeline.rss_articles(&source, &sort_name, &sort_url, page_number))
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| error.to_string())??;

    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let order_base = chrono::Utc::now().timestamp_millis()
        - (i64::from(page_number) - 1) * 10_000;
    for (index, article) in articles.iter().enumerate() {
        let sort = if article.sort.trim().is_empty() {
            sort_name_for_db.as_str()
        } else {
            article.sort.as_str()
        };
        conn.execute(
            r#"INSERT INTO rss_articles
               (sourceUrl, link, sort, title, pubDate, description, content, image,
                "group", read, favorite, "order", variables)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'默认分组',0,0,?9,?10)
               ON CONFLICT(sourceUrl, link) DO UPDATE SET
                 sort=excluded.sort, title=excluded.title, pubDate=excluded.pubDate,
                 description=excluded.description, content=excluded.content,
                 image=excluded.image, "order"=excluded."order", variables=excluded.variables"#,
            params![
                source_url_for_db,
                article.link,
                sort,
                article.title,
                article.pub_date,
                article.description,
                article.content,
                article.image,
                order_base - index as i64,
                serde_json::to_string(&article.variables).unwrap_or_else(|_| "{}".to_string()),
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    let cached = list_articles(&conn, &source_url_for_db, &sort_name_for_db)?;
    Ok(json!({
        "articles": cached,
        "next": next,
    }))
}

#[tauri::command(async)]
pub async fn rss_content(
    app: AppHandle,
    state: State<'_, AppState>,
    source_url: String,
    article: Value,
    rule_content: Option<String>,
) -> CmdResult<String> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        RssSourceRepo::list(&conn)
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|source| source.source_url == source_url)
            .ok_or_else(|| "RSS 源不存在".to_string())?
    };
    let article: RssArticle = serde_json::from_value(article).map_err(|error| error.to_string())?;
    let rule = rule_content
        .or_else(|| source.rule_content.clone())
        .unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        runtime
            .block_on(pipeline.rss_content(&source, &article, &rule))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
