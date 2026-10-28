//! 书架导出（T7.5）：仅含名称/作者/简介三键、2 空格缩进的 JSON（spec 5.1.1-12）。

use serde::Serialize;

use crate::domain::model::Book;
use crate::Result;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShelfBook {
    name: String,
    author: String,
    intro: String,
}

/// 导出书架 JSON（UTF-8，2 空格缩进）。
pub fn export_shelf(books: &[Book]) -> Result<String> {
    let items: Vec<ShelfBook> = books
        .iter()
        .map(|b| ShelfBook {
            name: b.name.clone(),
            author: b.author.clone(),
            intro: b.intro.clone().unwrap_or_default(),
        })
        .collect();
    let json = serde_json::to_string_pretty(&items)?;
    Ok(json)
}
