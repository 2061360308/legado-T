use local_reader_lib::data::db;
use local_reader_lib::data::repositories::BookRepo;
use local_reader_lib::data::shelf_export::export_shelf;
use local_reader_lib::domain::bookshelf::{
    list_by_group, move_to_top, resolve_group_sort, sort_books, BookSort, VirtualGroup,
};
use local_reader_lib::domain::model::{book_type, Book};
use local_reader_lib::domain::pinyin::PinyinComparator;

fn book(url: &str, name: &str, author: &str, group: u64, dur_time: i64, order: i32) -> Book {
    Book {
        book_url: url.into(),
        name: name.into(),
        author: author.into(),
        group,
        dur_chapter_time: dur_time,
        order,
        r#type: book_type::TEXT | book_type::LOCAL,
        intro: Some("简介内容".into()),
        ..Default::default()
    }
}

#[test]
fn virtual_group_queries() {
    let conn = db::open_in_memory().unwrap();
    BookRepo::upsert(&conn, &book("/a.txt", "书A", "甲", 2, 100, 0)).unwrap();
    BookRepo::upsert(&conn, &book("/b.txt", "书B", "乙", 0, 300, 1)).unwrap();
    // 网络书（无 LOCAL 位）。
    let mut net_book = book("/c.txt", "书C", "丙", 0, 200, 2);
    net_book.r#type = book_type::TEXT;
    BookRepo::upsert(&conn, &net_book).unwrap();

    let all = list_by_group(&conn, VirtualGroup::All, BookSort::LastRead).unwrap();
    assert_eq!(all.len(), 3);
    // 「全部」按最近阅读降序。
    assert_eq!(all[0].name, "书B");

    let local = list_by_group(&conn, VirtualGroup::Local, BookSort::Manual).unwrap();
    assert_eq!(local.len(), 2);

    let ungrouped = list_by_group(&conn, VirtualGroup::LocalUngrouped, BookSort::Manual).unwrap();
    assert_eq!(ungrouped.len(), 1);
    assert_eq!(ungrouped[0].name, "书B");

    let g2 = list_by_group(&conn, VirtualGroup::User(2), BookSort::Manual).unwrap();
    assert_eq!(g2.len(), 1);
    assert_eq!(g2[0].name, "书A");
}

#[test]
fn six_sorts_and_pinyin() {
    let mut books = vec![
        book("/a.txt", "重生", "张三", 0, 100, 5),
        book("/b.txt", "长安", "李四", 0, 200, 3),
        book("/c.txt", "白夜", "王五", 0, 300, 1),
    ];
    sort_books(&mut books, BookSort::BookNamePinyin);
    let names: Vec<&str> = books.iter().map(|b| b.name.as_str()).collect();
    // 长安(chang) < 白夜(bai)?? 拼音序：bai < chang < zhong。
    assert_eq!(names, vec!["白夜", "长安", "重生"]);

    sort_books(&mut books, BookSort::LastRead);
    assert_eq!(books[0].name, "白夜");

    sort_books(&mut books, BookSort::Manual);
    assert_eq!(books[0].order, 1);

    sort_books(&mut books, BookSort::AuthorPinyin);
    // 拼音升序：li(李四) < wang(王五) < zhang(张三)。
    assert_eq!(books[0].author, "李四");
    assert_eq!(books[2].author, "张三");
}

#[test]
fn pinyin_comparator_direct() {
    let cmp = PinyinComparator;
    assert!(cmp.compare("白夜", "长安").is_lt());
    assert!(cmp.compare("长安", "重生").is_lt());
    assert!(cmp.compare("abc", "abd").is_lt());
}

#[test]
fn move_to_top_sets_min_order_minus_one() {
    let conn = db::open_in_memory().unwrap();
    BookRepo::upsert(&conn, &book("/a.txt", "书A", "甲", 0, 0, 0)).unwrap();
    BookRepo::upsert(&conn, &book("/b.txt", "书B", "乙", 0, 0, 2)).unwrap();
    move_to_top(&conn, "/b.txt").unwrap();
    let b = BookRepo::get(&conn, "/b.txt").unwrap().unwrap();
    assert_eq!(b.order, -1);
}

#[test]
fn group_sort_minus_one_falls_back_to_global() {
    assert_eq!(resolve_group_sort(-1, BookSort::Manual), BookSort::Manual);
    assert_eq!(
        resolve_group_sort(2, BookSort::Manual),
        BookSort::BookNamePinyin
    );
}

#[test]
fn shelf_export_has_three_keys_and_two_space_indent() {
    let books = vec![book("/a.txt", "书A", "甲", 0, 0, 0)];
    let json = export_shelf(&books).unwrap();
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"author\""));
    assert!(json.contains("\"intro\""));
    assert!(!json.contains("\"bookUrl\""));
    assert!(json.contains("\n  {"));
}
