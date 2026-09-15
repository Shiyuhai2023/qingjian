//! 一次码表导入的结果。

use std::path::PathBuf;

use super::report::CodeTableImportReport;

/// 一次码表导入的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeTableImport {
    /// 写出的 `.qj`。
    pub path: PathBuf,

    /// 码表名（Rime 头里的 `name:`，否则文件名主干）。
    pub name: String,

    /// 导入统计。
    pub report: CodeTableImportReport,
}
