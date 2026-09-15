use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Dictionary(#[from] qingjian_dictionary::DictionaryError),

    #[error(transparent)]
    LanguageModel(#[from] qingjian_lm::LmError),

    #[error(transparent)]
    Glossary(#[from] qingjian_translate::GlossaryError),

    #[error(transparent)]
    Neural(#[from] qingjian_neural::NeuralError),

    /// 抽样对照出现白名单之外的不符。
    #[error("stroke verification failed: {unmatched} character(s) differ outside the whitelist")]
    Verify {
        /// 不符的字数。
        unmatched: usize,
    },

    /// 文件不是预期格式。
    #[error("{path}:{line}: {reason}")]
    Format {
        /// 出错的文件。
        path: PathBuf,

        /// 从 1 开始的行号。
        line: usize,

        /// 具体原因。
        reason: String,
    },
}
