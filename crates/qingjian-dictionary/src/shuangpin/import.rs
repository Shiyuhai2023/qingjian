//! 导入自定义双拼：Rime `.schema.yaml` → 求值一次 → TSV 产物，原始 yaml 一并保留（来源留证）。

use std::path::Path;

use super::evaluator::Evaluator;
use super::imported::ShuangpinImport;
use super::report::ShuangpinImportReport;
use super::speller::parse_speller;
use crate::error::DictionaryError;

/// 内置四套方案的名字：与它们同名的自定义方案拒绝导入（配置里 `custom:<名字>` 会与内置写法撞车）。
const BUILTIN_NAMES: [&str; 4] = ["xiaohe", "ziranma", "microsoft", "sogou"];

/// 把 `source`（Rime `.schema.yaml`）求值后导入到 `dest_dir`（用户数据目录的 `shuangpin/`）。
///
/// `syllables` 是全拼音节表（(音节, 声母长度)），由调用方给 Core 的 `parser::SYLLABLES` 现拆——
/// 求值器在 dictionary 这一层，不反过来依赖 Core。同名再导覆盖。
pub fn import_shuangpin(
    source: &Path,
    dest_dir: &Path,
    syllables: &[(&str, usize)],
) -> Result<ShuangpinImport, DictionaryError> {
    let text = std::fs::read_to_string(source)?;
    let speller = parse_speller(&text);
    let mut report = ShuangpinImportReport::default();
    let evaluator = Evaluator::compile(&speller.rules, &mut report);
    let mut tables = evaluator.tables(syllables, &mut report);
    let name = speller.name.unwrap_or_default();
    if name.is_empty() {
        return Err(DictionaryError::NoSchemeName);
    }
    let key = name.to_ascii_lowercase();
    if BUILTIN_NAMES.contains(&key.as_str()) {
        return Err(DictionaryError::SchemeNameConflict(name));
    }
    // 一条规则都没认下（或认下的规则一条表都没求出来）时拒绝：否则「原样通过」会把
    // 任何两字母音节当成合法码，导出一个看着能用、实际是恒等映射的假表
    if report.rules == 0 || tables.is_empty() {
        return Err(DictionaryError::NoUsableRules);
    }
    tables.name = name.clone();
    let stem = super::file_stem(&name);
    std::fs::create_dir_all(dest_dir)?;
    let target = dest_dir.join(format!("{stem}.tsv"));
    std::fs::write(&target, tables.to_tsv())?;
    // 原始 yaml 一并保留：求值产物错了能回来查来源
    std::fs::write(dest_dir.join(format!("{stem}.yaml")), &text)?;
    Ok(ShuangpinImport {
        path: target,
        name,
        report,
    })
}
