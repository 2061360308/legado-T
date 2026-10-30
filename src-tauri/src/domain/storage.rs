use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use md5::{Digest, Md5};
use serde::Serialize;

use crate::domain::model::Book;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct BookStorage {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BookPaths {
    pub dir: PathBuf,
    pub source: PathBuf,
    pub chapters: PathBuf,
    pub images: PathBuf,
    pub managed: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSize {
    pub source: u64,
    pub chapters: u64,
    pub cover: u64,
    pub total: u64,
}

impl BookStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(self.root.join("books"))?;
        fs::create_dir_all(self.root.join("staging"))?;
        Ok(())
    }

    pub fn is_managed(&self, book: &Book) -> bool {
        book.source_path.is_some()
    }

    pub fn book_id<'a>(&self, book: &'a Book) -> &'a str {
        book.book_url
            .strip_prefix("local:")
            .unwrap_or(&book.book_url)
    }

    pub fn book_dir(&self, id: &str) -> PathBuf {
        let id = id.strip_prefix("local:").unwrap_or(id);
        let id = Path::new(id)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        self.root.join("books").join(id)
    }

    pub fn source_path_for(&self, id: &str, ext: &str) -> PathBuf {
        self.book_dir(id).join(format!("source.{ext}"))
    }

    pub fn paths_for_id(&self, id: &str, ext: &str) -> BookPaths {
        let dir = self.book_dir(id);
        let chapters = dir.join("chapters");
        BookPaths {
            source: dir.join(format!("source.{ext}")),
            images: chapters.join("images"),
            chapters,
            dir,
            managed: true,
        }
    }

    pub fn paths(&self, book: &Book) -> Result<BookPaths> {
        if let Some(relative) = book.source_path.as_deref() {
            let source = self.root.join(relative);
            let dir = source
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| Error::Invalid("invalid book source path".into()))?;
            let chapters = dir.join("chapters");
            return Ok(BookPaths {
                images: chapters.join("images"),
                chapters,
                source,
                dir,
                managed: true,
            });
        }

        let source = PathBuf::from(&book.book_url);
        if !source.is_absolute() {
            return Err(Error::Invalid(format!(
                "legacy book path is not absolute: {}",
                book.book_url
            )));
        }
        let dir = source
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone());
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("book");
        let chapters = dir.join(format!("{name}.chapters"));
        Ok(BookPaths {
            images: chapters.join("images"),
            chapters,
            source,
            dir,
            managed: false,
        })
    }

    pub fn create_book_dir(&self, id: &str, ext: &str) -> Result<BookPaths> {
        let paths = self.paths_for_id(id, ext);
        fs::create_dir_all(&paths.dir)?;
        fs::create_dir_all(&paths.chapters)?;
        fs::create_dir_all(&paths.images)?;
        Ok(paths)
    }

    pub fn copy_source(&self, src: &Path, id: &str, ext: &str) -> Result<PathBuf> {
        let paths = self.paths_for_id(id, ext);
        fs::create_dir_all(&paths.dir)?;
        fs::copy(src, &paths.source)?;
        Ok(paths.source)
    }

    pub fn staging_path(&self, ext: &str) -> Result<PathBuf> {
        let dir = self.root.join("staging");
        fs::create_dir_all(&dir)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0);
        for seq in 0..1000u32 {
            let path = dir.join(format!("{}-{}-{seq}.{ext}", std::process::id(), stamp));
            if !path.exists() {
                return Ok(path);
            }
        }
        Err(Error::Io("could not allocate staging path".into()))
    }

    pub fn adopt_staged(&self, staged: &Path, id: &str, ext: &str) -> Result<PathBuf> {
        let paths = self.paths_for_id(id, ext);
        fs::create_dir_all(&paths.dir)?;
        if paths.source.exists() {
            return Err(Error::Invalid(format!(
                "book source already exists: {}",
                id
            )));
        }
        match fs::rename(staged, &paths.source) {
            Ok(()) => Ok(paths.source),
            Err(_) => {
                fs::copy(staged, &paths.source)?;
                fs::remove_file(staged)?;
                Ok(paths.source)
            }
        }
    }

    pub fn remove_book_dir(&self, id: &str) {
        let _ = fs::remove_dir_all(self.book_dir(id));
    }

    pub fn clear_cache(&self, book: &Book) -> Result<()> {
        let paths = self.paths(book)?;
        if paths.chapters.exists() {
            fs::remove_dir_all(paths.chapters)?;
        }
        Ok(())
    }

    pub fn delete_managed(&self, book: &Book) -> Result<bool> {
        if !self.is_managed(book) {
            return Ok(false);
        }
        let dir = self.book_dir(self.book_id(book));
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(true)
    }

    pub fn cover_path_for_id(&self, id: &str, ext: &str) -> PathBuf {
        self.book_dir(id).join(format!("cover.{ext}"))
    }

    pub fn cover_path(&self, book: &Book, ext: &str) -> PathBuf {
        if self.is_managed(book) {
            return self.cover_path_for_id(self.book_id(book), ext);
        }
        let digest = Md5::digest(book.book_url.as_bytes());
        self.root.join("covers").join(format!("{digest:x}.{ext}"))
    }

    pub fn relative_string(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .map(|value| value.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().to_string())
    }

    pub fn hydrate_book(&self, book: &mut Book) {
        book.cover_url = book.cover_url.as_deref().map(|value| self.resolve(value));
        book.custom_cover_url = book
            .custom_cover_url
            .as_deref()
            .map(|value| self.resolve(value));
    }

    pub fn normalize_book(&self, book: &mut Book) {
        book.cover_url = book.cover_url.as_deref().map(|value| self.normalize(value));
        book.custom_cover_url = book
            .custom_cover_url
            .as_deref()
            .map(|value| self.normalize(value));
    }

    pub fn resolve(&self, value: &str) -> String {
        if is_external(value) || Path::new(value).is_absolute() {
            return value.to_string();
        }
        self.root.join(value).to_string_lossy().to_string()
    }

    pub fn normalize(&self, value: &str) -> String {
        let path = Path::new(value);
        if !path.is_absolute() {
            return value.to_string();
        }
        path.strip_prefix(&self.root)
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| value.to_string())
    }

    pub fn storage_size(&self, book: &Book) -> Result<StorageSize> {
        let paths = self.paths(book)?;
        let source = file_size(&paths.source);
        let chapters = dir_size(&paths.chapters);
        let mut covers = HashSet::new();
        for value in [&book.cover_url, &book.custom_cover_url]
            .into_iter()
            .flatten()
        {
            covers.insert(self.resolve(value));
        }
        let cover = covers.iter().map(|value| file_size(Path::new(value))).sum();
        Ok(StorageSize {
            source,
            chapters,
            cover,
            total: source + chapters + cover,
        })
    }

    pub fn hash_file(path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)
            .map_err(|_| Error::Io(format!("文件读取失败: {}", path.display())))?;
        let mut hasher = Md5::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn is_external(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("asset:")
        || lower.starts_with("blob:")
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                dir_size(&path)
            } else {
                file_size(&path)
            }
        })
        .sum()
}
