//! 静态键位表：求值器的产物、TSV 落盘与读回。

use crate::error::DictionaryError;

/// TSV 里的分节：韵母表 / 零声母表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Finals,
    ZeroInitials,
}

/// 自定义双拼方案求值出的静态键位表（issue #8 卷 I 第 6 章）。
///
/// 与内置方案的 `Table` 同构（韵母键 → 韵母、零声母 → 两键写法），只是自己拥有数据；
/// Core 把它包进 `Scheme::Custom`，解码走同一套逻辑。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuangpinTables {
    /// 方案名（yaml 的 `schema.name`；配置里写成 `custom:<名字>`）。
    pub name: String,

    /// 韵母键 → 该键的韵母候选（按优先级）。
    pub finals: Vec<(char, Vec<String>)>,

    /// 零声母音节 → 两键写法（第一种是主写法）。
    pub zero_initials: Vec<(String, Vec<String>)>,

    /// 是否用到 `;` 键（某个韵母键是 `;`）。
    pub semicolon: bool,
}

impl ShuangpinTables {
    /// 一张表都没求出来：导入方据此拒绝。
    pub fn is_empty(&self) -> bool {
        self.finals.is_empty() && self.zero_initials.is_empty()
    }

    /// TSV 文本（落盘形式）：元数据头注释 + 两张表。
    pub fn to_tsv(&self) -> String {
        let mut out = String::from(
            "# 青简自定义双拼：静态键位表（导入期求值一次；原始 yaml 保留在 shuangpin/<名字>.yaml）\n",
        );
        out.push_str(&format!("# name: {}\n", self.name));
        out.push_str(&format!("# semicolon: {}\n", self.semicolon));
        out.push_str("# finals：键位→韵母（空格分隔，按优先级）\n");
        for (key, finals) in &self.finals {
            out.push_str(&format!("{key}\t{}\n", finals.join(" ")));
        }
        out.push_str("# zero_initials：零声母全拼→两键写法（空格分隔）\n");
        for (syllable, spellings) in &self.zero_initials {
            out.push_str(&format!("{syllable}\t{}\n", spellings.join(" ")));
        }
        out
    }

    /// 从 TSV 文本读回。
    pub fn from_tsv(text: &str) -> Result<Self, DictionaryError> {
        let mut tables = Self {
            name: String::new(),
            finals: Vec::new(),
            zero_initials: Vec::new(),
            semicolon: false,
        };
        let mut section: Option<Section> = None;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix('#') {
                let rest = rest.trim();
                if let Some(value) = rest.strip_prefix("name:") {
                    tables.name = value.trim().to_owned();
                } else if let Some(value) = rest.strip_prefix("semicolon:") {
                    tables.semicolon = value.trim() == "true";
                } else if rest.contains("finals") {
                    section = Some(Section::Finals);
                } else if rest.contains("zero_initials") {
                    section = Some(Section::ZeroInitials);
                }
                continue;
            }
            let Some((key, values)) = line.split_once('\t') else {
                continue;
            };
            let values: Vec<String> = values.split_whitespace().map(str::to_owned).collect();
            if values.is_empty() {
                continue;
            }
            match section {
                Some(Section::Finals) => {
                    if let Some(key) = key.chars().next() {
                        tables.finals.push((key, values));
                    }
                }
                Some(Section::ZeroInitials) => tables.zero_initials.push((key.to_owned(), values)),
                None => {}
            }
        }
        if tables.is_empty() {
            return Err(DictionaryError::NoUsableRules);
        }
        Ok(tables)
    }
}
