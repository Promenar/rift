// 简体中文界面修改：本文件的显示文案或相关界面测试已本地化。
//! Searchable settings destinations, without constructing hidden page trees.
use super::Category::*;
use super::{CATEGORIES, Category};
use crate::common::config::LayoutMode;

/// A destination with its search text precomputed once.
pub(super) struct Entry {
    pub title: &'static str,
    pub location: String,
    pub page: Category,
    pub layout: Option<LayoutMode>,
    pub description: String,
    name: String,
    context: String,
    haystack: String,
}

#[derive(Clone, Copy)]
pub(super) enum Row {
    Heading(&'static str),
    Setting(&'static Entry),
}

impl Entry {
    pub fn target(&self) -> Option<super::layouts::Target> {
        use super::layouts::Target;
        match section(&self.location) {
            "间距" => Some(Target::Spacing),
            "显示器覆盖设置" => Some(Target::Displays),
            _ => self.layout.map(Target::Layout),
        }
    }

    fn score(&self, query: &str, terms: &[&str]) -> usize {
        let exact = if self.name == query {
            1000
        } else if self.name.starts_with(query) {
            600
        } else if self.name.contains(query) {
            400
        } else {
            0
        };
        exact
            + terms
                .iter()
                .map(|term| {
                    if self.name.contains(term) {
                        60
                    } else if self.context.contains(term) {
                        20
                    } else {
                        5
                    }
                })
                .sum::<usize>()
    }
}

pub(super) fn grouped(results: &[&'static Entry]) -> Vec<Row> {
    let mut rows = Vec::new();
    for &(page, heading, _) in &CATEGORIES {
        let mut matches = results.iter().filter(|entry| entry.page == page).peekable();
        if matches.peek().is_some() {
            rows.push(Row::Heading(heading));
            rows.extend(matches.map(|entry| Row::Setting(entry)));
        }
    }
    rows
}

struct Draft {
    title: &'static str,
    location: String,
    page: Category,
    layout: Option<LayoutMode>,
    key: &'static str,
    help: &'static str,
    aliases: String,
}

impl Draft {
    fn finish(self) -> Entry {
        let description = describe(&self);
        let name = self.title.to_lowercase();
        let context = self.location.to_lowercase();
        let category_alias = match self.page {
            General => "General",
            Layouts => "Layouts",
            Workspaces => "Workspaces",
            Rules => "Rules",
            Keyboard => "Keyboard",
            Mouse => "Mouse & Trackpad",
            MenuBar => "Menu Bar",
            Overview => "Overview",
            StackTabs => "Stack Tabs",
            Advanced => "Advanced",
            About => "About",
        };
        let layout_alias = match self.layout {
            Some(LayoutMode::Traditional) => "Traditional",
            Some(LayoutMode::Bsp) => "BSP",
            Some(LayoutMode::MasterStack) => "Master Stack",
            Some(LayoutMode::Scrolling) => "Scrolling",
            Some(LayoutMode::Stack) => "Stack",
            Some(LayoutMode::Floating) => "Floating",
            None => "",
        };
        let haystack = format!(
            "{name} {context} {category_alias} {layout_alias} {} {} {}",
            self.aliases, self.key, self.help
        )
        .to_lowercase();
        Entry {
            title: self.title,
            location: self.location,
            page: self.page,
            layout: self.layout,
            description,
            name,
            context,
            haystack,
        }
    }
}

fn describe(entry: &Draft) -> String {
    if !entry.help.is_empty() {
        return entry.help.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    }
    let text = match entry.title {
        "屏幕边缘" => "设置窗口与屏幕边缘之间的距离。",
        "虚拟工作区" => "将窗口组织到不同工作区。",
        "工作区轻扫" => "使用触控板手势切换工作区。",
        "新窗口位置" => "选择新打开窗口的出现位置。",
        "默认布局" => "选择默认使用的布局。",
        "应用规则" => "选择 Rift 如何处理应用程序窗口。",
        "工作区规则" => "将匹配的窗口分配到工作区。",
        "键盘快捷键" => "查看、添加和编辑键盘快捷键。",
        "显示器覆盖设置" => "为单台显示器自定间距和列宽。",
        "工作区" => "添加工作区并自定名称和布局。",
        _ => return entry.location.clone(),
    };
    if entry.layout.is_some() {
        let layout = entry.location.strip_prefix("布局 › ").unwrap_or(&entry.location);
        format!("{layout} · {text}")
    } else {
        text.into()
    }
}

/// Destinations without schema fields: pages, sections, and handcrafted editors.
const MANUAL: &[(Category, Option<LayoutMode>, &str, &str, &str)] = &[
    (
        Layouts,
        None,
        "间距",
        "屏幕边缘",
        "Screen edges gaps gap margin padding outer edges spacing",
    ),
    (
        Layouts,
        None,
        "间距",
        "分别设置各边缘",
        "Set each edge separately gaps gap top bottom left right margin padding outer",
    ),
    (
        Layouts,
        None,
        "间距",
        "水平",
        "Horizontal gaps gap between windows inner horizontal spacing",
    ),
    (
        Layouts,
        None,
        "间距",
        "垂直",
        "Vertical gaps gap between windows inner vertical spacing",
    ),
    (
        Layouts,
        None,
        "显示器覆盖设置",
        "显示器覆盖设置",
        "Display overrides monitor screen connected per display override gaps spacing widths add display uuid",
    ),
    (
        Workspaces,
        None,
        "工作区",
        "虚拟工作区",
        "Virtual Workspaces desktops spaces enabled",
    ),
    (
        Rules,
        None,
        "规则",
        "应用规则",
        "App rules application bundle identifier manage ignore floating window title match workspace assign desktop",
    ),
    (
        Keyboard,
        None,
        "键盘",
        "键盘快捷键",
        "Keyboard shortcuts keys hotkeys keybindings bindings commands",
    ),
    (
        Keyboard,
        None,
        "键盘",
        "快捷键集",
        "Shortcut set keymap keybinding mode default",
    ),
    (
        Keyboard,
        None,
        "键盘",
        "可复用修饰键组合",
        "Reusable modifier combinations command option control shift modifiers",
    ),
    (
        Mouse,
        None,
        "鼠标与触控板",
        "工作区轻扫",
        "Workspace swipes trackpad gestures fingers swipe sensitivity haptic",
    ),
    (
        Mouse,
        None,
        "鼠标与触控板",
        "滚动布局手势",
        "Scrolling layout gestures trackpad gestures fingers swipe scrolling niri",
    ),
    (
        Mouse,
        None,
        "鼠标与触控板",
        "拖放",
        "Drag & Drop mouse modifier move resize swap stack drop",
    ),
    (
        Mouse,
        None,
        "鼠标与触控板",
        "指针移动",
        "Pointer movement mouse cursor movement",
    ),
    (
        Overview,
        None,
        "总览",
        "总览",
        "Overview window previews fade transitions",
    ),
    (
        StackTabs,
        None,
        "堆叠标签",
        "堆叠标签",
        "Stack Tabs stackline indicator color position thickness interaction",
    ),
    (
        Advanced,
        None,
        "高级",
        "启动命令",
        "Startup commands launch run shell exec",
    ),
    (
        Advanced,
        None,
        "高级",
        "自动聚焦排除列表",
        "Autofocus blacklist focus exclude ignore app",
    ),
    (
        Advanced,
        None,
        "高级",
        "外部编辑后重新载入配置",
        "Reload config when edited externally configuration file automatic reload toml",
    ),
    (
        Advanced,
        None,
        "高级",
        "配置文件",
        "Configuration file open config path toml reload",
    ),
    (
        About,
        None,
        "关于",
        "检查更新…",
        "Check for Updates… version latest release update",
    ),
    (About, None, "关于", "文档", "Documentation docs help guide manual"),
    (
        About,
        None,
        "关于",
        "Matrix 聊天室",
        "Matrix Room matrix community chat discussion support",
    ),
    (
        About,
        None,
        "关于",
        "发行说明",
        "Release Notes changelog version changes",
    ),
    (
        About,
        None,
        "关于",
        "赞助 Rift",
        "Sponsor Rift donate support sponsorship",
    ),
];

// Schema registration supplies navigation context, not duplicate field lists.
// It is initialized once without constructing any hidden Settings views.
fn catalog() -> &'static [Entry] {
    use crate::common::config::*;
    static CATALOG: std::sync::OnceLock<Vec<Entry>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        fn append<T: ConfigSchema>(
            out: &mut Vec<Draft>,
            group: &str,
            page: Category,
            layout: Option<LayoutMode>,
            location: &str,
        ) {
            for field in T::fields().iter().filter(|field| field.group == group) {
                out.push(Draft {
                    title: field.title.split(" (").next().unwrap_or(field.title),
                    location: location.to_owned(),
                    page,
                    layout,
                    key: field.key,
                    help: field.help,
                    aliases: format!("{} {}", field.aliases, field.title),
                });
            }
        }
        let mut out = Vec::new();
        append::<Settings>(&mut out, "general", General, None, "通用 › 窗口行为");
        append::<Settings>(&mut out, "advanced", Advanced, None, "高级");
        append::<Settings>(&mut out, "pointer", Mouse, None, "鼠标与触控板 › 聚焦");
        append::<VirtualWorkspaceSettings>(&mut out, "", Workspaces, None, "工作区");
        append::<GestureSettings>(
            &mut out,
            "main",
            Mouse,
            None,
            "鼠标与触控板 › 工作区轻扫",
        );
        append::<GestureSettings>(
            &mut out,
            "advanced",
            Mouse,
            None,
            "鼠标与触控板 › 高级轻扫设置",
        );
        append::<ScrollingGestureSettings>(
            &mut out,
            "",
            Mouse,
            None,
            "鼠标与触控板 › 滚动布局手势",
        );
        append::<DragDropSettings>(&mut out, "", Mouse, None, "鼠标与触控板 › 拖放");
        append::<MenuBarSettings>(&mut out, "", MenuBar, None, "菜单栏");
        append::<MissionControlSettings>(&mut out, "", Overview, None, "总览");
        append::<StackLineSettings>(&mut out, "", StackTabs, None, "堆叠标签");
        append::<LayoutSettings>(&mut out, "", Layouts, None, "布局");
        append::<BaseLayoutSettings>(&mut out, "", Layouts, None, "布局 › 共享设置");
        for (title, layout) in super::pages::layouts() {
            if *layout != LayoutMode::Floating {
                append::<BaseLayoutSettings>(
                    &mut out,
                    "",
                    Layouts,
                    Some(*layout),
                    &format!("布局 › {title}"),
                );
            }
        }
        append::<TraditionalLayoutSettings>(
            &mut out,
            "",
            Layouts,
            Some(LayoutMode::Traditional),
            "布局 › 传统平铺",
        );
        append::<BspLayoutSettings>(&mut out, "", Layouts, Some(LayoutMode::Bsp), "布局 › 二分平铺");
        append::<StackSettings>(&mut out, "", Layouts, Some(LayoutMode::Stack), "布局 › 堆叠");
        append::<MasterStackSettings>(
            &mut out,
            "",
            Layouts,
            Some(LayoutMode::MasterStack),
            "布局 › 主从布局",
        );
        append::<ScrollingLayoutSettings>(
            &mut out,
            "",
            Layouts,
            Some(LayoutMode::Scrolling),
            "布局 › 滚动布局",
        );
        append::<OuterGaps>(&mut out, "", Layouts, None, "间距");
        append::<InnerGaps>(&mut out, "", Layouts, None, "间距");

        let pages = CATEGORIES.iter().map(|&(page, title, _)| (page, None, "设置", title, ""));
        let layouts = super::pages::layouts()
            .iter()
            .map(|&(title, layout)| (Layouts, Some(layout), "布局", title, "layout"));
        for (page, layout, location, title, aliases) in
            pages.chain(layouts).chain(MANUAL.iter().copied())
        {
            // A destination that names a documented field enriches it instead of repeating it.
            let fields: Vec<_> = out
                .iter()
                .enumerate()
                .filter(|(_, field)| {
                    field.title == title && field.page == page && field.layout == layout
                })
                .map(|(index, _)| index)
                .collect();
            let documented = fields
                .iter()
                .copied()
                .find(|&index| out[index].location == location)
                .or_else(|| (fields.len() == 1).then(|| fields[0]));
            match documented {
                Some(index) => {
                    let field = &mut out[index];
                    field.aliases.push(' ');
                    field.aliases.push_str(aliases);
                }
                None if fields.is_empty() => out.push(Draft {
                    title,
                    location: location.to_owned(),
                    page,
                    layout,
                    key: "",
                    help: "",
                    aliases: aliases.to_owned(),
                }),
                None => {}
            }
        }
        out.into_iter().map(Draft::finish).collect()
    })
}

pub(super) fn results(query: &str) -> Vec<&'static Entry> {
    let query = query.trim().to_lowercase();
    let terms: Vec<_> = query.split_whitespace().collect();
    if terms.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<_> = catalog()
        .iter()
        .filter(|entry| terms.iter().all(|term| entry.haystack.contains(term)))
        .map(|entry| (entry.score(&query, &terms), entry))
        .collect();
    hits.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(a.1.title.cmp(b.1.title))
            .then(a.1.location.cmp(&b.1.location))
    });
    hits.into_iter().map(|(_, entry)| entry).collect()
}

/// Scope repeated names such as “Enabled” to their documented section.
pub(super) fn section(location: &str) -> &str {
    location.rsplit(" › ").next().unwrap_or(location)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_finds_settings_and_their_layouts() {
        for (query, title, page, layout) in [
            (
                "默认列宽",
                "默认列宽",
                Layouts,
                Some(LayoutMode::Scrolling),
            ),
            (" FFM ", "焦点跟随指针", Mouse, None),
            ("窗口偏移", "窗口偏移", Layouts, Some(LayoutMode::Stack)),
            ("screen edges", "屏幕边缘", Layouts, None),
            ("per display override", "显示器覆盖设置", Layouts, None),
        ] {
            let found = results(query);
            let first = found.first().expect("find a setting, not just its section");
            assert_eq!((first.title, first.page, first.layout), (title, page, layout));
        }
        assert!(results("gaps").iter().any(|result| result.title == "屏幕边缘"));
        assert!(results("column width").iter().any(|result| {
            result.title == "默认列宽" && result.layout == Some(LayoutMode::Scrolling)
        }));
        assert!(results("Master Stack").iter().any(|result| {
            result.layout == Some(LayoutMode::MasterStack)
        }));
        assert!(
            results("column width unrelatedword").is_empty(),
            "match every query word"
        );
        assert!(results("   ").is_empty());
    }

    #[test]
    fn catalog_lists_each_destination_once() {
        let mut seen = std::collections::HashSet::new();
        for entry in catalog() {
            assert!(
                seen.insert((
                    entry.title,
                    &entry.location,
                    entry.page,
                    entry.layout.map(|layout| layout.config_choice_index())
                )),
                "duplicate destination {} in {}",
                entry.title,
                entry.location
            );
        }
    }
}
