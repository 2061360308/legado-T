use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::data::repositories::BookRepo;
use crate::domain::model::ContentBlock;
use crate::domain::storage::BookStorage;
use crate::Result;

pub fn migrate_legacy_books(conn: &Connection, storage: &BookStorage) -> Result<usize> {
    let books = BookRepo::list(conn)?;
    let mut migrated = 0;
    for mut book in books {
        if book.source_path.is_some() {
            continue;
        }
        let source = PathBuf::from(&book.book_url);
        if !source.is_absolute() || !source.is_file() {
            continue;
        }
        let hash = BookStorage::hash_file(&source)?;
        let id = format!("local:{hash}");
        if id == book.book_url || BookRepo::get(conn, &id)?.is_some() {
            continue;
        }
        let ext = source
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| "txt".to_string());
        let paths = storage.paths_for_id(&id, &ext);
        fs::create_dir_all(&paths.dir)?;
        fs::create_dir_all(&paths.chapters)?;

        if !paths.source.exists() {
            fs::copy(&source, &paths.source)?;
        }
        let legacy_chapters = source.with_file_name(format!(
            "{}.chapters",
            source
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("book")
        ));
        if legacy_chapters.is_dir() {
            copy_dir(&legacy_chapters, &paths.chapters)?;
            rewrite_cached_image_paths(storage, &paths.chapters, &legacy_chapters)?;
        }

        let old_url = book.book_url.clone();
        book.book_url = id.clone();
        book.source_path = Some(storage.relative_string(&paths.source));
        book.source_hash = Some(hash);
        book.sync_time = mtime_secs(&paths.source);
        BookRepo::upsert(conn, &book)?;

        conn.execute(
            r#"UPDATE chapters
               SET bookUrl = ?1,
                   url = ?1 || substr(url, length(?2) + 1),
                   resourceUrl = CASE WHEN resourceUrl = ?2 THEN ?1 ELSE resourceUrl END
               WHERE bookUrl = ?2"#,
            params![id, old_url],
        )?;
        conn.execute(
            "UPDATE bookmarks SET bookUrl = ?1 WHERE bookUrl = ?2",
            params![id, old_url],
        )?;
        conn.execute(
            "UPDATE kv SET key = replace(key, ?2, ?1) WHERE instr(key, ?2) > 0",
            params![id, old_url],
        )?;
        conn.execute(
            "UPDATE caches SET key = replace(key, ?2, ?1) WHERE instr(key, ?2) > 0",
            params![id, old_url],
        )?;
        BookRepo::delete(conn, &old_url)?;

        if source.starts_with(storage.root()) {
            let _ = fs::remove_file(&source);
        }
        if legacy_chapters.starts_with(storage.root()) {
            let _ = fs::remove_dir_all(&legacy_chapters);
        }
        migrated += 1;
    }
    Ok(migrated)
}

fn mtime_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn rewrite_cached_image_paths(
    storage: &BookStorage,
    chapters: &Path,
    old_chapters: &Path,
) -> Result<()> {
    let Ok(entries) = fs::read_dir(chapters) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(mut blocks) = serde_json::from_str::<Vec<ContentBlock>>(&raw) else {
            continue;
        };
        for block in &mut blocks {
            let src = match block {
                ContentBlock::Image { src, .. } | ContentBlock::ImagePage { src, .. } => src,
                ContentBlock::Paragraph { .. } => continue,
            };
            let old = Path::new(src);
            let new = if old.starts_with(old_chapters) {
                old.strip_prefix(old_chapters)
                    .map(|relative| chapters.join(relative))
                    .unwrap_or_else(|_| old.to_path_buf())
            } else if old.starts_with(storage.root()) {
                old.to_path_buf()
            } else {
                continue;
            };
            *src = storage.relative_string(&new);
        }
        fs::write(&path, serde_json::to_vec(&blocks)?)?;
    }
    Ok(())
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if from.is_dir() {
            fs::create_dir_all(&to)?;
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
