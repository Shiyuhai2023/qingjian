//! 配置热加载记的状态。

use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use qingjian_platform::{AuxCodeConfig, DictionariesConfig};
use qingjian_predict::PredictConfig;

/// 热加载状态。
pub(crate) struct ConfigReload {
    /// `config.toml` 路径。
    pub(super) config_path: PathBuf,

    /// 上次看文件的时间（节流用）。
    pub(super) last_check: Instant,

    /// 随包领域词库目录。
    pub(super) bundled_dicts_dir: Option<PathBuf>,

    /// 随包辅码码表目录（随包根 `codes/`）。
    pub(super) bundled_codes_dir: Option<PathBuf>,

    /// 用户导入词库目录（`<用户目录>/dicts`）。
    pub(super) user_dicts_dir: Option<PathBuf>,

    /// 用户导入码表目录（`<用户目录>/codes`）。
    pub(super) user_codes_dir: Option<PathBuf>,

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
