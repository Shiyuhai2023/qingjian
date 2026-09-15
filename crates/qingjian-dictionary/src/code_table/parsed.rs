//! Rime 码表文本的解析：列位置、跨文件合表与导入统计。

use std::path::Path;

use super::table::is_valid_code;

/// Rime `.dict.yaml` 缺省的三列，与 Rime 一致。
const DEFAULT_COLUMNS: [&str; 3] = ["text", "code", "weight"];

/// 码表文件里的列位置：`columns` 列表按名字定下标。表里没有 `code` 列就是纯词表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Columns {
    /// 词在哪一列。
    pub text: usize,

    /// 码在哪一列；`None` 表示这是纯词表（`columns: [text, weight]`）。
    pub code: Option<usize>,
}

impl Default for Columns {
    fn default() -> Self {
        names_to_columns(&DEFAULT_COLUMNS.map(str::to_owned))
    }
}

/// 按 `columns` 列表里的名字定下标；列表里没有的名字就是 `None`。`text` 缺省在第 0 列。
fn names_to_columns(names: &[String]) -> Columns {
    let index_of = |want: &str| names.iter().position(|name| name == want);
    Columns {
        text: index_of("text").unwrap_or(0),
        code: index_of("code"),
    }
}

/// 一份码表文件（不含 `import_tables` 合进来的部分）解析出来的东西。
#[derive(Debug, Default)]
pub(super) struct ParsedTable {
    /// YAML 头里的 `name:`。
    pub name: Option<String>,

    /// YAML 头里的 `version:`。
    pub version: Option<String>,

    /// 要一并读入的其他码表（`import_tables` 的原样字符串，相对路径由调用方按主文件目录解析）。
    pub import_tables: Vec<String>,

    /// 列位置。
    pub columns: Columns,

    /// `(词, 码)` 对，码已经过校验。
    pub pairs: Vec<(String, String)>,

    /// 正文行数（不含注释与空行）。
    pub read: usize,

    /// 成功建成条目的行数。
    pub with_code: usize,

    /// 有词没有码的行数。
    pub no_code: usize,

    /// 码非法被丢掉的行数。
    pub skipped: usize,
}

impl ParsedTable {
    /// 解析一份码表文本：`---` … `...` 是 YAML 头（`name` / `version` / `columns` / `import_tables`），
    /// 正文每行按 `columns` 的列位置取词与码。没有头、直接 `词\t码` 的纯文本也认（按缺省列序）。
    pub fn parse(text: &str) -> Self {
        let mut parsed = Self::default();
        let mut names: Vec<String> = DEFAULT_COLUMNS
            .iter()
            .map(|name| (*name).to_owned())
            .collect();
        let mut in_header = false;
        let mut header_done = false;
        let mut pending_imports = false;
        let mut body: Vec<&str> = Vec::new();
        for raw in text.lines() {
            let line = raw.trim_end();
            let trimmed = line.trim();
            if !header_done {
                if trimmed == "---" {
                    in_header = true;
                    continue;
                }
                if trimmed == "..." {
                    header_done = true;
                    pending_imports = false;
                    continue;
                }
                // 头里（`---` 之后），或还没进头但这一行不像正文（没有制表符）
                if in_header || !line.contains('\t') {
                    in_header = true;
                    if pending_imports {
                        match trimmed.strip_prefix("- ") {
                            Some(item) => {
                                push_import(&mut parsed, item);
                                continue;
                            }
                            None => pending_imports = false,
                        }
                    }
                    if let Some((key, value)) = split_key(trimmed) {
                        match key {
                            "name" => parsed.name = Some(unquote(value)),
                            "version" => parsed.version = Some(unquote(value)),
                            "columns" => {
                                names = inline_list(value).unwrap_or_else(|| {
                                    value.split_whitespace().map(str::to_owned).collect()
                                })
                            }
                            "import_tables" => match inline_list(value) {
                                Some(items) => parsed.import_tables.extend(items),
                                None => pending_imports = true,
                            },
                            _ => {}
                        }
                    }
                    continue;
                }
                header_done = true;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            body.push(line);
        }
        parsed.columns = names_to_columns(&names);
        for line in body {
            parsed.push_body(line);
        }
        parsed
    }

    /// 正文一行：按列下标取词与码。码列里空格分开的多个码都收（Rime 的简码 / 全码写法）。
    fn push_body(&mut self, line: &str) {
        self.read += 1;
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let Some(word) = fields
            .get(self.columns.text)
            .copied()
            .filter(|word| !word.is_empty())
        else {
            self.skipped += 1;
            return;
        };
        let Some(code_index) = self.columns.code else {
            self.no_code += 1;
            return;
        };
        let field = fields.get(code_index).copied().unwrap_or_default();
        let mut produced = 0;
        for token in field.split_whitespace() {
            let code = token.to_ascii_lowercase();
            if is_valid_code(&code) {
                self.pairs.push((word.to_owned(), code));
                produced += 1;
            }
        }
        if produced > 0 {
            self.with_code += 1;
        } else if field.is_empty() {
            self.no_code += 1;
        } else {
            self.skipped += 1;
        }
    }
}

/// 追加一条 `import_tables`。
fn push_import(parsed: &mut ParsedTable, item: &str) {
    let item = unquote(item);
    if !item.is_empty() {
        parsed.import_tables.push(item);
    }
}

/// `key: value` 拆成两半；不是这个形状返回 `None`。
fn split_key(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    let key = key.trim();
    (!key.is_empty() && !key.contains(char::is_whitespace)).then_some((key, value.trim()))
}

/// `[a, b]` 形式的行内列表；不是这个形状返回 `None`。
fn inline_list(value: &str) -> Option<Vec<String>> {
    let body = value.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        body.split(',')
            .map(|item| unquote(item.trim()))
            .filter(|item| !item.is_empty())
            .collect(),
    )
}

/// 去掉 YAML 值两边的引号。
fn unquote(value: &str) -> String {
    value.trim().trim_matches(['\'', '"']).to_owned()
}

/// 码表文件的主干名：`stroke.dict.yaml` → `stroke`（与词库导入同一套规则）。
pub(super) fn stem_of(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = crate::import::strip_extensions(name);
    (!stem.is_empty()).then_some(stem)
}
