//! 文件与编码探测平台能力（M3 / T8.x 实现）。

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::{Error, Result};

#[derive(Debug, Default)]
pub struct FileSystem;

/// 只读取文件的一段字节，避免为了单章正文把整本 TXT 读进内存。
pub fn read_file_range(path: &Path, start: i64, end: i64) -> Result<Vec<u8>> {
    let mut file =
        File::open(path).map_err(|_| Error::Io(format!("文件读取失败: {}", path.display())))?;
    let len = file
        .metadata()
        .map_err(|e| Error::Io(format!("读取文件信息失败: {e}")))?
        .len();
    let start = start.max(0) as u64;
    let end = (end.max(0) as u64).min(len).max(start);
    if end <= start {
        return Ok(Vec::new());
    }
    file.seek(SeekFrom::Start(start))
        .map_err(|e| Error::Io(format!("定位文件失败: {e}")))?;
    let mut buffer = vec![0u8; (end - start) as usize];
    file.read_exact(&mut buffer)
        .map_err(|e| Error::Io(format!("文件读取失败: {e}")))?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_only_requested_range() {
        let path = std::env::temp_dir().join(format!(
            "local-reader-range-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, b"0123456789").unwrap();
        assert_eq!(read_file_range(&path, 2, 5).unwrap(), b"234");
        assert_eq!(read_file_range(&path, 8, 100).unwrap(), b"89");
        assert!(read_file_range(&path, 20, 30).unwrap().is_empty());
        let _ = fs::remove_file(path);
    }
}
