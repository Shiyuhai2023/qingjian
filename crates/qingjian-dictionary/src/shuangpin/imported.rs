use std::path::PathBuf;

use super::report::ShuangpinImportReport;

/// 一次自定义双拼导入的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuangpinImport {
    /// 写出的静态键位表（`shuangpin/<名字>.tsv`）。
    pub path: PathBuf,

    /// 方案名（配置里写 `custom:<名字>`）。
    pub name: String,

    /// 求值统计。
    pub report: ShuangpinImportReport,
}
