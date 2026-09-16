//! 「通用」页双拼下拉：选项从内置方案与 `shuangpin/` 目录现扫（见 [source]），选中「自定义…」时导入（见 [import]）。

mod choice;
mod import;
mod source;

pub(crate) use source::{choices, selected_index};

use crate::panel::Settings;

/// 下拉换选：普通方案直接落盘，末尾的「自定义…」去开文件选择器导入。
pub(crate) fn select(settings: &mut Settings, index: usize) {
    let choices = choices(settings);
    let Some(choice) = choices.get(index) else {
        // 越界（列表刚变过）：不改
        return;
    };
    if choice.opens_file_dialog() {
        import::run(settings);
        return;
    }
    // 「自定义…」在上面已经分流走，到这里的一定是可落盘的方案
    let key = choice.key().unwrap_or_default().to_owned();
    settings.save("general", "shuangpin", key);
}
