use std::path::PathBuf;

/// `stroke` 子命令的参数。
pub struct StrokeOptions {
    /// CNS 筆順資料（`CNS_strokes_sequence.txt`）：`CNS 字碼\t[1-5]{n}`。
    pub cns_seq: PathBuf,

    /// CNS→Unicode 对照表（`CNS2UNICODE_Unicode*.txt`）：给文件或目录（目录取其中的对照表）。
    pub cns_map: Vec<PathBuf>,

    /// 官方筆畫數（`CNS_stroke.txt`）：与序列长度自洽的才留；不给就不过滤。
    pub cns_count: Option<PathBuf>,

    /// 自洽过滤的容差：序列长度与筆畫數之差超过它的字丢掉。
    pub max_diff: usize,

    /// 字表白名单（缺省通用规范字表）：只出表里的字，按表序排列。
    pub filter: PathBuf,

    /// 大陆序覆盖表（`assets/stroke/prc-rules.tsv`）。
    pub prc_rules: PathBuf,

    /// 产物路径（缺省 `<输出目录>/codes/stroke.tsv`）。
    pub output: Option<PathBuf>,

    /// 写完产物后按抽样对照表比对大陆笔画数，白名单之外一处不符即失败。
    pub verify: bool,

    /// 抽样用的字表（按表序每 `stride` 字取一个）。
    pub sample: PathBuf,

    /// 抽样密度：每几字取一个。
    pub stride: usize,

    /// 对照表（`字\t笔画数`，大陆规范），只覆盖抽样表。
    pub reference: PathBuf,

    /// 残留差异白名单（`字\t本表笔画数\t对照笔画数\t说明`）。
    pub whitelist: PathBuf,
}
