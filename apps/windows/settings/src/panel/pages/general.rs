//! 「通用」页：学习语言、每页候选数、双拼、英文模式候选。

use qingjian_platform::MAX_PAGE_SIZE;
use windows_reactor::*;

use super::shuangpin;
use crate::panel::controls::{feedback, field, index_of, page};
use crate::panel::{Message, Settings};

/// 学习语言：界面名 + 配置写法。
pub(crate) const LANGUAGES: [(&str, &str); 4] = [
    ("英语", "en"),
    ("日语", "ja"),
    ("西班牙语", "es"),
    ("不显示译文", "off"),
];

fn string_combo(
    options: &'static [(&str, &str)],
    current: &str,
    callback: Callback<Option<usize>>,
) -> ComboBox {
    ComboBox::new()
        .items_source(options.iter().map(|(label, _)| *label))
        .selected_index(index_of(options, current))
        .on_selection_changed(callback)
}

/// 双拼下拉：选项现扫（内置四套 + 导入过的自定义方案 +「自定义…」），按当前写法选中。
///
/// 控件 key 里带重建计数，换一次选就整条重建：WinUI 的选中项是控件自己的状态，
/// 只有重建才能保证「自定义…」导入成功（或取消）后界面停在配置里的真实值上。
fn shuangpin_combo(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let choices = shuangpin::choices(settings);
    let combo = ComboBox::new()
        .items_source(choices.iter().map(|choice| choice.label()))
        .selected_index(shuangpin::selected_index(
            &choices,
            &settings.config.general.shuangpin,
        ))
        .on_selection_changed(context.callback(Message::Shuangpin));
    StackPanel::new().keyed_children([KeyedView::new(
        format!("shuangpin-{}", settings.shuangpin_revision),
        combo,
    )])
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let g = &settings.config.general;
    let english_off = !settings.config.apps.english_candidates_off.is_empty();
    let rows = [
        field(
            "学习语言",
            "候选词右侧显示哪种语言的译词，只列出装了释义表的语言；「不显示译文」同时关掉生词标记与释义兜底。",
            string_combo(
                &LANGUAGES,
                &g.learning_language,
                context.callback(Message::LearningLanguage),
            ),
        ),
        field(
            "每页候选数",
            "",
            NumberBox::new()
                .minimum(1.0)
                .maximum(MAX_PAGE_SIZE as f64)
                .value(g.page_size as f64)
                .on_value_changed(context.callback(Message::PageSize)),
        ),
        field(
            "双拼",
            "开双拼后 v、u、i 是音节键，表达式与问字模式只能用 ? 开头进；微软、搜狗方案的 ; 键是 ing。选「自定义…」导入 Rime 的双拼方案（.schema.yaml），导入的方案按名字排在内置四套后面。\n方案文件的写法：每条规则把完整拼音改写成实际按键，替换串里的字母就是键——声母一条、韵母一条、零声母一条（只收两键一音节；完整方案每个声母韵母各写一条，现成的可取 Rime 社区 rime-double-pinyin）。示例里 zhong 敲 vs、ai 敲 ad：\nschema:\n  schema_id: my-scheme\n  name: 我的方案\nspeller:\n  algebra:\n    # $1 是括号里匹配到的部分，原样放回：换声母保留韵母（v$1），换韵母保留声母（$1s）\n    # 声母 zh 打 v：zhong 改写成 vong\n    - xform/^zh([a-z]*)$/v$1/\n    # 韵母 ong 打 s：vong 改写成 vs\n    - xform/^([a-z]*)ong$/$1s/\n    # 零声母 ai 打 ad\n    - xform/^ai$/ad/",
            shuangpin_combo(settings, context),
        ),
        field(
            "大千注音",
            "启用大千注音键盘布局（容错设定如 ㄢㄤ、ㄣㄥ 不分，请至「模糊音」分页开启）。",
            ToggleSwitch::new()
                .is_on(g.zhuyin)
                .on_toggled(context.callback(Message::Zhuyin)),
        ),
        field(
            "繁体输出",
            "打字时将候选词转换为繁体中文。",
            ToggleSwitch::new()
                .is_on(g.traditional)
                .on_toggled(context.callback(Message::Traditional)),
        ),
        field(
            "中文模式标点转全角",
            "没在打拼音时敲 , . ? ! 等出「，。？！」，数字后面的点保持半角；悬浮状态条的「，。」格也能切，切的是当前模式那份。",
            ToggleSwitch::new()
                .is_on(g.full_width_punctuation)
                .on_toggled(context.callback(Message::FullWidthPunctuation)),
        ),
        field(
            "英文模式标点转全角",
            "中英各记一份，缺省英文半角。",
            ToggleSwitch::new()
                .is_on(g.english_full_width_punctuation)
                .on_toggled(context.callback(Message::EnglishFullWidthPunctuation)),
        ),
        field(
            "英文模式（Caps Lock）也给候选",
            "Tab 或方向键选词；空格、回车、标点仍原样上屏敲的字母，不选词时与直接打字一样。",
            ToggleSwitch::new()
                .is_on(g.english_candidates)
                .on_toggled(context.callback(Message::EnglishCandidates)),
        ),
        field(
            "但在终端和代码编辑器里不给",
            "终端、Windows Terminal、VS Code、Cursor、JetBrains 等，那里的候选窗口会挡住应用自己的补全；名单可在配置文件里改。",
            ToggleSwitch::new()
                .is_on(english_off)
                .is_enabled(g.english_candidates)
                .on_toggled(context.callback(Message::EnglishOffInApps)),
        ),
        field(
            "输入拼音时中文候选排在英文词前面",
            "开着时整段输入是英文词时（hello、key）英文词排第二，空格上屏的仍是中文；关着（缺省）拼音不成立的输入英文词排第一。",
            ToggleSwitch::new()
                .is_on(g.chinese_first)
                .on_toggled(context.callback(Message::ChineseFirst)),
        ),
        feedback(&settings.notice),
    ];
    page("通用", StackPanel::new().spacing(16.0).children(rows))
}
