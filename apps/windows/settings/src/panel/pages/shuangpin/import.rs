//! 选中「自定义…」后的导入：rfd 选 Rime `.schema.yaml` → 求值一次 → 落 `shuangpin/`，配置改成 `custom:<名字>`。

use qingjian_core::syllable_entries;
use qingjian_dictionary::{DictionaryError, import_shuangpin};

use super::source::custom_dir;
use crate::panel::Settings;

/// 开文件选择器选一份 Rime `.schema.yaml`，求值后放进自定义方案目录。
///
/// 成功：配置落成 `custom:<方案名>`（下拉随之选中它），统计进页面提示区。
/// 失败：只写底部红字一行，配置保持原样——原来选的方案继续用。
pub(crate) fn run(settings: &mut Settings) {
    let Some(source) = rfd::FileDialog::new()
        .add_filter("双拼方案", &["yaml", "yml"])
        .add_filter("所有文件", &["*"])
        .set_title("导入双拼方案")
        .pick_file()
    else {
        return;
    };
    match import_shuangpin(&source, &custom_dir(settings), &syllable_entries()) {
        Ok(imported) => {
            let report = imported.report;
            settings.save("general", "shuangpin", format!("custom:{}", imported.name));
            settings.notice.succeed(format!(
                "已导入双拼方案「{}」：规则 {} 条 · 未支持 {} 条 · 韵母键 {} 个 · 零声母 {} 个",
                imported.name,
                report.rules,
                report.unsupported,
                report.finals,
                report.zero_initials
            ));
        }
        Err(DictionaryError::NoSchemeName) => settings
            .notice
            .fail("这个文件里没有方案名：Rime 方案要在 schema 里写 name。".to_owned()),
        Err(DictionaryError::SchemeNameConflict(name)) => settings
            .notice
            .fail(format!("方案名「{name}」与内置方案同名，请改名后再导入。")),
        Err(DictionaryError::NoUsableRules) => settings
            .notice
            .fail("这个文件里没有可用的双拼规则。".to_owned()),
        Err(error) => settings.notice.fail(format!("导入双拼方案失败：{error}")),
    }
}
