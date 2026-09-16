//! 各分节页：每个文件一个 `view(settings, context)`，页内常量（下拉选项等）与它同放。
//! `shuangpin/` 不单独成页，是「通用」页双拼下拉的数据源与导入。

pub(super) mod about;
pub(super) mod advanced;
pub(super) mod aux_code;
pub(super) mod candidates;
pub(super) mod cloud;
pub(super) mod dictionaries;
pub(super) mod fuzzy;
pub(super) mod general;
pub(super) mod shortcut;
pub(super) mod shuangpin;
pub(super) mod usage;
