//! 自定义双拼：Rime `.schema.yaml` 的 speller 子集 → 导入期求值一次 → 静态键位表（TSV 落盘）。
//!
//! 只解析 `algebra` 投影链（`xform` / `derive` / `abbrev` 白名单）与 `alphabet` / `initials` 两张字符表；
//! 求值在导入期完成，运行时不做代数求值、不碰正则。Core 不认识文件：它只拿 [`ShuangpinTables`]
//! 构造 `Scheme::Custom`（issue #8 卷 I 第 6 章）。

mod evaluator;
mod import;
mod imported;
mod report;
mod speller;
mod tables;

#[cfg(test)]
mod tests;

use std::path::Path;

pub use import::import_shuangpin;
pub use imported::ShuangpinImport;
pub use report::ShuangpinImportReport;
pub use tables::ShuangpinTables;

use crate::error::DictionaryError;

/// 读回导入的静态键位表（`shuangpin/<名字>.tsv`）。
pub fn load_shuangpin(path: &Path) -> Result<ShuangpinTables, DictionaryError> {
    ShuangpinTables::from_tsv(&std::fs::read_to_string(path)?)
}
