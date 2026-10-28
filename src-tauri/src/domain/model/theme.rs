//! 主题与阅读排版方案实体。

use serde::{Deserialize, Serialize};

/// 全局主题配置（以 themeName 为主键，同名覆盖）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub theme_name: String,
    #[serde(default)]
    pub is_night_theme: bool,
    pub primary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub bottom_background: String,
}

/// 背景取值类型（spec 6.12-2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BgKind {
    /// 颜色值。
    Color,
    /// 内置资源路径。
    Builtin,
    /// 外部图片路径。
    External,
}

impl Default for BgKind {
    fn default() -> Self {
        BgKind::Color
    }
}

/// 单模式配色。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModeColors {
    pub bg_kind: BgKind,
    pub bg_value: String,
    pub text_color: String,
    /// 状态栏图标明暗（true 为深色图标）。
    #[serde(default)]
    pub dark_status_icon: bool,
}

/// 阅读排版方案（日/夜/墨水屏三模式）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadStyleScheme {
    pub name: String,
    pub day: ModeColors,
    pub night: ModeColors,
    pub eink: ModeColors,
}

/// 预置排版方案数量（前 5 套不可删除）。
pub const BUILTIN_SCHEME_COUNT: usize = 5;

impl ReadStyleScheme {
    /// 是否可删除（前 5 套预置不可删，第 6 套起可删，spec 6.12-1）。
    pub fn is_deletable(index: usize) -> bool {
        index >= BUILTIN_SCHEME_COUNT
    }
}
