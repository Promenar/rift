use std::collections::BTreeMap;
use std::path::PathBuf;

use regex::RegexBuilder;
pub use rift_protocol::{AnimationEasing, ConfigCommand, LayoutMode, WorkspaceSelector};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{ConfigEnum, ConfigSchema};
use crate::actor::wm_controller::WmCommand;
use crate::common::collections::HashMap;
use crate::sys::hotkey::{Hotkey, HotkeySpec, Modifiers, modifiers_from_str};

pub const MAX_WORKSPACES: usize = 128;

// TODO: when to remove these?
pub(super) const DEPRECATED_MAP: &[(&str, &str)] = &[
    ("stack_windows", "toggle_stack"),
    ("unstack_windows", "toggle_stack"),
    ("toggle_tile_orientation", "toggle_orientation"),
];

pub fn data_dir() -> PathBuf { dirs::home_dir().unwrap().join(".rift") }
pub fn restore_file() -> PathBuf { data_dir().join("layout.ron") }
pub fn config_file() -> PathBuf {
    dirs::home_dir().unwrap().join(".config").join("rift").join("config.toml")
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Eq, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMode {
    /// 每个 macOS 桌面拥有独立编号的工作区集合。
    #[default]
    #[setting(label = "各显示器独立")]
    PerDisplay,
    /// 所有显示器共用一组工作区。每个工作区同一时间只属于一台显示器。
    #[setting(label = "全局")]
    Global,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct VirtualWorkspaceSettings {
    #[serde(default)]
    /// 每台显示器各自拥有从 1 开始编号的工作区。关闭后，所有显示器共用连续编号，
    /// 且每个工作区固定在一台显示器上。
    #[setting(
        label = "各显示器使用独立工作区",
        aliases = "workspace mode global per display multiple monitors spaces",
        custom
    )]
    pub mode: WorkspaceMode,
    #[serde(default = "yes")]
    /// 将窗口组织到独立的虚拟工作区。
    #[setting(ignore)]
    pub enabled: bool,
    #[serde(default = "default_workspace_count")]
    /// 虚拟工作区的数量。
    #[setting(
        label = "工作区数量",
        custom,
        aliases = "workspace count number desktops spaces"
    )]
    pub default_workspace_count: usize,
    #[serde(default = "yes")]
    /// 自动将新窗口分配到工作区。
    // #[setting(
    //     label = "Auto-assign windows",
    //     aliases = "automatically assign apps workspaces"
    // )]
    #[setting(ignore)]
    pub auto_assign_windows: bool,
    #[serde(default = "yes")]
    /// 记住各工作区中获得焦点的窗口。
    #[setting(label = "记住工作区焦点", aliases = "remember active focus")]
    pub preserve_focus_per_workspace: bool,
    #[serde(default)]
    /// 再次选择当前工作区时返回上一工作区。
    #[setting(
        label = "再次选择当前工作区时返回上一工作区",
        aliases = "back and forth previous desktop"
    )]
    pub workspace_auto_back_and_forth: bool,
    #[serde(default, alias = "prevent_wrapping_around")]
    /// 切换到第一个或最后一个工作区时停止，不循环跳转。
    #[setting(label = "禁用循环切换", aliases = "cycle last first workspace")]
    pub prevent_wrapping: bool,
    #[serde(default = "default_workspace_names")]
    /// 为虚拟工作区指定的名称。
    #[setting(
        label = "工作区",
        custom,
        aliases = "names rename layout desktop space count number add remove"
    )]
    pub workspace_names: Vec<String>,
    #[serde(default)]
    /// 未被其它规则分配的新窗口使用此工作区。
    #[setting(label = "默认工作区", custom, aliases = "startup desktop space")]
    pub default_workspace: usize,
    #[serde(default)]
    /// 窗口标题变化时重新检查工作区规则。
    #[setting(
        label = "标题变化时重新应用规则",
        aliases = "application title updated match workspace"
    )]
    pub reapply_app_rules_on_title_change: bool,
    #[serde(default)]
    /// 将匹配的应用和窗口分配到工作区。
    #[setting(label = "应用规则", custom)]
    pub app_rules: Vec<AppWorkspaceRule>,
    #[serde(default)]
    /// 为单个工作区设置独立布局和有序的显示器偏好。
    #[setting(label = "工作区规则", custom)]
    pub workspace_rules: Vec<WorkspaceLayoutRule>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceLayoutRule {
    pub workspace: WorkspaceSelector,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutMode>,
    /// 按优先级排列的显示器 UUID、名称，或 builtin/left/right/top/bottom 选择器。
    /// 第一个可用偏好生效；仅在 `mode = "global"` 时适用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<serde_with::OneOrMany<_>>")]
    pub display: Option<Vec<String>>,
}

// Allow specifying a workspace by numeric index or by name in the config.
// This supports both `workspace = 2` and `workspace = "coding"` in app rules.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct AppWorkspaceRule {
    /// 应用 Bundle 标识符，例如 "com.apple.Terminal"。
    #[setting(custom)]
    pub app_id: Option<String>,
    /// 目标工作区索引（从 0 开始）或工作区名称。未指定时窗口进入当前工作区。
    #[setting(custom)]
    pub workspace: Option<WorkspaceSelector>,
    #[serde(default)]
    #[setting(custom)]
    pub floating: bool,
    /// 浮动窗口的初始归一化位置。可用屏幕区域左上角为 `(0, 0)`，
    /// 右下角为 `(1, 1)`。
    #[setting(custom)]
    pub position: Option<AppRulePosition>,
    /// 首选窗口尺寸，以逻辑像素为单位。
    #[setting(custom)]
    pub size: Option<AppRuleSize>,
    /// 应用此规则后聚焦窗口，必要时切换虚拟工作区。
    #[serde(default)]
    #[setting(custom)]
    pub focus: bool,
    /// 显式管理覆盖：`false` 使窗口对 Rift 不可见；`true` 对可见窗口强制启用管理。
    /// 省略时保留 Rift 的正常可管理性判断。
    #[serde(default)]
    #[setting(custom)]
    pub manage: Option<bool>,
    #[setting(custom)]
    pub app_name: Option<String>,
    /// 可选的窗口标题正则表达式，用于匹配 window.title。
    ///
    /// 存在时优先作为标题匹配条件使用。
    #[setting(custom)]
    pub title_regex: Option<String>,
    /// 可选的窗口标题子字符串，用于匹配 window.title。
    ///
    /// 存在时按子串匹配处理，便于不想写完整正则的用户。
    #[setting(custom)]
    pub title_substring: Option<String>,

    /// 可选：要匹配的无障碍角色（AXRole）。存在时必须为非空字符串，
    /// 并与 AX API 报告的窗口角色精确比较。
    #[setting(custom)]
    pub ax_role: Option<String>,

    /// 无障碍子角色，必须非空，并与 AX API 报告的窗口子角色精确比较。
    #[setting(custom)]
    pub ax_subrole: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct AppRulePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct AppRuleSize {
    pub w: Option<f64>,
    pub h: Option<f64>,
}

impl Default for VirtualWorkspaceSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: WorkspaceMode::PerDisplay,
            default_workspace_count: default_workspace_count(),
            auto_assign_windows: true,
            preserve_focus_per_workspace: true,
            workspace_auto_back_and_forth: false,
            prevent_wrapping: false,
            workspace_names: default_workspace_names(),
            default_workspace: 0,
            reapply_app_rules_on_title_change: false,
            app_rules: Vec::new(),
            workspace_rules: Vec::new(),
        }
    }
}

impl VirtualWorkspaceSettings {
    pub fn workspace_name(&self, index: usize) -> String {
        self.workspace_names
            .get(index)
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Workspace {}", index + 1))
    }

    pub fn resize(&mut self, count: usize) {
        self.default_workspace_count = count;
        // Invalid counts are left for the shared validator, without large allocations.
        if !(1..=MAX_WORKSPACES).contains(&count) {
            return;
        }
        let removed = self.workspace_names.iter().skip(count).cloned().collect::<Vec<_>>();
        self.workspace_names.truncate(count);
        self.default_workspace = self.default_workspace.min(count - 1);
        let removed_selector = |v: &WorkspaceSelector| match v {
            WorkspaceSelector::Index(i) => *i >= count,
            WorkspaceSelector::Name(n) => removed.contains(n),
        };
        self.workspace_rules.retain(|r| !removed_selector(&r.workspace));
        for r in &mut self.app_rules {
            if r.workspace.as_ref().is_some_and(removed_selector) {
                r.workspace = None;
            }
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.default_workspace_count == 0 {
            issues.push("default_workspace_count must be at least 1".to_string());
        }
        if self.default_workspace_count > MAX_WORKSPACES {
            issues.push(format!(
                "default_workspace_count should not exceed {} for performance reasons",
                MAX_WORKSPACES
            ));
        }

        if self.workspace_names.len() > self.default_workspace_count {
            issues.push("More workspace names provided than default_workspace_count".to_string());
        }

        if self.default_workspace >= self.default_workspace_count {
            issues.push(format!(
                "default_workspace ({}) must be less than default_workspace_count ({})",
                self.default_workspace, self.default_workspace_count
            ));
        }

        let mut names = crate::common::collections::HashMap::default();
        for index in 0..self.default_workspace_count.min(MAX_WORKSPACES) {
            let name = self.workspace_name(index);
            if let Some(first) = names.get(&name) {
                issues.push(format!(
                    "Duplicate workspace name {name:?} (workspaces {} and {}). Workspace names must be unique.",
                    first + 1, index + 1
                ));
            } else {
                names.insert(name, index);
            }
        }

        for (index, rule) in self.workspace_rules.iter().enumerate() {
            if rule.layout.is_none() && rule.display.is_none() {
                issues.push(format!("Workspace rule {index} has no layout or display"));
            }
            if let Some(preferences) = &rule.display
                && (preferences.is_empty()
                    || preferences.iter().any(|value| value.trim().is_empty()))
            {
                issues.push(format!("Workspace rule {index} display preferences must contain nonempty UUIDs, names, or keywords"));
            }
        }

        // Validate rules and check duplicates in a single pass
        let mut seen_app_ids = crate::common::collections::HashSet::default();
        let mut seen_app_names = crate::common::collections::HashSet::default();
        let mut seen_title_regexes = crate::common::collections::HashSet::default();
        let mut seen_title_substrings = crate::common::collections::HashSet::default();
        let mut seen_ax_roles = crate::common::collections::HashSet::default();
        let mut seen_ax_subroles = crate::common::collections::HashSet::default();

        for (index, rule) in self.app_rules.iter().enumerate() {
            let app_id_empty = rule.app_id.as_ref().map_or(true, |id| id.is_empty());
            if app_id_empty
                && rule.app_name.is_none()
                && rule.title_regex.is_none()
                && rule.title_substring.is_none()
                && rule.ax_role.is_none()
                && rule.ax_subrole.is_none()
            {
                issues.push(format!(
                    "App rule {} has no app_id, app_name, title_regex, or title_substring specified",
                    index
                ));
            }

            if let Some(ref workspace) = rule.workspace {
                if let WorkspaceSelector::Index(idx) = workspace {
                    if *idx >= self.default_workspace_count {
                        issues.push(format!(
                            "App rule {} references workspace {} but only {} workspaces will be created",
                            index, idx, self.default_workspace_count
                        ));
                    }
                }
            }

            if let Some(position) = rule.position {
                if !position.x.is_finite()
                    || !position.y.is_finite()
                    || !(0.0..=1.0).contains(&position.x)
                    || !(0.0..=1.0).contains(&position.y)
                {
                    issues.push(format!(
                        "App rule {} position x and y must be finite values between 0 and 1",
                        index
                    ));
                }
                if !rule.floating {
                    issues.push(format!(
                        "App rule {} specifies position, but position only applies when floating = true",
                        index
                    ));
                }
            }

            if let Some(size) = rule.size {
                if size.w.is_none() && size.h.is_none() {
                    issues.push(format!(
                        "App rule {} size must specify at least one of w or h",
                        index
                    ));
                }
                if size.w.is_some_and(|value| !value.is_finite() || value <= 0.0)
                    || size.h.is_some_and(|value| !value.is_finite() || value <= 0.0)
                {
                    issues.push(format!(
                        "App rule {} size dimensions must be finite positive values",
                        index
                    ));
                }
            }

            if let Some(ref app_id) = rule.app_id {
                if !app_id.is_empty() && !app_id.contains('.') {
                    issues.push(format!(
                        "App rule {} has suspicious app_id '{}' (should be bundle identifier like 'com.example.app')",
                        index, app_id
                    ));
                }

                let has_specific_match = rule.app_name.is_some()
                    || rule.title_regex.is_some()
                    || rule.title_substring.is_some()
                    || rule.ax_role.is_some()
                    || rule.ax_subrole.is_some();
                if !app_id.is_empty() && !has_specific_match && !seen_app_ids.insert(app_id) {
                    issues.push(format!("Duplicate app_id '{}' in rule {}", app_id, index));
                }
            }

            if let Some(ref app_name) = rule.app_name {
                let has_specific_match = rule.app_id.is_some()
                    || rule.title_regex.is_some()
                    || rule.title_substring.is_some()
                    || rule.ax_role.is_some()
                    || rule.ax_subrole.is_some();
                if !has_specific_match && !seen_app_names.insert(app_name) {
                    issues.push(format!("Duplicate app_name '{}' in rule {}", app_name, index));
                }
            }

            if let Some(ref title_re) = rule.title_regex {
                if title_re.is_empty() {
                    issues.push(format!("App rule {} has empty title_regex", index));
                } else if let Err(error) =
                    RegexBuilder::new(title_re).case_insensitive(true).build()
                {
                    issues.push(format!(
                        "App rule {} has invalid title_regex '{}': {}",
                        index, title_re, error
                    ));
                } else if !seen_title_regexes.insert(title_re) {
                    issues.push(format!("Duplicate title_regex '{}' in rule {}", title_re, index));
                }
            }

            if rule.manage == Some(false)
                && (rule.workspace.is_some()
                    || rule.floating
                    || rule.position.is_some()
                    || rule.size.is_some()
                    || rule.focus)
            {
                issues.push(format!(
                    "App rule {} sets manage = false, so its workspace, floating, position, size, and focus effects are ignored",
                    index
                ));
            }

            if let Some(ref title_sub) = rule.title_substring {
                if title_sub.is_empty() {
                    issues.push(format!("App rule {} has empty title_substring", index));
                } else if !seen_title_substrings.insert(title_sub) {
                    issues.push(format!(
                        "Duplicate title_substring '{}' in rule {}",
                        title_sub, index
                    ));
                }
            }

            if let Some(ref ax_role) = rule.ax_role {
                if ax_role.is_empty() {
                    issues.push(format!("App rule {} has empty ax_role", index));
                } else if !seen_ax_roles.insert(ax_role) {
                    issues.push(format!("Duplicate ax_role '{}' in rule {}", ax_role, index));
                }
            }

            if let Some(ref ax_sub) = rule.ax_subrole {
                if ax_sub.is_empty() {
                    issues.push(format!("App rule {} has empty ax_subrole", index));
                } else if !seen_ax_subroles.insert(ax_sub) {
                    issues.push(format!("Duplicate ax_subrole '{}' in rule {}", ax_sub, index));
                }
            }
        }

        issues
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConfigSource {
    pub settings: Settings,
    pub keys: std::collections::BTreeMap<String, WmCommand>,
    #[serde(default)]
    pub binding_modes:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, WmCommand>>,
    #[serde(default)]
    pub virtual_workspaces: VirtualWorkspaceSettings,
    /// 可在按键绑定中复用的修饰键组合。
    /// 例如 "comb1" = "Alt + Shift" 之后，可在按键中使用 "comb1 + C"。
    #[serde(default)]
    pub modifier_combinations: std::collections::BTreeMap<String, String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Config {
    pub settings: Settings,
    pub keys: Vec<(Hotkey, WmCommand)>,
    #[serde(default)]
    pub binding_mode_specs: BindingModeSpecs,
    pub virtual_workspaces: VirtualWorkspaceSettings,
}

pub type BindingModeSpecs = Vec<(String, Vec<(String, WmCommand)>)>;

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
#[setting(group = "general")]
pub struct Settings {
    #[serde(default)]
    /// 窗口布局变化时播放动画。
    #[setting(
        label = "窗口变化动画",
        group = "general",
        aliases = "animation transitions motion"
    )]
    pub animate: bool,
    #[serde(default = "default_animation_duration")]
    /// 窗口动画时长，以秒为单位。
    // #[setting(
    //     label = "Duration (seconds)",
    //     group = "general",
    //     enabled_by = "animate",
    //     aliases = "animation speed seconds"
    // )]
    #[setting(ignore)]
    pub animation_duration: f64,
    #[serde(default = "default_animation_fps")]
    /// 窗口动画的最大每秒帧数。
    // #[setting(
    //     label = "Frame rate",
    //     group = "general",
    //     enabled_by = "animate",
    //     aliases = "animation fps performance"
    // )]
    #[setting(ignore)]
    pub animation_fps: f64,
    #[serde(default)]
    /// 窗口动画使用的时序曲线。
    // #[setting(
    //     label = "Animation easing",
    //     group = "general",
    //     choices,
    //     enabled_by = "animate",
    //     aliases = "transition curve"
    // )]
    #[setting(ignore)]
    pub animation_easing: AnimationEasing,
    #[serde(default = "yes")]
    /// 启动 Rift 时关闭自动平铺。
    #[setting(
        label = "启动时关闭平铺",
        group = "general",
        aliases = "floating startup automatic tiling"
    )]
    pub default_disable: bool,
    #[serde(default = "yes")]
    /// 焦点变化时将指针移入获得焦点的窗口。
    #[setting(
        label = "将指针移到焦点窗口",
        group = "pointer",
        aliases = "mouse follows focus cursor warp"
    )]
    pub mouse_follows_focus: bool,
    #[serde(default = "yes")]
    /// 将指针移入焦点窗口后隐藏指针。
    #[setting(
        label = "聚焦后隐藏指针",
        group = "pointer",
        enabled_by = "mouse_follows_focus",
        aliases = "hide mouse cursor"
    )]
    pub mouse_hides_on_focus: bool,
    #[serde(default = "yes")]
    /// 指针移到窗口上方时聚焦该窗口。
    #[setting(
        label = "焦点跟随指针",
        group = "pointer",
        order = 0,
        aliases = "ffm focus follows mouse hover autofocus"
    )]
    pub focus_follows_mouse: bool,
    /// 将上下排列的显示器视为水平指针链。
    /// `top-to-bottom` 把较高的显示器映射到左侧，`bottom-to-top` 反向映射。
    #[serde(default)]
    #[setting(ignore)]
    pub horizontal_mouse_warp: Option<HorizontalMouseWarp>,
    /// 按住时临时禁用焦点跟随指针的快捷键。
    /// 接受完整快捷键（如 "Ctrl + A"）或仅修饰键（如 "Ctrl"）。
    #[serde(default)]
    #[setting(ignore)]
    pub focus_follows_mouse_disable_hotkey: Option<HotkeySpec>,
    /// 激活时不应触发自动工作区切换的应用。
    /// 填写常会不当抢占焦点的 Bundle 标识符（如 "com.apple.Spotlight"），
    /// 避免其引发工作区切换。
    #[serde(default)]
    #[setting(
        label = "自动聚焦排除列表",
        group = "advanced",
        custom,
        aliases = "focus exclude ignore app"
    )]
    pub auto_focus_blacklist: Vec<String>,
    #[serde(default)]
    #[setting(ignore)]
    pub layout: LayoutSettings,
    #[serde(default)]
    #[setting(ignore)]
    pub ui: UiSettings,
    /// 触控板手势设置。
    #[serde(default)]
    #[setting(ignore)]
    pub gestures: GestureSettings,
    /// 修饰键拖动与平铺窗口放置设置。
    #[serde(default)]
    #[setting(ignore)]
    pub drag_drop: DragDropSettings,

    /// 启动时执行的命令，例如用于订阅事件的命令。
    #[serde(default)]
    #[setting(
        label = "启动命令",
        group = "advanced",
        custom,
        aliases = "launch run shell exec"
    )]
    pub run_on_start: Vec<String>,

    /// 配置文件变化时启用热重载。
    #[serde(default = "yes")]
    /// 配置文件变化时自动重新载入。
    #[setting(
        label = "外部编辑后重新载入配置",
        group = "advanced",
        aliases = "configuration file automatic reload toml"
    )]
    pub hot_reload: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum HorizontalMouseWarp {
    TopToBottom,
    BottomToTop,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UiSettings {
    #[serde(default)]
    pub menu_bar: MenuBarSettings,
    #[serde(default)]
    pub stack_line: StackLineSettings,
    #[serde(default)]
    pub mission_control: MissionControlSettings,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
#[setting(group = "main")]
pub struct GestureSettings {
    /// 启用水平轻扫以切换虚拟工作区。
    #[serde(default)]
    #[setting(label = "启用", group = "main")]
    pub enabled: bool,
    /// 若启用，Rift 会拦截并处理水平轻扫事件，macOS 与前台应用不再同时处理。
    #[serde(default = "yes")]
    #[setting(label = "拦截 macOS 工作区轻扫", group = "main")]
    pub consume_dock_swipe: bool,
    /// 反转水平方向，交换下一个与上一个工作区。
    #[serde(default)]
    #[setting(label = "反转方向", group = "main")]
    pub invert_horizontal_swipe: bool,
    /// 将手势识别为水平轻扫时，允许的最大垂直偏移绝对值。
    #[serde(default = "default_swipe_vertical_tolerance")]
    #[setting(label = "垂直容差", group = "advanced")]
    pub swipe_vertical_tolerance: f64,
    /// 轻扫时尝试跳过空工作区（需要相关支持）。
    #[serde(default)]
    #[setting(label = "跳过空工作区", group = "main")]
    pub skip_empty: bool,
    /// 轻扫所需的手指数，默认为 3。
    #[serde(default = "default_swipe_fingers")]
    #[setting(label = "手指数", group = "main")]
    pub fingers: usize,
    /// 触发轻扫所需的归一化水平距离（0 到 1）。
    #[serde(default = "default_distance_pct")]
    #[setting(label = "触发距离 (%)", group = "advanced", scale = 100.0)]
    pub distance_pct: f64,
    /// 轻扫生效时提供触觉反馈。
    #[serde(default = "yes")]
    #[setting(label = "触觉反馈", group = "main")]
    pub haptics_enabled: bool,
    /// 触觉反馈模式：通用、对齐或级别变化。
    #[serde(default)]
    #[setting(label = "触觉反馈模式", group = "advanced", choices)]
    pub haptic_pattern: HapticPattern,
}

impl Default for GestureSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            consume_dock_swipe: true,
            invert_horizontal_swipe: false,
            swipe_vertical_tolerance: default_swipe_vertical_tolerance(),
            skip_empty: true,
            fingers: default_swipe_fingers(),
            distance_pct: default_distance_pct(),
            haptics_enabled: true,
            haptic_pattern: HapticPattern::LevelChange,
        }
    }
}

///
/// Serialized values are `cmd`, `alt`, `shift`, `ctrl`, `fn`, or a combination in
/// key-binding syntax such as `Ctrl + Alt + Shift + Meta`.
#[derive(Debug, PartialEq, Clone, Copy, Default, Eq, SerializeDisplay, DeserializeFromStr)]
pub enum MouseModifier {
    /// 地球仪/Fn 功能键。通常与应用的快捷键冲突较少，因此作为默认值。
    #[default]
    Fn,
    /// 一个或多个修饰键族。单个族按住即匹配；两个及以上需完全匹配。
    Keys(Modifiers),
}

impl std::str::FromStr for MouseModifier {
    type Err = anyhow::Error;

    /// Parses a key binding's modifiers, such as `command` or `Ctrl + Alt`. Left/right
    /// spellings stand for either side.
    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        if spec.trim().eq_ignore_ascii_case("fn") {
            return Ok(Self::Fn);
        }

        Ok(Self::Keys(modifiers_from_str(spec)?.families()))
    }
}

impl std::fmt::Display for MouseModifier {
    /// Use the legacy canonical names when serializing single modifiers.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fn => f.write_str("fn"),
            Self::Keys(Modifiers::META) => f.write_str("cmd"),
            Self::Keys(Modifiers::ALT) => f.write_str("alt"),
            Self::Keys(Modifiers::SHIFT) => f.write_str("shift"),
            Self::Keys(Modifiers::CONTROL) => f.write_str("ctrl"),
            Self::Keys(modifiers) => write!(f, "{modifiers}"),
        }
    }
}

/// 修饰键加鼠标按键手势使用的操作。
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Eq, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum MouseAction {
    /// 不捕获此按键，点击按正常方式传递。
    #[setting(label = "直接传递")]
    None,
    /// 在窗口内部任意位置拖动窗口。
    #[default]
    #[setting(label = "移动窗口")]
    Move,
}

/// 平铺窗口在另一窗口中心区域释放时执行的操作。
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Eq, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum MouseDropAction {
    /// 交换源窗口与目标窗口在布局中的位置。
    #[default]
    #[setting(label = "交换位置")]
    Swap,
    /// 将源窗口并入目标的堆叠组，并选中源窗口。
    #[setting(label = "堆叠")]
    Stack,
}

/// 修饰键鼠标操作与原生平铺窗口放置设置。
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct DragDropSettings {
    /// 成功放置窗口后提供触控板触觉反馈。
    #[serde(default = "yes")]
    #[setting(label = "触觉反馈", order = 7)]
    pub haptics_enabled: bool,
    /// 启用原生拖放目标选择和修饰键鼠标操作。
    #[serde(default = "yes")]
    #[setting(label = "启用", order = 0)]
    pub enabled: bool,
    /// 与 `action1` 或 `action2` 配合按住的修饰键：`cmd`、`alt`、`shift`、`ctrl`、`fn`，
    /// 或 `Ctrl + Alt + Shift + Meta` 之类的组合（`fn` 不能参与组合）。
    /// `AltRight` 等左右写法匹配任意一侧；单修饰键按住即生效，组合需完全匹配。
    #[serde(default)]
    #[setting(custom, order = 1)]
    pub modifier: MouseModifier,
    /// 按住指定修饰键时，鼠标左键执行的操作。
    #[serde(default)]
    #[setting(label = "主按钮", choices, order = 2)]
    pub action1: MouseAction,
    /// 按住指定修饰键时，鼠标右键执行的操作。
    #[serde(default = "default_mouse_action_none")]
    #[setting(label = "辅助按钮", choices, order = 3)]
    pub action2: MouseAction,
    /// 将平铺窗口拖到目标窗口中心区域时执行的操作。
    #[serde(default)]
    #[setting(label = "放到中心时", choices, order = 4)]
    pub drop_action: MouseDropAction,
    /// 各边缘区域占目标窗口尺寸的比例。有效范围 `0.10..=0.45`，默认 `0.25`。
    #[serde(default = "default_drop_zone_fraction")]
    #[setting(label = "中心放置区域 (%)", scale = 100.0, order = 5)]
    pub drop_zone_fraction: f64,
    /// 为待放置操作显示半透明圆角 WindowServer 覆盖层。
    /// 仅当目标平铺区域或放置区域变化时更新。
    #[serde(default = "yes")]
    #[setting(label = "显示放置预览", order = 6)]
    pub preview: bool,
}

impl Default for DragDropSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            haptics_enabled: true,
            modifier: MouseModifier::Fn,
            action1: MouseAction::Move,
            action2: MouseAction::None,
            drop_action: MouseDropAction::Swap,
            drop_zone_fraction: default_drop_zone_fraction(),
            preview: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum MenuBarDisplayMode {
    #[default]
    #[setting(label = "所有工作区")]
    All,
    #[setting(label = "当前工作区")]
    Active,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum ActiveWorkspaceLabel {
    #[default]
    #[setting(label = "序号")]
    Index,
    #[setting(label = "名称")]
    Name,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceDisplayStyle {
    #[default]
    #[setting(label = "布局")]
    Layout,
    #[setting(label = "标签")]
    Label,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct MenuBarSettings {
    #[serde(default = "yes")]
    /// 在菜单栏显示 Rift 与工作区信息。
    #[setting(label = "显示菜单栏指示器")]
    pub enabled: bool,
    #[serde(default)]
    /// 在菜单栏中包含没有窗口的工作区。
    #[setting(label = "显示空工作区", enabled_by = "enabled")]
    pub show_empty: bool,
    #[serde(default = "yes")]
    /// 在工作区旁显示当前快捷键模式（默认模式除外）。
    #[setting(label = "显示快捷键模式", enabled_by = "enabled")]
    pub show_binding_mode: bool,
    #[serde(default)]
    /// 仅显示当前菜单栏所在显示器的工作区。
    #[setting(label = "仅显示当前显示器", enabled_by = "enabled")]
    pub current_display_only: bool,
    #[serde(default)]
    /// 选择显示全部工作区或仅显示当前工作区。
    #[setting(label = "显示的工作区", choices, enabled_by = "enabled")]
    pub mode: MenuBarDisplayMode,
    #[serde(default)]
    /// 使用标签样式时，选择显示工作区编号或名称。
    #[setting(label = "工作区标签", choices, enabled_by = "enabled")]
    pub active_label: ActiveWorkspaceLabel,
    #[serde(default)]
    /// 显示工作区标签或布局指示图标。
    #[setting(label = "显示样式", choices, enabled_by = "enabled")]
    pub display_style: WorkspaceDisplayStyle,
    #[serde(default)]
    /// 窗口全屏时用方括号包裹工作区标签或布局指示。
    #[setting(label = "显示全屏标记", enabled_by = "enabled")]
    pub show_fullscreen_indicator: bool,
    #[serde(default = "default_layout_folder")]
    /// 存放自定义布局指示图片的文件夹。
    #[setting(label = "布局文件夹", custom, enabled_by = "enabled")]
    pub layout_folder: PathBuf,
}

impl MenuBarSettings {
    pub fn resolved_layout_folder(&self) -> PathBuf {
        let Ok(relative) = self.layout_folder.strip_prefix("~") else {
            return self.layout_folder.clone();
        };
        dirs::home_dir()
            .map(|home| home.join(relative))
            .unwrap_or_else(|| self.layout_folder.clone())
    }
}

impl Default for MenuBarSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            show_empty: false,
            show_binding_mode: true,
            current_display_only: false,
            mode: MenuBarDisplayMode::default(),
            active_label: ActiveWorkspaceLabel::default(),
            display_style: WorkspaceDisplayStyle::default(),
            show_fullscreen_indicator: false,
            layout_folder: default_layout_folder(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct StackLineSettings {
    /// 在堆叠窗口旁显示原生 Liquid Glass 标签。
    #[serde(default = "yes")]
    #[setting(label = "启用")]
    pub enabled: bool,
    #[serde(default)]
    /// 悬停标签时的行为：仅高亮，或像点击一样选中。
    #[setting(label = "交互方式", choices, enabled_by = "enabled")]
    pub hover: StackLineHoverMode,
    #[serde(default)]
    /// 水平堆叠时标签的位置。
    #[setting(label = "水平堆叠时的位置", choices, enabled_by = "enabled")]
    pub horiz_placement: HorizontalPlacement,
    #[serde(default)]
    /// 垂直堆叠时标签的位置；标题沿标签栏排列。
    #[setting(label = "垂直堆叠时的位置", choices, enabled_by = "enabled")]
    pub vert_placement: VerticalPlacement,
    #[serde(default = "default_stack_line_thickness")]
    /// 标签栏厚度，单位为点（至少 20）。
    #[setting(label = "厚度 (点)", enabled_by = "enabled")]
    pub thickness: f64,
    #[serde(default)]
    /// 标签栏与窗口之间的间距，单位为点。
    #[setting(label = "间距 (点)", enabled_by = "enabled")]
    pub spacing: f64,
}

impl Default for StackLineSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            hover: StackLineHoverMode::default(),
            horiz_placement: HorizontalPlacement::default(),
            vert_placement: VerticalPlacement::default(),
            thickness: default_stack_line_thickness(),
            spacing: 0.0,
        }
    }
}

fn default_stack_line_thickness() -> f64 { cgs::tabs::TabBar::HEIGHT }

impl StackLineSettings {
    /// 原生标签栏低于此厚度时不可用。
    pub const MIN_THICKNESS: f64 = 20.0;

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !self.thickness.is_finite() || self.thickness < Self::MIN_THICKNESS {
            issues.push(format!(
                "ui.stack_line.thickness must be at least {}, got {}",
                Self::MIN_THICKNESS,
                self.thickness
            ));
        }
        if !self.spacing.is_finite() || self.spacing < 0.0 {
            issues.push(format!(
                "ui.stack_line.spacing must be non-negative, got {}",
                self.spacing
            ));
        }
        issues
    }
}

/// 保留原生标签栏之前的取值含义，使现有配置行为不变。
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum StackLineHoverMode {
    /// 悬停标签即选中，效果与点击相同。
    #[setting(label = "悬停即选中")]
    Click,
    /// 悬停时仅高亮标签，点击后选中。
    #[default]
    #[setting(label = "点击选择")]
    Hover,
}

impl StackLineHoverMode {
    pub fn selects_on_hover(self) -> bool { self == Self::Click }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalPlacement {
    #[default]
    #[setting(label = "上方")]
    Top,
    #[setting(label = "下方")]
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum VerticalPlacement {
    #[default]
    #[setting(label = "左侧")]
    Left,
    #[setting(label = "右侧")]
    Right,
}

/// Space a stack gives up to its tab bar, and on which edge.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct StackLineReserve {
    pub thickness: f64,
    pub spacing: f64,
    pub horiz: HorizontalPlacement,
    pub vert: VerticalPlacement,
}

impl StackLineReserve {
    /// Thickness plus spacing; nothing when the bar is absent.
    pub fn total(&self) -> f64 {
        if self.thickness > 0.0 {
            self.thickness + self.spacing
        } else {
            0.0
        }
    }

    /// The reservation along one axis of a node of `kind`. Horizontal stacks give up
    /// height for a bar above or below; vertical stacks give up width for one beside.
    pub fn on_axis(&self, kind: crate::layout_engine::LayoutKind, horizontal: bool) -> f64 {
        use crate::layout_engine::LayoutKind::*;
        match (kind, horizontal) {
            (HorizontalStack, false) | (VerticalStack, true) => self.total(),
            _ => 0.0,
        }
    }

    /// 收缩堆叠区域，为标签栏留出空间。
    pub fn inset(
        &self,
        mut rect: objc2_core_foundation::CGRect,
        horizontal_stack: bool,
    ) -> objc2_core_foundation::CGRect {
        let reserve = self.total();
        if horizontal_stack {
            let reserve = reserve.min(rect.size.height);
            if self.horiz == HorizontalPlacement::Top {
                rect.origin.y += reserve;
            }
            rect.size.height -= reserve;
        } else {
            let reserve = reserve.min(rect.size.width);
            if self.vert == VerticalPlacement::Left {
                rect.origin.x += reserve;
            }
            rect.size.width -= reserve;
        }
        rect
    }

    /// 标签栏沿堆叠预留边缘的自身区域，不会越界。
    pub fn bar(
        &self,
        rect: objc2_core_foundation::CGRect,
        horizontal_stack: bool,
    ) -> objc2_core_foundation::CGRect {
        use objc2_core_foundation::{CGPoint, CGRect, CGSize};
        let (o, s) = (rect.origin, rect.size);
        let t = self.thickness.min(if horizontal_stack { s.height } else { s.width }).max(0.0);
        match (horizontal_stack, self.horiz, self.vert) {
            (true, HorizontalPlacement::Top, _) => CGRect::new(o, CGSize::new(s.width, t)),
            (true, HorizontalPlacement::Bottom, _) => {
                CGRect::new(CGPoint::new(o.x, o.y + s.height - t), CGSize::new(s.width, t))
            }
            (false, _, VerticalPlacement::Left) => CGRect::new(o, CGSize::new(t, s.height)),
            (false, _, VerticalPlacement::Right) => {
                CGRect::new(CGPoint::new(o.x + s.width - t, o.y), CGSize::new(t, s.height))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct MissionControlSettings {
    /// 在总览中显示没有窗口的工作区。
    #[serde(default = "yes")]
    #[setting(label = "显示空工作区", enabled_by = "enabled")]
    pub show_empty_workspaces: bool,
    #[serde(default = "yes")]
    /// 在总览中显示每个窗口的预览。需要屏幕录制权限。
    #[setting(label = "窗口预览", enabled_by = "enabled")]
    pub window_previews: bool,
    #[serde(default)]
    /// 启用工作区总览。需要重启 Rift。
    #[setting(label = "启用", order = 0)]
    pub enabled: bool,
    #[serde(default)]
    /// 总览标签与装饰使用渐隐渐显过渡。
    #[setting(label = "淡入淡出", enabled_by = "enabled")]
    pub fade_enabled: bool,
    #[serde(default = "default_mission_control_fade_duration_ms")]
    /// 总览进入与镜头导航时长，单位为毫秒。
    #[setting(label = "淡入淡出时长", unit = "ms", enabled_by = "enabled")]
    pub fade_duration_ms: f64,
}

fn default_mission_control_fade_duration_ms() -> f64 { 180.0 }

fn default_drop_zone_fraction() -> f64 { 0.25 }

fn default_mouse_action_none() -> MouseAction { MouseAction::None }

fn default_master_stack_ratio() -> f64 { 0.6 }

fn default_master_stack_count() -> usize { 1 }

fn default_scrolling_column_width_ratio() -> f64 { 0.7 }
fn default_scrolling_preset_column_widths() -> Vec<f64> { vec![1.0 / 3.0, 0.5, 2.0 / 3.0] }

fn default_scrolling_min_column_width_ratio() -> f64 { 0.3 }

fn default_scrolling_max_column_width_ratio() -> f64 { 0.9 }

impl StackLineSettings {
    pub fn reserve(&self) -> StackLineReserve {
        StackLineReserve {
            thickness: if self.enabled {
                self.thickness.max(0.0)
            } else {
                0.0
            },
            spacing: self.spacing,
            horiz: self.horiz_placement,
            vert: self.vert_placement,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowInsertionPoint {
    /// 在当前选中项之后插入新窗口。
    #[default]
    NextToSelection,
    /// 将新窗口追加到布局树末尾。
    EndOfTree,
}

/// 所有布局系统共用的选项。
///
/// 这些字段会同时展开到 `[settings.layout]` 与各布局表中；
/// 单个布局表的值优先。
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct BaseLayoutSettings {
    /// 新纳入管理的窗口在布局中的插入位置。
    #[serde(default)]
    #[setting(label = "新窗口位置", custom)]
    pub window_insertion_point: Option<WindowInsertionPoint>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct TraditionalLayoutSettings {
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 插入节点时使用 Sway 风格的兄弟节点归一化。新节点获得兄弟节点的平均权重，
    /// 而不是拆分选中节点的份额。
    #[serde(default = "yes")]
    #[setting(label = "保持窗口等大")]
    pub equalize_nodes: bool,
}

impl Default for TraditionalLayoutSettings {
    fn default() -> Self {
        Self {
            base: BaseLayoutSettings::default(),
            equalize_nodes: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct BspLayoutSettings {
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 单个窗口按此宽高比居中。
    #[setting(label = "单窗口宽高比", custom)]
    pub single_window_aspect_ratio: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct LayoutSettings {
    /// 所有布局类型继承的设置，布局表中的值可覆盖。
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 未单独指定布局的工作区使用的布局。
    #[serde(default)]
    #[setting(label = "默认布局", choices)]
    pub mode: LayoutMode,
    /// 传统布局配置。
    #[serde(default)]
    #[setting(ignore)]
    pub traditional: TraditionalLayoutSettings,
    /// 二分布局配置。
    #[serde(default)]
    #[setting(ignore)]
    pub bsp: BspLayoutSettings,
    /// 堆叠系统配置。
    #[serde(default)]
    #[setting(ignore)]
    pub stack: StackSettings,
    /// 主从布局配置。
    #[serde(default)]
    #[setting(ignore)]
    pub master_stack: MasterStackSettings,
    #[serde(default)]
    #[setting(ignore)]
    pub gaps: GapSettings,
    /// 滚动布局配置（niri 风格列）。
    #[serde(default)]
    #[setting(ignore)]
    pub scrolling: ScrollingLayoutSettings,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct ScrollingLayoutSettings {
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 是否对滚动布局中的窗口移动播放动画。
    /// 强烈建议保持启用。
    #[serde(default = "default_scrolling_animate")]
    #[setting(
        label = "导航动画",
        custom,
        aliases = "animation transitions motion"
    )]
    pub animate: Option<bool>,
    /// 当前列的默认宽度，占屏幕宽度的比例。
    #[serde(default = "default_scrolling_column_width_ratio")]
    #[setting(
        label = "默认列宽",
        custom,
        aliases = "column width sizing percentages tiling"
    )]
    pub column_width_ratio: f64,
    /// 按配置顺序循环使用的列宽比例。
    #[serde(default = "default_scrolling_preset_column_widths")]
    #[setting(
        label = "预设宽度",
        custom,
        aliases = "column width presets cycle sizes"
    )]
    pub preset_column_widths: Vec<f64>,
    /// 窗口进入滚动布局时保留其现有列宽。
    #[serde(default = "yes")]
    #[setting(label = "保留窗口尺寸")]
    pub preserve_window_sizes: bool,
    /// 工作区只有一个滚动列时填满可用宽度。
    /// 添加另一列后恢复该列存储的宽度。
    #[serde(default)]
    #[setting(label = "扩展单列", custom)]
    pub expand_single_column: bool,
    /// 调整尺寸命令允许的最小列宽比例。
    #[serde(default = "default_scrolling_min_column_width_ratio")]
    #[setting(
        label = "最小宽度",
        scale = 100.0,
        unit = "%",
        aliases = "minimum column width sizing percentages tiling"
    )]
    pub min_column_width_ratio: f64,
    /// 调整尺寸命令允许的最大列宽比例。
    #[serde(default = "default_scrolling_max_column_width_ratio")]
    #[setting(
        label = "最大宽度",
        scale = 100.0,
        unit = "%",
        aliases = "maximum column width sizing percentages tiling"
    )]
    pub max_column_width_ratio: f64,
    /// 按显示器 UUID 单独指定列宽，未指定项使用通用设置。
    #[serde(default)]
    #[setting(label = "显示器设置", custom)]
    pub per_display: HashMap<String, ScrollingWidthOverride>,
    /// 焦点列的对齐方式：左侧、居中或右侧。
    #[serde(default)]
    #[setting(label = "对齐方式", choices)]
    pub alignment: ScrollingAlignment,
    /// 水平焦点导航行为：
    /// - niri：仅在需要时按导航方向显示。
    /// - anchored：始终将焦点列对齐到 `alignment`。
    /// niri 模式下手势释放可自由平移；anchored 模式会吸附到对齐位置。
    #[serde(default)]
    #[setting(label = "焦点导航", choices)]
    pub focus_navigation_style: ScrollingFocusNavigationStyle,
    /// 滚动布局的触控板手势。
    #[serde(default)]
    #[setting(ignore)]
    pub gestures: ScrollingGestureSettings,
}

fn default_scrolling_animate() -> Option<bool> { Some(true) }

impl Default for ScrollingLayoutSettings {
    fn default() -> Self {
        Self {
            base: BaseLayoutSettings::default(),
            animate: Some(true),
            column_width_ratio: default_scrolling_column_width_ratio(),
            preset_column_widths: default_scrolling_preset_column_widths(),
            preserve_window_sizes: true,
            expand_single_column: false,
            min_column_width_ratio: default_scrolling_min_column_width_ratio(),
            max_column_width_ratio: default_scrolling_max_column_width_ratio(),
            per_display: HashMap::default(),
            alignment: ScrollingAlignment::default(),
            focus_navigation_style: ScrollingFocusNavigationStyle::default(),
            gestures: ScrollingGestureSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScrollingWidthOverride {
    pub column_width_ratio: Option<f64>,
    pub min_column_width_ratio: Option<f64>,
    pub max_column_width_ratio: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum MasterStackSide {
    #[default]
    #[setting(label = "左侧")]
    Left,
    #[setting(label = "右侧")]
    Right,
    #[setting(label = "上方")]
    Top,
    #[setting(label = "下方")]
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum ScrollingAlignment {
    #[setting(label = "左侧")]
    Left,
    #[default]
    #[setting(label = "居中")]
    Center,
    #[setting(label = "右侧")]
    Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum ScrollingFocusNavigationStyle {
    #[default]
    #[setting(label = "按需显示")]
    Niri,
    #[setting(label = "固定对齐")]
    Anchored,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct MasterStackSettings {
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 为主区域保留的空间比例，范围为 0.05 到 0.95。
    #[serde(default = "default_master_stack_ratio")]
    #[setting(label = "主区域宽度", custom)]
    pub master_ratio: f64,
    /// 主区域保留的窗口数，至少为 1。
    #[serde(default = "default_master_stack_count")]
    #[setting(label = "主区域窗口数")]
    pub master_count: usize,
    #[serde(default)]
    #[setting(label = "主区域位置", choices)]
    pub master_side: MasterStackSide,
    /// 主区域已满时，新窗口的插入位置。
    #[serde(default = "default_master_stack_new_window_placement")]
    #[setting(label = "新窗口归属", choices)]
    pub new_window_placement: MasterStackNewWindowPlacement,
    /// 主区域的排列方向，覆盖根据主区域位置推导的默认方向。
    #[serde(default)]
    #[setting(label = "主区域排列", custom)]
    pub master_arrangement: Option<crate::layout_engine::Orientation>,
    /// 从区域的排列方向，覆盖根据主区域位置推导的默认方向。
    #[serde(default)]
    #[setting(label = "从区域排列", custom)]
    pub stack_arrangement: Option<crate::layout_engine::Orientation>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum MasterStackNewWindowPlacement {
    #[setting(label = "主区域")]
    Master,
    #[setting(label = "从区域")]
    Stack,
    #[setting(label = "当前焦点区域")]
    Focused,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, ConfigSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScrollingGestureSettings {
    /// 启用连续水平视口手势。
    #[serde(default)]
    #[setting(label = "启用", order = 0)]
    pub enabled: bool,
    /// 兼容旧配置保留；手势释放动画由滚动动画控制。
    #[serde(default, skip_serializing)]
    #[setting(ignore)]
    pub animate: Option<bool>,
    /// 反转水平方向，交换左侧与右侧。
    #[serde(default)]
    #[setting(label = "反转水平方向", order = 1)]
    pub invert_horizontal: bool,
    /// 将手势识别为水平轻扫时，允许的最大垂直偏移绝对值。
    #[serde(default = "default_swipe_vertical_tolerance")]
    #[setting(label = "垂直容差", order = 5)]
    pub vertical_tolerance: f64,
    /// 滚动手势所需的手指数。
    #[serde(default = "default_swipe_fingers")]
    #[setting(label = "手指数", order = 2)]
    pub fingers: usize,
    /// 兼容旧配置保留；连续滚动使用较小的意图死区。
    #[deprecated(since = "0.6.3")]
    #[serde(default = "default_distance_pct")]
    #[setting(ignore)]
    pub distance_pct: f64,
    /// 滚动超过窗口条带末端时触发工作区切换。
    #[serde(default)]
    #[setting(label = "延续为工作区轻扫", order = 3)]
    pub propagate_to_workspace_swipe: bool,
    /// 释放时触发一次工作区切换所需的边缘位移，按工作区宽度的比例计算。
    #[serde(default = "default_overscroll_threshold")]
    #[setting(label = "工作区切换阈值 (%)", scale = 100.0, order = 6)]
    pub workspace_switch_threshold: f64,
}

impl Default for ScrollingGestureSettings {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            enabled: false,
            animate: None,
            invert_horizontal: false,
            vertical_tolerance: default_swipe_vertical_tolerance(),
            fingers: default_swipe_fingers(),
            distance_pct: default_distance_pct(),
            propagate_to_workspace_swipe: false,
            workspace_switch_threshold: default_overscroll_threshold(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum StackDefaultOrientation {
    #[setting(label = "与父布局垂直")]
    Perpendicular,
    #[setting(label = "与父布局相同")]
    Same,
    #[setting(label = "水平")]
    Horizontal,
    #[setting(label = "垂直")]
    Vertical,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct StackSettings {
    #[serde(flatten)]
    #[setting(ignore)]
    pub base: BaseLayoutSettings,
    /// 堆叠偏移：每个堆叠窗口的偏移量（单位：点）。
    /// 使用增强堆叠系统时，这能为堆叠中的每个窗口形成可见边缘，
    /// 同时保持焦点窗口完整可见。建议使用 30–50 点以获得良好辨识度。
    #[serde(default = "default_stack_offset")]
    #[setting(label = "窗口偏移 (点)", aliases = "stack offset overlap")]
    pub stack_offset: f64,

    /// 堆叠窗口时的默认方向行为。
    /// 可选值：
    /// - "perpendicular"（默认）：选择与父布局垂直的方向
    /// - "same"：使用与父布局相同的方向
    /// - "horizontal"/"vertical"：显式指定方向
    #[serde(default = "default_stack_orientation")]
    #[setting(label = "方向", choices)]
    pub default_orientation: StackDefaultOrientation,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GapSettings {
    #[serde(default)]
    pub outer: OuterGaps,
    #[serde(default)]
    pub inner: InnerGaps,
    #[serde(default)]
    pub per_display: HashMap<String, GapOverride>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct OuterGaps {
    #[serde(default)]
    #[setting(label = "上方", custom)]
    pub top: f64,
    #[serde(default)]
    #[setting(label = "左侧", custom)]
    pub left: f64,
    #[serde(default)]
    #[setting(label = "下方", custom)]
    pub bottom: f64,
    #[serde(default)]
    #[setting(label = "右侧", custom)]
    pub right: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, ConfigSchema)]
#[serde(deny_unknown_fields)]
pub struct InnerGaps {
    #[serde(default)]
    #[setting(label = "水平", custom)]
    pub horizontal: f64,
    #[serde(default)]
    #[setting(label = "垂直", custom)]
    pub vertical: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GapOverride {
    #[serde(default)]
    pub outer: Option<OuterGaps>,
    #[serde(default)]
    pub inner: Option<InnerGaps>,
}

impl Default for StackSettings {
    fn default() -> Self {
        Self {
            base: BaseLayoutSettings::default(),
            stack_offset: default_stack_offset(),
            default_orientation: default_stack_orientation(),
        }
    }
}

impl Default for MasterStackSettings {
    fn default() -> Self {
        Self {
            base: BaseLayoutSettings::default(),
            master_ratio: default_master_stack_ratio(),
            master_count: default_master_stack_count(),
            master_side: MasterStackSide::Left,
            new_window_placement: default_master_stack_new_window_placement(),
            master_arrangement: None,
            stack_arrangement: None,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.animation_duration < 0.0 {
            issues.push(format!(
                "animation_duration must be non-negative, got {}",
                self.animation_duration
            ));
        }

        if self.animation_fps <= 0.0 {
            issues.push(format!(
                "animation_fps must be positive, got {}",
                self.animation_fps
            ));
        }

        issues.extend(self.layout.validate());
        issues.extend(self.ui.stack_line.validate());

        if !(0.10..=0.45).contains(&self.drag_drop.drop_zone_fraction) {
            issues.push(format!(
                "drag_drop.drop_zone_fraction must be between 0.10 and 0.45, got {}",
                self.drag_drop.drop_zone_fraction
            ));
        }

        if self.gestures.swipe_vertical_tolerance < 0.0 {
            issues.push(format!(
                "gestures.swipe_vertical_tolerance must be non-negative, got {}",
                self.gestures.swipe_vertical_tolerance
            ));
        }

        issues
    }
}

impl LayoutSettings {
    pub fn base_for(&self, mode: LayoutMode) -> &BaseLayoutSettings {
        match mode {
            LayoutMode::Traditional => &self.traditional.base,
            LayoutMode::Bsp => &self.bsp.base,
            LayoutMode::Stack => &self.stack.base,
            LayoutMode::MasterStack => &self.master_stack.base,
            LayoutMode::Scrolling => &self.scrolling.base,
            LayoutMode::Floating => &self.base,
        }
    }

    pub fn window_insertion_point_for(&self, mode: LayoutMode) -> WindowInsertionPoint {
        self.base_for(mode)
            .window_insertion_point
            .or(self.base.window_insertion_point)
            .unwrap_or_default()
    }

    pub fn resolved_base_for(&self, mode: LayoutMode) -> BaseLayoutSettings {
        BaseLayoutSettings {
            window_insertion_point: Some(self.window_insertion_point_for(mode)),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        issues.extend(self.stack.validate());

        issues.extend(self.master_stack.validate());

        issues.extend(self.gaps.validate());

        issues.extend(self.scrolling.validate());

        issues
    }
}

impl ScrollingLayoutSettings {
    pub fn widths_for_display(&self, display_uuid: Option<&str>) -> (f64, f64, f64) {
        let override_ = display_uuid.and_then(|uuid| self.per_display.get(uuid));
        (
            override_.and_then(|o| o.column_width_ratio).unwrap_or(self.column_width_ratio),
            override_
                .and_then(|o| o.min_column_width_ratio)
                .unwrap_or(self.min_column_width_ratio),
            override_
                .and_then(|o| o.max_column_width_ratio)
                .unwrap_or(self.max_column_width_ratio),
        )
    }

    pub fn effective_for_display(&self, display_uuid: Option<&str>) -> Self {
        let mut resolved = self.clone();
        (
            resolved.column_width_ratio,
            resolved.min_column_width_ratio,
            resolved.max_column_width_ratio,
        ) = self.widths_for_display(display_uuid);
        resolved.per_display.clear();
        resolved
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        Self::validate_widths("layout.scrolling", self.widths_for_display(None), &mut issues);
        for uuid in self.per_display.keys() {
            Self::validate_widths(
                &format!("layout.scrolling.per_display[{uuid}]"),
                self.widths_for_display(Some(uuid)),
                &mut issues,
            );
        }

        if self.gestures.vertical_tolerance < 0.0 {
            issues.push(format!(
                "layout.scrolling.gestures.vertical_tolerance must be non-negative, got {}",
                self.gestures.vertical_tolerance
            ));
        }

        issues
    }

    fn validate_widths(path: &str, (ratio, min, max): (f64, f64, f64), issues: &mut Vec<String>) {
        if !(0.0..=1.0).contains(&ratio) {
            issues.push(format!(
                "{path}.column_width_ratio must be between 0.0 and 1.0, got {ratio}",
            ));
        }
        if !(0.0..=1.0).contains(&min) {
            issues.push(format!(
                "{path}.min_column_width_ratio must be between 0.0 and 1.0, got {min}",
            ));
        }
        if !(0.0..=1.0).contains(&max) {
            issues.push(format!(
                "{path}.max_column_width_ratio must be between 0.0 and 1.0, got {max}",
            ));
        }
        if min > max {
            issues.push(format!(
                "{path}.min_column_width_ratio ({min}) must be <= max_column_width_ratio ({max})",
            ));
        }
        if !(min..=max).contains(&ratio) {
            issues.push(format!(
                "{path}.column_width_ratio ({ratio}) must be within min/max bounds",
            ));
        }
    }
}

impl StackSettings {
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.stack_offset < 0.0 {
            issues.push(format!(
                "stack_offset must be non-negative, got {}",
                self.stack_offset
            ));
        }

        issues
    }
}

impl MasterStackSettings {
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !(0.05..=0.95).contains(&self.master_ratio) {
            issues.push(format!(
                "master_stack.master_ratio must be between 0.05 and 0.95, got {}",
                self.master_ratio
            ));
        }

        if self.master_count == 0 {
            issues.push("master_stack.master_count must be at least 1".to_string());
        }

        issues
    }
}

impl GapSettings {
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        issues.extend(self.outer.validate());

        issues.extend(self.inner.validate());

        for (uuid, overrides) in &self.per_display {
            if let Some(outer) = &overrides.outer {
                for issue in outer.validate() {
                    issues.push(format!("per_display[{uuid}] {issue}"));
                }
            }
            if let Some(inner) = &overrides.inner {
                for issue in inner.validate() {
                    issues.push(format!("per_display[{uuid}] {issue}"));
                }
            }
        }

        issues
    }

    pub fn effective_for_display(&self, display_uuid: Option<&str>) -> GapSettings {
        let mut resolved = GapSettings {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
            per_display: HashMap::default(),
        };
        if let Some(uuid) = display_uuid {
            if let Some(overrides) = self.per_display.get(uuid) {
                if let Some(outer_override) = &overrides.outer {
                    resolved.outer = outer_override.clone();
                }
                if let Some(inner_override) = &overrides.inner {
                    resolved.inner = inner_override.clone();
                }
            }
        }
        resolved
    }
}

impl OuterGaps {
    pub fn validate(&self) -> Vec<String> {
        [
            ("top", self.top),
            ("left", self.left),
            ("bottom", self.bottom),
            ("right", self.right),
        ]
        .into_iter()
        .filter(|(_, value)| *value < 0.0)
        .map(|(name, value)| format!("outer.{name} gap must be non-negative, got {value}"))
        .collect()
    }
}

impl InnerGaps {
    pub fn validate(&self) -> Vec<String> {
        [("horizontal", self.horizontal), ("vertical", self.vertical)]
            .into_iter()
            .filter(|(_, value)| *value < 0.0)
            .map(|(name, value)| format!("inner.{name} gap must be non-negative, got {value}"))
            .collect()
    }
}

fn yes() -> bool { true }

fn default_stack_offset() -> f64 { 40.0 }

pub fn default_stack_orientation() -> StackDefaultOrientation {
    StackDefaultOrientation::Perpendicular
}

fn default_master_stack_new_window_placement() -> MasterStackNewWindowPlacement {
    MasterStackNewWindowPlacement::Master
}

fn default_animation_duration() -> f64 { 0.3 }

fn default_animation_fps() -> f64 { 100.0 }

fn default_layout_folder() -> PathBuf { PathBuf::from("~/.config/rift/layouts") }

fn default_workspace_count() -> usize { 4 }

fn default_workspace_names() -> Vec<String> {
    vec![
        "Main".to_string(),
        "Development".to_string(),
        "Communication".to_string(),
        "Utilities".to_string(),
    ]
}

// Interpreted as normalized fraction when <= 1.0. If > 1.0 and <= 100.0,
// it is treated as a percentage (e.g. 40.0 -> 0.40).
fn default_swipe_vertical_tolerance() -> f64 { 0.4 }
fn default_swipe_fingers() -> usize { 3 }
fn default_distance_pct() -> f64 { 0.08 }
fn default_overscroll_threshold() -> f64 { 0.55 }

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, ConfigEnum)]
#[serde(rename_all = "snake_case")]
pub enum HapticPattern {
    #[setting(label = "通用")]
    Generic,
    #[setting(label = "对齐")]
    Alignment,
    #[default]
    #[setting(label = "级别变化")]
    LevelChange,
}

impl ConfigSource {
    pub fn keymap(&self, mode: &str) -> Option<&BTreeMap<String, WmCommand>> {
        if mode == "default" {
            Some(&self.keys)
        } else {
            self.binding_modes.get(mode)
        }
    }

    pub fn keymap_mut(&mut self, mode: &str) -> Result<&mut BTreeMap<String, WmCommand>, String> {
        if mode == "default" {
            Ok(&mut self.keys)
        } else {
            self.binding_modes.get_mut(mode).ok_or_else(|| "Keymap no longer exists".into())
        }
    }

    fn check_keymap_name(&self, name: &str, old: Option<&str>) -> Result<(), String> {
        if name.is_empty()
            || name == "default"
            || self.binding_modes.contains_key(name) && old != Some(name)
        {
            Err("Choose a unique keymap name".into())
        } else {
            Ok(())
        }
    }

    pub fn create_keymap(&mut self, name: String) -> Result<(), String> {
        self.check_keymap_name(&name, None)?;
        self.binding_modes.insert(name, Default::default());
        Ok(())
    }

    pub fn rename_keymap(&mut self, old: &str, name: String) -> Result<(), String> {
        self.check_keymap_name(&name, Some(old))?;
        let bindings = self.binding_modes.remove(old).ok_or("Mode no longer exists")?;
        self.binding_modes.insert(name.clone(), bindings);
        self.rewrite_keymap_references(old, &name);
        Ok(())
    }

    pub fn delete_keymap(&mut self, name: &str) -> Result<(), String> {
        self.binding_modes.remove(name).ok_or("Mode no longer exists")?;
        self.rewrite_keymap_references(name, "default");
        Ok(())
    }

    fn rewrite_keymap_references(&mut self, old: &str, target: &str) {
        use rift_protocol::ReactorCommand;

        use crate::actor::reactor::Command;
        use crate::actor::wm_controller::WmCmd;
        fn rewrite(command: &mut WmCommand, old: &str, target: &str) {
            match command {
                WmCommand::Wm(WmCmd::BindingMode(mode))
                | WmCommand::ReactorCommand(Command::Reactor(ReactorCommand::BindingMode(mode)))
                    if mode == old =>
                {
                    target.clone_into(mode)
                }
                WmCommand::Sequence(commands) => {
                    commands.iter_mut().for_each(|command| rewrite(command, old, target))
                }
                _ => {}
            }
        }
        for command in self
            .keys
            .values_mut()
            .chain(self.binding_modes.values_mut().flat_map(|map| map.values_mut()))
        {
            rewrite(command, old, target);
        }
    }
}
