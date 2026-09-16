//! 用户导入的自定义双拼方案：导入期求值出的静态键位表（issue #8 卷 I 第 6 章）。

use super::table::DIGRAPH_INITIALS;
use crate::parser;

/// 用户导入的自定义双拼方案。
///
/// 键位表在导入期求值一次（`qingjian_dictionary` 的投影链），运行时不做代数求值、不碰正则；
/// 解码与内置四套方案共用同一套逻辑（见 [`super::Scheme`]），只是表数据是自有的。
/// 声母键沿用内置同款（`v` / `i` / `u` 是 zh / ch / sh），所以表里不存声母键。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomScheme {
    /// 方案名（yaml 的 `schema.name`；配置里写成 `custom:<名字>`）。
    pub name: String,

    /// 韵母键 → 该键的韵母候选（按优先级，与内置 `Table::finals` 同构）。
    pub finals: Vec<(char, Vec<String>)>,

    /// 零声母音节 → 两键写法（第一种是主写法，与内置 `Table::zero_initials` 同构）。
    pub zero_initials: Vec<(String, Vec<String>)>,

    /// 是否用到 `;` 键（某个韵母键是 `;`）。
    pub semicolon: bool,
}

impl CustomScheme {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 键 `key` 当韵母时的候选韵母（按优先级）。
    pub(super) fn finals(&self, key: char) -> Vec<&str> {
        self.finals
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(Vec::new(), |(_, finals)| {
                finals.iter().map(String::as_str).collect()
            })
    }

    /// 一个全拼音节的主写法（两个键）。测试与文档用。
    pub fn encode(&self, syllable: &str) -> Option<[char; 2]> {
        if let Some((_, spellings)) = self.zero_initials.iter().find(|(s, _)| s == syllable) {
            let mut chars = spellings[0].chars();
            return Some([chars.next()?, chars.next()?]);
        }
        let initial = parser::INITIALS
            .iter()
            .copied()
            .filter(|initial| syllable.starts_with(initial))
            .max_by_key(|initial| initial.len())?;
        let final_ = &syllable[initial.len()..];
        let second = self
            .finals
            .iter()
            .find(|(_, finals)| finals.iter().any(|candidate| candidate == final_))
            .map(|(key, _)| *key)?;
        let first = DIGRAPH_INITIALS
            .iter()
            .find(|(_, i)| *i == initial)
            .map(|(key, _)| *key)
            .or_else(|| initial.chars().next())?;
        Some([first, second])
    }
}
