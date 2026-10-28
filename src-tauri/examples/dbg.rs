fn main() {
    let mut text = String::new();
    for i in 1..=5 {
        text.push_str(&format!("第{}章 标题{}\n", i, i));
        text.push_str(&"章节正文内容。".repeat(700));
    }
    for pat in [r"(?m)^第\d+章", r"(?m)^第\d+章.*$", r"(?m)^第\d+章[^\n]*"] {
        let re = fancy_regex::Regex::new(pat).unwrap();
        let hits: Vec<_> = re.find_iter(&text).filter_map(|m| m.ok()).collect();
        println!("pat={} hits={}", pat, hits.len());
    }
    // 长行单独测试。
    let two = "第1章 标题1\n".to_string()
        + &"内容。".repeat(700)
        + "\n第2章 标题2\n"
        + &"内容。".repeat(700);
    let re = fancy_regex::Regex::new(r"(?m)^第\d+章.*$").unwrap();
    let hits: Vec<_> = re.find_iter(&two).filter_map(|m| m.ok()).collect();
    println!(
        "two-chapter hits={} first_end={}",
        hits.len(),
        hits.first().map(|m| m.end()).unwrap_or(0)
    );
}
