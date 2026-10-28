//! 规则类实体：目录规则 / 替换净化 / 字典规则 / 在线语音引擎。
//!
//! 字段名与语义对齐 legado 实体；JSON 序列化为 camelCase。
//! 旧字段到新字段的映射（如 `regex`→`pattern`）在 `data::rule_io` 中处理。

use serde::{Deserialize, Serialize};

/// 目录规则（TxtTocRule）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TxtTocRule {
    /// 以时间戳为唯一标识。
    pub id: i64,
    pub name: String,
    /// 规则表达式（多行模式）。
    pub rule: String,
    #[serde(default)]
    pub example: String,
    /// 探测优先级，取值越大越先探测；默认规则集为连续序号。
    #[serde(default)]
    pub serial_number: i32,
    /// 启用开关。
    #[serde(default)]
    pub enable: bool,
}

/// 替换净化规则（ReplaceRule）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceRule {
    /// 以时间戳为唯一标识。
    pub id: i64,
    pub name: String,
    /// 逗号分隔的多组字符串。
    #[serde(default)]
    pub group: String,
    /// 替换内容表达式（正则或普通串）。
    pub pattern: String,
    /// 替换为内容。
    #[serde(default)]
    pub replacement: String,
    /// 作用范围（子串模糊匹配书名或书源名）。
    #[serde(default)]
    pub scope: String,
    /// 排除范围。
    #[serde(default)]
    pub exclude_scope: String,
    /// 作用于标题，默认关。
    #[serde(default)]
    pub scope_title: bool,
    /// 作用于正文，默认开。
    #[serde(default = "default_true")]
    pub scope_content: bool,
    /// 启用开关，默认开。
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    /// 是否为正则，默认开。
    #[serde(default = "default_true")]
    pub is_regex: bool,
    /// 超时毫秒；<=0 回退默认 3000。
    #[serde(default)]
    pub timeout_millisecond: i32,
    /// 排序值，最小值表示置顶。
    #[serde(default)]
    pub sort_order: i32,
}

impl Default for ReplaceRule {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            group: String::new(),
            pattern: String::new(),
            replacement: String::new(),
            scope: String::new(),
            exclude_scope: String::new(),
            scope_title: false,
            scope_content: true,
            is_enabled: true,
            is_regex: true,
            timeout_millisecond: 0,
            sort_order: 0,
        }
    }
}

impl ReplaceRule {
    /// 生效超时毫秒：<=0 时回退默认 3000（spec 6.7-6）。
    pub fn effective_timeout_ms(&self) -> i32 {
        if self.timeout_millisecond <= 0 {
            3000
        } else {
            self.timeout_millisecond
        }
    }
}

/// 字典规则（DictRule），以 name 为唯一标识。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DictRule {
    pub name: String,
    /// 请求规则（选词以占位符表示）。
    pub url_rule: String,
    /// 结果规则，可空。
    #[serde(default)]
    pub show_rule: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub sort_number: i32,
    #[serde(default)]
    pub js_lib: String,
}

/// 在线语音引擎（HttpTTS）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HttpTts {
    /// 以时间戳为唯一标识。
    pub id: i64,
    pub name: String,
    /// 合成 URL 模板，可含正文与语速等占位符。
    pub url: String,
    #[serde(default)]
    pub content_type: String,
    /// 并发限流值，"0" 表示不限。
    #[serde(default)]
    pub concurrent_rate: String,
    #[serde(default)]
    pub login_url: String,
    #[serde(default)]
    pub login_ui: String,
    #[serde(default)]
    pub header: String,
    #[serde(default)]
    pub js_lib: String,
    #[serde(default)]
    pub enabled_cookie_jar: bool,
    #[serde(default)]
    pub login_check_js: String,
    /// 最近更新时间，导入缺失时取当前时间。
    #[serde(default)]
    pub last_update_time: i64,
}

fn default_true() -> bool {
    true
}
