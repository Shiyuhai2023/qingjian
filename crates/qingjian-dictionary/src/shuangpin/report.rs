//! 一次自定义双拼导入的统计。

/// 一次自定义双拼导入的统计。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShuangpinImportReport {
    /// 认出的投影规则数（xform / derive / abbrev）。
    pub rules: usize,

    /// 超出子集被跳过的规则原文（recognizer 类算子、编不过的正则、带选项的规则）：逐条留证，
    /// 设置页据此告诉用户「哪些规则没生效」，不是只给一个数。
    pub unsupported: Vec<String>,

    /// 求出的韵母键个数。
    pub finals: usize,

    /// 求出的零声母音节个数。
    pub zero_initials: usize,

    /// 是否用到 `;` 键。
    pub semicolon: bool,
}
