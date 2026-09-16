//! 双拼下拉的一项：可落盘的方案，或末尾那个去开文件选择器的「自定义…」。

/// 末尾「自定义…」项的文案。
const IMPORT_LABEL: &str = "自定义…";

/// 双拼下拉里的一项。
#[derive(Debug)]
pub(crate) enum SchemeChoice {
    /// 可落盘的方案：界面名 + 配置写法（空串为全拼）。
    Scheme {
        /// 界面上显示的名字。
        label: String,

        /// 配置里的写法：内置键名 / `custom:<名字>` / 空串（全拼）。
        key: String,
    },

    /// 末尾的「自定义…」：选中它开文件选择器导入，本身不是方案、不落盘。
    Import,
}

impl SchemeChoice {
    /// 一个可落盘的方案项。
    pub(crate) fn scheme(label: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Scheme {
            label: label.into(),
            key: key.into(),
        }
    }

    /// 界面上的名字；「自定义…」是固定文案。
    pub(crate) fn label(&self) -> &str {
        match self {
            Self::Scheme { label, .. } => label,
            Self::Import => IMPORT_LABEL,
        }
    }

    /// 配置里的写法；「自定义…」没有（`None`）。
    pub(crate) fn key(&self) -> Option<&str> {
        match self {
            Self::Scheme { key, .. } => Some(key),
            Self::Import => None,
        }
    }

    /// 是不是末尾的「自定义…」：选中它要开文件选择器，不直接落盘。
    pub(crate) fn opens_file_dialog(&self) -> bool {
        matches!(self, Self::Import)
    }
}
