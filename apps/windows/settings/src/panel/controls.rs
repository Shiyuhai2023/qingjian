//! 各页共用的表单零件（标签行、说明小字、整项、页外壳）与打开文件 / 目录的小工具。

use std::path::{Path, PathBuf};

use windows_reactor::*;

use super::notice::Notice;
use super::{LABEL_WIDTH, Message, Settings};

/// 随包资源（相对随包根，如 `data/generated/dicts`），定位逻辑与 Server 共用。
pub(super) fn repo_resource(rel: &str) -> Option<PathBuf> {
    qingjian_platform::resources::bundled_resource(rel)
}

pub(super) fn open_in_editor(path: &Path) {
    if let Err(error) = std::process::Command::new("notepad").arg(path).spawn() {
        eprintln!("打开 {} 失败: {error}", path.display());
    }
}

/// 资源管理器打开目录或网址。
pub(super) fn open_with_explorer(target: &str) {
    if let Err(error) = std::process::Command::new("explorer").arg(target).spawn() {
        eprintln!("打开 {target} 失败: {error}");
    }
}

/// 一行设置：固定宽标签 + 控件。
pub(super) fn labeled(label: &str, control: impl Into<View>) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .children([
            TextBlock::new().text(label).width(LABEL_WIDTH).into(),
            control.into(),
        ])
}

/// 灰色小字说明，可换行。
pub(super) fn note(text: &str) -> View {
    TextBlock::new()
        .text(text)
        .text_wrapping(TextWrapping::Wrap)
        .font_size(12.0)
        .opacity(0.6)
        .into()
}

/// 一整项：「标签 + 控件」一行，下接说明（`hint` 为空则不加）。
pub(super) fn field(label: &str, hint: &str, control: impl Into<View>) -> View {
    let row = labeled(label, control);
    if hint.is_empty() {
        row
    } else {
        StackPanel::new().spacing(4.0).children([row, note(hint)])
    }
}

/// 在 `(界面名, 配置写法)` 列表里找 `value` 的下标，找不到取 0。
pub(super) fn index_of(options: &[(&str, &str)], value: &str) -> usize {
    options.iter().position(|(_, v)| *v == value).unwrap_or(0)
}

/// 一行「名称 · N 条 · 随包 / 许可证」，坏文件标出来：词库页与辅码页共用。
pub(super) fn entry_title(
    name: &str,
    entries: usize,
    license: &str,
    builtin: bool,
    broken: bool,
) -> String {
    if broken {
        return format!("{name}（文件损坏）");
    }
    let mut text = format!("{name} · {entries} 条");
    if builtin {
        text.push_str(" · 随包");
    } else if !license.is_empty() {
        text.push_str(&format!(" · {license}"));
    }
    text
}

/// 一行「复选框 + 可选的移除按钮」：词库页与辅码页共用，坏文件禁掉开关。
pub(super) fn check_row(
    stem: &str,
    label: String,
    enabled: bool,
    broken: bool,
    toggle: impl Fn(bool) -> Message + 'static,
    remove: Option<Message>,
    context: &mut ViewContext<Settings>,
) -> KeyedView {
    let check = CheckBox::new()
        .is_checked(enabled)
        .is_enabled(!broken)
        .on_is_checked_changed(context.callback(toggle))
        .content(label);
    let row = match remove {
        Some(message) => StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children((
                check,
                Button::new()
                    .on_click(context.message(message))
                    .content("移除"),
            )),
        None => check,
    };
    KeyedView::new(stem.to_owned(), row)
}

/// 页面底部的提示：成功统计一行灰字、失败一行红字，都没有就不画。
pub(super) fn feedback(notice: &Notice) -> View {
    let mut lines: Vec<KeyedView> = Vec::new();
    if let Some(text) = &notice.note {
        lines.push(KeyedView::new("notice-note", note(text)));
    }
    if let Some(text) = &notice.error {
        lines.push(KeyedView::new(
            "notice-error",
            TextBlock::new()
                .text(text)
                .text_wrapping(TextWrapping::Wrap)
                .font_size(12.0)
                .foreground(ThemeBrush::SystemCritical),
        ));
    }
    StackPanel::new().spacing(4.0).keyed_children(lines)
}

/// 一页外壳：可滚动 + 大标题 + 内容。
pub(super) fn page(title: &str, body: impl Into<View>) -> View {
    ScrollViewer::new().content(
        StackPanel::new().spacing(16.0).margin(24.0).children([
            TextBlock::new()
                .text(title)
                .font_size(24.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .into(),
            body.into(),
        ]),
    )
}
