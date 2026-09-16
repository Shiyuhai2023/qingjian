//! Rime `.schema.yaml` 的最小解析：只取 speller 子集（`algebra` 投影链 + 两张字符表），其余忽略。

/// 从 schema yaml 里挖出来的 speller 子集。
pub(super) struct Speller {
    /// 方案名（`schema: name:`）。
    pub name: Option<String>,

    /// `alphabet` 字符表（解析留证；求值只用代数投影，不读它）。
    pub alphabet: Option<String>,

    /// `initials` 字符表（同上）。
    pub initials: Option<String>,

    /// `algebra` 投影规则原文（`xform/…/…/` 等）。
    pub rules: Vec<String>,
}

/// 顶层分节：只关心 `schema` 与 `speller`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Schema,
    Speller,
}

/// 解析 yaml 文本；不认识的顶层段（`recognizer` / `punctuator` / `key_binder`、`patch`、
/// `__include` / `__patch`）全部忽略——它们按 spec 卷 I 第 6 章本来就不在子集里。
pub(super) fn parse_speller(text: &str) -> Speller {
    let mut speller = Speller {
        name: None,
        alphabet: None,
        initials: None,
        rules: Vec::new(),
    };
    let mut section = None;
    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            section = match trimmed.strip_suffix(':') {
                Some("schema") => Some(Section::Schema),
                Some("speller") => Some(Section::Speller),
                _ => None,
            };
            continue;
        }
        match section {
            Some(Section::Schema) => {
                if let Some(value) = trimmed.strip_prefix("name:")
                    && speller.name.is_none()
                {
                    speller.name = Some(unquote(value.trim()));
                }
            }
            Some(Section::Speller) => {
                if let Some(value) = trimmed.strip_prefix("alphabet:") {
                    speller.alphabet = Some(unquote(value.trim()));
                } else if let Some(value) = trimmed.strip_prefix("initials:") {
                    speller.initials = Some(unquote(value.trim()));
                } else if let Some(item) = trimmed.strip_prefix("- ") {
                    speller.rules.push(item.trim().to_owned());
                }
            }
            None => {}
        }
    }
    speller
}

/// 去掉 YAML 值两边的引号。
fn unquote(value: &str) -> String {
    value.trim().trim_matches(['\'', '"']).to_owned()
}
