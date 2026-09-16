//! 配置热加载记的状态。

use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use qingjian_platform::{AuxCodeConfig, DictionariesConfig};
use qingjian_predict::PredictConfig;

/// 随包与用户数据目录：启动与热加载用的是同一批（词库、码表、自定义双拼）。
/// 分开传参数会越传越长，且热加载与原路径不一致时找不到文件。
#[derive(Debug, Clone, Default)]
pub struct DataDirs {
    /// 随包领域词库目录。
    pub bundled_dicts: Option<PathBuf>,

    /// 随包辅码码表目录（随包根 `codes/`）。
    pub bundled_codes: Option<PathBuf>,

    /// 用户导入词库目录（`<用户目录>/dicts`）。
    pub user_dicts: Option<PathBuf>,

    /// 用户导入码表目录（`<用户目录>/codes`）。
    pub user_codes: Option<PathBuf>,

    /// 自定义双拼产物目录（`<用户目录>/shuangpin`）。
    pub shuangpin: Option<PathBuf>,
}

/// 热加载状态。
pub(crate) struct ConfigReload {
    /// `config.toml` 路径。
    pub(super) config_path: PathBuf,

    /// 上次看文件的时间（节流用）。
    pub(super) last_check: Instant,

    /// 随包与用户数据目录。
    pub(super) dirs: DataDirs,

    /// 上次看到的码表目录 mtime：设置页刚导入一张表就靠它即时生效。
    pub(super) codes_mtime: Option<SystemTime>,

    /// 上次看到的 mtime。
    pub(super) last_mtime: Option<SystemTime>,

    /// 已应用的 `[predict]`。
    pub(super) applied_predict: PredictConfig,

    /// 已应用的 `[dictionaries]`。
    pub(super) applied_dictionaries: DictionariesConfig,

    /// 已应用的 `[aux_code]`。
    pub(super) applied_aux_code: AuxCodeConfig,
}
