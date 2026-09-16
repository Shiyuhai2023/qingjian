//! 「通用」页双拼下拉的数据源：全拼（关）、内置四套、`shuangpin/` 里导入过的自定义方案，末尾一项「自定义…」。

use std::path::{Path, PathBuf};

use qingjian_core::ShuangpinScheme;

use super::choice::SchemeChoice;
use crate::panel::Settings;

/// 自定义方案目录 `%APPDATA%\Qingjian\shuangpin`：导入的 `.tsv` 产物与原始 `.yaml` 都在这儿。
pub(crate) fn custom_dir(settings: &Settings) -> PathBuf {
    settings.data_dir().join("shuangpin")
}

/// 下拉的全部选项，顺序：全拼（关）、内置四套（照 [`ShuangpinScheme::ALL`]）、自定义（按名字排序）、「自定义…」。
///
/// 第一项「全拼」是关掉双拼用的，缺了它没法回到全拼；内置四套之后才是扫出来的自定义方案。
pub(crate) fn choices(settings: &Settings) -> Vec<SchemeChoice> {
    let mut choices = vec![SchemeChoice::scheme("全拼（不启用双拼）", "")];
    choices.extend(
        ShuangpinScheme::ALL
            .iter()
            .map(|scheme| SchemeChoice::scheme(scheme.label(), scheme.key())),
    );
    choices.extend(custom_choices(settings));
    choices.push(SchemeChoice::Import);
    choices
}

/// 扫 `shuangpin/` 目录里导入过的方案。
fn custom_choices(settings: &Settings) -> Vec<SchemeChoice> {
    scan(&custom_dir(settings))
}

/// 按名字排序列出 `dir` 里的方案；坏文件跳过并在日志里记一笔。
///
/// 名字一律走 [`qingjian_dictionary::load_shuangpin`] 读回来（与 Server 侧同一个读法），这里不解析 TSV。
fn scan(dir: &Path) -> Vec<SchemeChoice> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // 一份都没导入过时目录还不存在，不是错
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            eprintln!("读双拼方案目录 {} 失败: {error}", dir.display());
            return Vec::new();
        }
    };
    let mut schemes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "tsv") {
            continue;
        }
        match qingjian_dictionary::load_shuangpin(&path) {
            // 名字为空的手写 TSV 用不了（配置里写不出可用的 `custom:<名字>`），跳过
            Ok(tables) if tables.name.trim().is_empty() => {
                eprintln!("跳过没有方案名的双拼产物 {}", path.display());
            }
            // 配置里写的是 `custom:<导入返回的 name>`，所以这里也用 name 当选中依据
            Ok(tables) => schemes.push(SchemeChoice::scheme(
                tables.name.clone(),
                format!("custom:{}", tables.name),
            )),
            Err(error) => eprintln!("跳过读不了的双拼方案 {}: {error}", path.display()),
        }
    }
    schemes.sort_by(|left, right| left.label().cmp(right.label()));
    schemes
}

/// 配置写法在选项里的下标，找不到取 0（也就是「全拼（不启用双拼）」）。
///
/// `custom:<名字>` 指向的产物被删掉时也走这条：Server 侧读不到同样按全拼跑
/// （`GeneralConfig::shuangpin_with` 的告警语义），下拉如实回到「全拼（不启用双拼）」，
/// 配置本身不动——文件放回来重开这一页就恢复选中。
pub(crate) fn selected_index(choices: &[SchemeChoice], value: &str) -> usize {
    choices
        .iter()
        .position(|choice| choice.key() == Some(value))
        .unwrap_or(0)
}
#[cfg(test)]
mod tests {
    use super::*;
    use qingjian_dictionary::ShuangpinTables;

    /// 一张最小可用表，只为在扫描里带出方案名。
    fn tables(name: &str) -> ShuangpinTables {
        ShuangpinTables {
            name: name.to_owned(),
            finals: vec![('d', vec!["ai".to_owned()])],
            zero_initials: Vec::new(),
            semicolon: false,
        }
    }

    #[test]
    fn scan_sorts_by_name_and_skips_broken_files() {
        let dir = std::env::temp_dir().join(format!(
            "qingjian-settings-shuangpin-scan-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ziranma.tsv"), tables("zeta").to_tsv()).unwrap();
        std::fs::write(dir.join("xiaohe.tsv"), tables("alpha").to_tsv()).unwrap();
        std::fs::write(dir.join("broken.tsv"), "这不是键位表").unwrap();
        std::fs::write(dir.join("no-name.tsv"), "# semicolon: false\nd\tai\n").unwrap();
        // 原始 yaml 与方案目录同放，不算方案
        std::fs::write(dir.join("alpha.yaml"), "原样保留的来源").unwrap();

        let scanned = scan(&dir);
        let labels: Vec<&str> = scanned.iter().map(|choice| choice.label()).collect();
        assert_eq!(labels, ["alpha", "zeta"]);
        assert_eq!(scanned[0].key(), Some("custom:alpha"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_of_a_missing_directory_is_empty() {
        let dir = std::env::temp_dir().join("qingjian-settings-shuangpin-absent-目录");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(scan(&dir).is_empty());
    }

    #[test]
    fn selection_falls_back_to_full_pinyin() {
        let choices = vec![
            SchemeChoice::scheme("全拼（不启用双拼）", ""),
            SchemeChoice::scheme("小鹤双拼", "xiaohe"),
            SchemeChoice::scheme("alpha", "custom:alpha"),
            SchemeChoice::Import,
        ];
        assert_eq!(selected_index(&choices, ""), 0);
        assert_eq!(selected_index(&choices, "xiaohe"), 1);
        assert_eq!(selected_index(&choices, "custom:alpha"), 2);
        // 「自定义…」不落盘，配置里永远不会是它
        assert_eq!(selected_index(&choices, "自定义…"), 0);
        // 产物被删掉的自定义方案退回「关」
        assert_eq!(selected_index(&choices, "custom:没了"), 0);
    }
}
