//! 自定义双拼：投影链求值、导入落盘与读回。

use std::path::{Path, PathBuf};

use super::ShuangpinTables;
use super::evaluator::Evaluator;
use super::report::ShuangpinImportReport;
use super::speller::parse_speller;
use super::{import_shuangpin, load_shuangpin};
use crate::error::DictionaryError;

/// 迷你方案：只覆盖测试要验的几条规则（含两条超出子集的）。
const MINI: &str = "\
# 迷你方案（测试用）
schema:
  schema_id: mini
  name: 迷你
speller:
  alphabet: zyxwvutsrqponmlkjihgfedcba
  initials: zyxwvutsrqponmlkjihgfedcba
  algebra:
    - xform/^zh([a-z]*)$/v$1/
    - xform/^([a-z]*)ong$/$1s/
    - xform/^ai$/ad/
    - xform/^an$/aj/
    - derive/^b(.*)$/p$1/
    - xform/^([a-z]*)an$/$1j/
    - xform/^([a-z]*)ing$/$1;/
    - xform/^([a-z])a$/$1a/
    - xlit/abc/def/
    - xform/^a$/b/,%/
";

/// (音节, 声母长度)：求值器要的那份输入（Core 的 `parser::SYLLABLES` 现拆）。
const SYLLABLES: [(&str, usize); 8] = [
    ("zhong", 2),
    ("ai", 0),
    ("an", 0),
    ("en", 0),
    ("ba", 1),
    ("ban", 1),
    ("xing", 1),
    ("nve", 1),
];

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("qingjian-shuangpin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 跑一遍迷你方案的求值。
fn evaluate(text: &str, syllables: &[(&str, usize)]) -> (ShuangpinTables, ShuangpinImportReport) {
    let speller = parse_speller(text);
    let mut report = ShuangpinImportReport::default();
    let evaluator = Evaluator::compile(&speller.rules, &mut report);
    let tables = evaluator.tables(syllables, &mut report);
    (tables, report)
}

#[test]
fn evaluates_two_key_codes_into_finals_and_zero_initials() {
    let (tables, report) = evaluate(MINI, &SYLLABLES);
    // 键按字符序，';' 在前
    assert_eq!(
        tables.finals,
        vec![
            (';', vec!["ing".to_owned()]),
            ('a', vec!["a".to_owned()]),
            ('j', vec!["an".to_owned()]),
            ('s', vec!["ong".to_owned()]),
        ]
    );
    assert_eq!(
        tables.zero_initials,
        vec![
            ("ai".to_owned(), vec!["ad".to_owned()]),
            ("an".to_owned(), vec!["aj".to_owned()]),
            ("en".to_owned(), vec!["en".to_owned()]),
        ]
    );
    assert!(tables.semicolon);
    // 8 条 xform / derive 认下，xlit 与带选项的规则各算一条「未支持」
    assert_eq!(report.rules, 8);
    assert_eq!(report.unsupported, 2);
    assert_eq!(report.finals, 4);
    assert_eq!(report.zero_initials, 3);
}

#[test]
fn derive_forks_and_duplicates_collapse() {
    // ban 经 derive 分出 ban / pan，两条都映到同一个韵母键 j、同一个韵母 an
    let (tables, _) = evaluate(MINI, &[("ban", 1)]);
    assert_eq!(tables.finals, vec![('j', vec!["an".to_owned()])]);
    assert!(tables.zero_initials.is_empty());
}

#[test]
fn abbrev_ends_the_chain_for_that_form() {
    // abbrev 产出后不再参与后面的规则：en 先被缩成 ef，后面的 an 规则不再碰它
    let text = "schema:\n  name: 缩\nspeller:\n  algebra:\n    - abbrev/^([a-z]*)en$/ef/\n    - xform/^ef$/zz/\n";
    let (tables, report) = evaluate(text, &[("en", 0), ("ben", 1)]);
    assert_eq!(report.rules, 2);
    assert_eq!(
        tables.zero_initials,
        vec![("en".to_owned(), vec!["ef".to_owned()])]
    );
    assert_eq!(tables.finals, vec![('f', vec!["en".to_owned()])]);
}

#[test]
fn unsupported_rules_are_skipped_not_fatal() {
    let text = "schema:\n  name: 杂\nspeller:\n  algebra:\n    - xform/^ai$/ad/\n    - erase/^x$/\n    - xlit/abc/def/\n";
    let (tables, report) = evaluate(text, &[("ai", 0)]);
    assert_eq!(report.rules, 1);
    assert_eq!(report.unsupported, 2);
    assert_eq!(
        tables.zero_initials,
        vec![("ai".to_owned(), vec!["ad".to_owned()])]
    );
}

#[test]
fn import_writes_the_tables_and_keeps_the_source_yaml() {
    let dir = TempDir::new("import");
    let source = dir.path().join("mini.schema.yaml");
    std::fs::write(&source, MINI).unwrap();
    let imported = import_shuangpin(&source, &dir.path().join("shuangpin"), &SYLLABLES).unwrap();
    assert_eq!(imported.name, "迷你");
    assert!(imported.path.ends_with("迷你.tsv"));
    assert!(dir.path().join("shuangpin/迷你.yaml").is_file());

    let tables = load_shuangpin(&imported.path).unwrap();
    let (expected, _) = evaluate(MINI, &SYLLABLES);
    assert_eq!(tables.finals, expected.finals);
    assert_eq!(tables.zero_initials, expected.zero_initials);
    assert_eq!(tables.name, "迷你");
    assert!(tables.semicolon);
}

#[test]
fn rejects_builtin_names_empty_schemes_and_missing_names() {
    let dir = TempDir::new("reject");
    let cases = [
        (
            "xiaohe",
            "schema:\n  name: xiaohe\nspeller:\n  algebra:\n    - xform/^ai$/ad/\n",
            true,
        ),
        (
            "noname",
            "schema:\n  schema_id: x\nspeller:\n  algebra:\n    - xform/^ai$/ad/\n",
            false,
        ),
        (
            "norule",
            "schema:\n  name: 空\nspeller:\n  algebra:\n    - erase/^x$/\n",
            false,
        ),
    ];
    for (file, text, conflict) in cases {
        let source = dir.path().join(format!("{file}.schema.yaml"));
        std::fs::write(&source, text).unwrap();
        let error = import_shuangpin(&source, &dir.path().join("out"), &SYLLABLES).unwrap_err();
        if conflict {
            assert!(
                matches!(error, DictionaryError::SchemeNameConflict(_)),
                "{file}: {error:?}"
            );
        } else {
            assert!(
                matches!(
                    error,
                    DictionaryError::NoSchemeName | DictionaryError::NoUsableRules
                ),
                "{file}: {error:?}"
            );
        }
    }
}

#[test]
fn tsv_round_trips() {
    let tables = ShuangpinTables {
        name: "手写".to_owned(),
        finals: vec![
            ('a', vec!["a".to_owned(), "ia".to_owned()]),
            (';', vec!["ing".to_owned()]),
        ],
        zero_initials: vec![("ai".to_owned(), vec!["ad".to_owned()])],
        semicolon: true,
    };
    let text = tables.to_tsv();
    let read = ShuangpinTables::from_tsv(&text).unwrap();
    assert_eq!(read, tables);
    assert!(matches!(
        ShuangpinTables::from_tsv("# 空表\n"),
        Err(DictionaryError::NoUsableRules)
    ));
}

/// 方案名里带路径字符时：导入与读取两侧按同一个规则拼文件名，不会「导得进去读不出来」。
#[test]
fn a_name_with_path_characters_still_loads_back() {
    let dir = TempDir::new("stem");
    let source = dir.path().join("slash.schema.yaml");
    std::fs::write(
        &source,
        "schema:\n  name: 我/的\nspeller:\n  algebra:\n    - xform/^ai$/ad/\n",
    )
    .unwrap();
    let imported = import_shuangpin(&source, &dir.path().join("shuangpin"), &SYLLABLES).unwrap();
    assert_eq!(imported.name, "我/的");
    assert!(imported.path.ends_with("我_的.tsv"), "{:?}", imported.path);
    // 读取侧（qingjian-platform 的 custom:<名字> 加载）按同一个 stem 拼路径
    let path = dir
        .path()
        .join("shuangpin")
        .join(format!("{}.tsv", super::file_stem("我/的")));
    assert_eq!(load_shuangpin(&path).unwrap().name, "我/的");
}
