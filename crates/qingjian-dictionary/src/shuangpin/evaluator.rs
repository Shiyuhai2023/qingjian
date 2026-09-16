//! 投影链：把 Rime speller 的 `xform` / `derive` / `abbrev` 规则在导入期跑一遍，产出静态键位表。
//!
//! 语义照 librime 的 Projection：xform 匹配就改写、derive 匹配就分叉（原样与改写都继续）、
//! abbrev 匹配就产出并终结这条（不再参与后面的规则）。

use regex::Regex;

use super::report::ShuangpinImportReport;
use super::tables::ShuangpinTables;

/// 投影规则的类型（白名单：xform / derive / abbrev；其他一律「未支持」跳过）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuleKind {
    /// 匹配就改写，不匹配原样留下。
    Xform,

    /// 匹配就同时产出原样与改写两条，原样继续参与后面的规则。
    Derive,

    /// 匹配就产出改写并**终结**这条（不再参与后面的规则）。
    Abbrev,
}

/// 一条编译好的投影规则。
pub(super) struct SpellerRule {
    kind: RuleKind,
    pattern: Regex,
    replacement: String,
}

impl SpellerRule {
    /// 解析 `xform/<pattern>/<replacement>/`；多出字段（规则选项）或正则编不过都返回 `None`（未支持）。
    fn parse(text: &str) -> Option<Self> {
        let mut parts = text.splitn(4, '/');
        let kind = match parts.next()?.trim() {
            "xform" => RuleKind::Xform,
            "derive" => RuleKind::Derive,
            "abbrev" => RuleKind::Abbrev,
            _ => return None,
        };
        let pattern = parts.next()?;
        let replacement = parts.next()?.to_owned();
        // 第四个字段非空是规则选项（如 `xform/…/…/%`），超出子集
        if parts.next().is_some_and(|rest| !rest.is_empty()) {
            return None;
        }
        let pattern = Regex::new(pattern).ok()?;
        Some(Self {
            kind,
            pattern,
            replacement: normalize_replacement(&replacement),
        })
    }

    /// 匹配就返回改写后的串，不匹配返回 `None`。
    fn apply(&self, text: &str) -> Option<String> {
        self.pattern.is_match(text).then(|| {
            self.pattern
                .replace(text, self.replacement.as_str())
                .into_owned()
        })
    }
}

/// Rime 的替换串与 Rust `regex` 的语法基本一致，只差一处：Rust 把 `$1a` 当成组名 `1a`，
/// 而 Rime 的意思是「第一组 + 字面 a」。把 `$` 加数字一律写成 `$` 加花括号包住数字，再交给 `regex`。
fn normalize_replacement(replacement: &str) -> String {
    let chars: Vec<char> = replacement.chars().collect();
    let mut out = String::with_capacity(replacement.len() + 4);
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '$' && chars.get(index + 1).is_some_and(char::is_ascii_digit) {
            let start = index + 1;
            let mut end = start;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            out.push('$');
            out.push('{');
            out.extend(&chars[start..end]);
            out.push('}');
            index = end;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// 投影链。
pub(super) struct Evaluator {
    rules: Vec<SpellerRule>,
}

impl Evaluator {
    /// 编译规则链；编不过的逐条进「未支持」清单。
    pub(super) fn compile(rule_texts: &[String], report: &mut ShuangpinImportReport) -> Self {
        let mut rules = Vec::new();
        for text in rule_texts {
            match SpellerRule::parse(text) {
                Some(rule) => {
                    rules.push(rule);
                    report.rules += 1;
                }
                None => {
                    report.unsupported.push(text.clone());
                    tracing::warn!(%text, "双拼投影规则超出子集，跳过");
                }
            }
        }
        Self { rules }
    }

    /// 一个串跑完整条链的全部结果（去重）。
    pub(super) fn project(&self, input: &str) -> Vec<String> {
        let mut working = vec![input.to_owned()];
        let mut done: Vec<String> = Vec::new();
        for rule in &self.rules {
            let mut next = Vec::with_capacity(working.len() * 2);
            for text in working.drain(..) {
                let Some(replaced) = rule.apply(&text) else {
                    next.push(text);
                    continue;
                };
                match rule.kind {
                    RuleKind::Xform => next.push(replaced),
                    RuleKind::Derive => {
                        next.push(text);
                        next.push(replaced);
                    }
                    RuleKind::Abbrev => done.push(replaced),
                }
            }
            next.sort();
            next.dedup();
            working = next;
        }
        working.extend(done);
        working.sort();
        working.dedup();
        working
    }

    /// 全拼音节表 → 静态键位表。`syllables` 给 (音节, 声母长度)；声母长度 0 = 零声母。
    ///
    /// 只收两键码；两键里的第一键（声母）不读——自定义方案沿用内置的声母键（`v` / `i` / `u` 是 zh / ch / sh），
    /// 求值器只记第二键（韵母键）与零声母写法。带 `;` 的两键码把 `semicolon` 置真。
    pub(super) fn tables(
        &self,
        syllables: &[(&str, usize)],
        report: &mut ShuangpinImportReport,
    ) -> ShuangpinTables {
        let mut finals: Vec<(char, Vec<String>)> = Vec::new();
        let mut zero_initials: Vec<(String, Vec<String>)> = Vec::new();
        for &(syllable, initial_len) in syllables {
            for keys in self.project(syllable) {
                let keys: String = keys.chars().map(|c| c.to_ascii_lowercase()).collect();
                let chars: Vec<char> = keys.chars().collect();
                if chars.len() != 2 || !chars.iter().all(|c| c.is_ascii_lowercase() || *c == ';') {
                    continue;
                }
                if chars.contains(&';') {
                    report.semicolon = true;
                }
                if initial_len == 0 {
                    push_zero_initial(&mut zero_initials, syllable, &keys);
                } else {
                    push_final(&mut finals, chars[1], &syllable[initial_len..]);
                }
            }
        }
        finals.sort_by_key(|(key, _)| *key);
        zero_initials.sort_by(|a, b| a.0.cmp(&b.0));
        report.finals = finals.len();
        report.zero_initials = zero_initials.len();
        ShuangpinTables {
            name: String::new(),
            finals,
            zero_initials,
            semicolon: report.semicolon,
        }
    }
}

/// 韵母键表里追加一条（去重；先出现的优先级高）。
fn push_final(finals: &mut Vec<(char, Vec<String>)>, key: char, final_: &str) {
    match finals.iter_mut().find(|(k, _)| *k == key) {
        Some((_, list)) => {
            if !list.iter().any(|f| f == final_) {
                list.push(final_.to_owned());
            }
        }
        None => finals.push((key, vec![final_.to_owned()])),
    }
}

/// 零声母表里追加一条（去重）。
fn push_zero_initial(zero_initials: &mut Vec<(String, Vec<String>)>, syllable: &str, keys: &str) {
    match zero_initials.iter_mut().find(|(s, _)| s == syllable) {
        Some((_, list)) => {
            if !list.iter().any(|k| k == keys) {
                list.push(keys.to_owned());
            }
        }
        None => zero_initials.push((syllable.to_owned(), vec![keys.to_owned()])),
    }
}
