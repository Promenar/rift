use std::path::PathBuf;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    Direction, DisplaySelector, LayoutMode, ResizeOrientation, RestoreScope, RestoreSource,
    WindowId, WorkspaceSelector,
};

/// Editable argument types shared by command discovery and native settings.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandArgument {
    Direction(Direction),
    Orientation(ResizeOrientation),
    Workspace(WorkspaceSelector),
    WorkspaceIndex(usize),
    Display(DisplaySelector),
    Layout(LayoutMode),
    Keymap(String),
    Number(f64),
    Integer(i32),
    Backwards(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowSizePreset {
    Smart,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FloatingWindowSize {
    Dimensions { w: f64, h: f64 },
    Preset(FloatingWindowSizePreset),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToggleWindowFloatingOptions {
    #[serde(default)]
    pub center: bool,
    #[serde(default)]
    pub size: Option<FloatingWindowSize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, rift_config_derive::CommandCatalog)]
#[serde(rename_all = "snake_case")]
#[command(
    group = 0,
    defaults(
        Direction = "Direction::Left",
        ResizeOrientation = "ResizeOrientation::Smart",
        LayoutMode = "LayoutMode::Traditional"
    )
)]
pub enum LayoutCommand {
    #[command(group = 1, label = "聚焦下一个窗口", order = 13)]
    NextWindow,
    #[command(group = 1, label = "聚焦上一个窗口", order = 14)]
    PrevWindow,
    #[command(group = 1, label = "聚焦窗口", order = 12)]
    MoveFocus(#[serde(rename = "direction")] Direction),
    #[command(group = 1, label = "聚焦父级", order = 15)]
    Ascend,
    #[command(group = 1, label = "聚焦子级", order = 16)]
    Descend,
    #[command(label = "移动窗口", order = 0)]
    MoveNode(Direction),
    #[command(label = "合并窗口", order = 1)]
    JoinWindow(Direction),
    #[command(label = "纳入或移出窗口", order = 2)]
    ConsumeOrExpelWindow(Direction),
    #[command(label = "切换堆叠", group = 4, order = 17)]
    ToggleStack,
    #[command(label = "切换方向", group = 4, order = 18)]
    ToggleOrientation,
    #[command(label = "平衡布局", group = 4, order = 19)]
    Balance,
    #[command(label = "取消合并窗口", group = 4, order = 20)]
    UnjoinWindows,
    #[command(label = "切换焦点窗口浮动", order = 4)]
    ToggleFocusFloating,
    #[command(label = "切换浮动", order = 3)]
    ToggleWindowFloating,
    #[command(skip, alias = "ToggleWindowFloating")]
    ToggleWindowFloatingWithOptions(ToggleWindowFloatingOptions),
    #[command(label = "切换全屏", order = 5)]
    ToggleFullscreen,
    #[command(label = "间隙内全屏", order = 6)]
    ToggleFullscreenWithinGaps,
    #[command(label = "增大窗口", order = 7)]
    ResizeWindowGrow(ResizeOrientation),
    #[command(label = "缩小窗口", order = 8)]
    ResizeWindowShrink(ResizeOrientation),
    #[command(label = "调整窗口尺寸", order = 9, sample = "Self::ResizeWindowBy { amount: 10.0 }")]
    ResizeWindowBy { amount: f64 },
    #[command(
        group = 4,
        label = "滚动条带",
        order = 22,
        sample = "Self::ScrollStrip { delta: 100.0 }"
    )]
    ScrollStrip { delta: f64 },
    #[command(label = "吸附条带", group = 4, order = 23)]
    SnapStrip,
    #[command(group = 4, label = "循环预设宽度", order = 24)]
    SwitchPresetColumnWidth {
        #[serde(default)]
        backwards: bool,
    },
    #[command(label = "居中选中", order = 10)]
    CenterSelection,
    #[command(skip)]
    NextWorkspace(Option<bool>),
    #[command(skip)]
    PrevWorkspace(Option<bool>),
    #[command(skip)]
    SwitchToWorkspace(usize),
    #[command(skip)]
    MoveWindowToWorkspace {
        workspace: WorkspaceSelector,
        follow: bool,
        window_id: Option<u32>,
    },
    #[command(group = 4, label = "设置布局为", order = 21)]
    SetWorkspaceLayout {
        workspace: Option<usize>,
        mode: LayoutMode,
    },
    #[command(skip)]
    CreateWorkspace,
    #[command(skip)]
    SwitchToLastWorkspace,
    #[command(
        label = "交换指定窗口",
        order = 51,
        sample = "Self::SwapWindows(WindowId { pid: 0, idx: 1 }, WindowId { pid: 0, idx: 2 })"
    )]
    SwapWindows(WindowId, WindowId),
    /// 将 `window` 的标签移动到 `target` 标签在共享堆叠中的位置，
    /// 并把两者之间的标签向腾出的空位移动。
    #[command(
        label = "移动堆叠标签",
        order = 52,
        sample = "Self::MoveStackTab { window: WindowId { pid: 0, idx: 1 }, target: WindowId { pid: 0, idx: 2 } }"
    )]
    MoveStackTab { window: WindowId, target: WindowId },
    #[command(
        group = 4,
        label = "调整主区比例",
        order = 25,
        sample = "Self::AdjustMasterRatio(0.05)"
    )]
    AdjustMasterRatio(f64),
    #[command(
        group = 4,
        label = "调整主区数量",
        order = 26,
        sample = "Self::AdjustMasterCount { delta: 1 }"
    )]
    AdjustMasterCount { delta: i32 },
    #[command(label = "提升为主窗口", group = 4, order = 27)]
    PromoteToMaster,
    #[command(group = 4, label = "交换主区与堆栈", order = 28)]
    SwapMasterStack,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, rift_config_derive::CommandCatalog,
)]
#[serde(rename_all = "snake_case")]
#[command(defaults(
    Direction = "Direction::Left",
    DisplaySelector = "DisplaySelector::Direction(Direction::Left)"
))]
pub enum ReactorCommand {
    #[command(label = "打开设置", order = 47)]
    OpenSettings,
    #[command(skip)]
    BindingMode(String),
    #[command(label = "打印布局树", order = 48)]
    Debug,
    #[command(label = "序列化状态", order = 49)]
    Serialize,
    #[command(
        group = 4,
        label = "保存布局到文件",
        order = 54,
        sample = "Self::SaveLayout { path: std::path::PathBuf::new() }"
    )]
    SaveLayout { path: PathBuf },
    #[command(label = "保存并退出", order = 50)]
    SaveAndExit,
    #[command(
        group = 4,
        label = "从文件恢复布局",
        order = 55,
        sample = "Self::RestoreLayout { path: std::path::PathBuf::new(), scope: RestoreScope::Workspace, source: RestoreSource::SavedActiveSpace }"
    )]
    RestoreLayout {
        path: PathBuf,
        scope: RestoreScope,
        #[serde(default)]
        source: RestoreSource,
    },
    #[command(label = "切换空间", group = 2, order = 39)]
    SwitchSpace(Direction),
    #[command(skip)]
    ToggleSpaceActivated,
    #[command(
        group = 1,
        label = "聚焦指定窗口",
        order = 53,
        sample = "Self::FocusWindow { window_id: WindowId { pid: 0, idx: 1 }, window_server_id: None }"
    )]
    FocusWindow {
        window_id: WindowId,
        window_server_id: Option<u32>,
    },
    #[command(skip)]
    ShowMissionControlAll,
    #[command(skip)]
    ShowMissionControlCurrent,
    #[command(skip)]
    DismissMissionControl,
    #[command(group = 3, label = "移动指针到显示器", order = 38)]
    MoveMouseToDisplay(DisplaySelector),
    #[command(label = "聚焦显示器", group = 3, order = 35)]
    FocusDisplay(DisplaySelector),
    #[command(skip)]
    CloseWindow { window_server_id: Option<u32> },
    #[command(label = "移动窗口到显示器", group = 3, order = 36)]
    MoveWindowToDisplay {
        selector: DisplaySelector,
        window_id: Option<u32>,
    },
    /// 将当前活动工作区移动到另一台显示器。
    ///
    /// Rift 的工作区是显示器本地的，因此窗口会移动到目标显示器上序号相同的
    /// 工作区，且该工作区变为活动状态。
    #[command(label = "移动工作区到显示器", group = 3, order = 37)]
    MoveWorkspaceToDisplay {
        selector: DisplaySelector,
        /// 当方向选择器在该方向上没有更远的显示器时，从相反边缘继续。
        #[serde(default)]
        wrap_around: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricsCommand {
    ShowTiming,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigCommand {
    SetAnimate(bool),
    SetAnimationDuration(f64),
    SetAnimationFps(f64),
    SetAnimationEasing(AnimationEasing),
    SetMouseFollowsFocus(bool),
    SetMouseHidesOnFocus(bool),
    SetFocusFollowsMouse(bool),
    SetStackOffset(f64),
    SetOuterGaps {
        top: f64,
        left: f64,
        bottom: f64,
        right: f64,
    },
    SetInnerGaps {
        horizontal: f64,
        vertical: f64,
    },
    SetWorkspaceNames(Vec<String>),
    Set {
        key: String,
        value: Value,
    },
    GetConfig,
    SaveConfig,
    ReloadConfig,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    rift_config_derive::ConfigEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum AnimationEasing {
    #[default]
    #[setting(label = "缓入缓出")]
    EaseInOut,
    #[setting(label = "线性")]
    Linear,
    #[setting(label = "正弦缓入")]
    EaseInSine,
    #[setting(label = "正弦缓出")]
    EaseOutSine,
    #[setting(label = "正弦缓入缓出")]
    EaseInOutSine,
    #[setting(label = "二次缓入")]
    EaseInQuad,
    #[setting(label = "二次缓出")]
    EaseOutQuad,
    #[setting(label = "二次缓入缓出")]
    EaseInOutQuad,
    #[setting(label = "三次缓入")]
    EaseInCubic,
    #[setting(label = "三次缓出")]
    EaseOutCubic,
    #[setting(label = "三次缓入缓出")]
    EaseInOutCubic,
    #[setting(label = "四次缓入")]
    EaseInQuart,
    #[setting(label = "四次缓出")]
    EaseOutQuart,
    #[setting(label = "四次缓入缓出")]
    EaseInOutQuart,
    #[setting(label = "五次缓入")]
    EaseInQuint,
    #[setting(label = "五次缓出")]
    EaseOutQuint,
    #[setting(label = "五次缓入缓出")]
    EaseInOutQuint,
    #[setting(label = "指数缓入")]
    EaseInExpo,
    #[setting(label = "指数缓出")]
    EaseOutExpo,
    #[setting(label = "指数缓入缓出")]
    EaseInOutExpo,
    #[setting(label = "圆形缓入")]
    EaseInCirc,
    #[setting(label = "圆形缓出")]
    EaseOutCirc,
    #[setting(label = "圆形缓入缓出")]
    EaseInOutCirc,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiftCommand {
    Layout(LayoutCommand),
    Metrics(MetricsCommand),
    Reactor(ReactorCommand),
    Config(ConfigCommand),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TypedRiftCommand {
    Layout(LayoutCommand),
    Metrics(MetricsCommand),
    Reactor(ReactorCommand),
    Config(ConfigCommand),
}

#[derive(Deserialize)]
enum LegacyCommand {
    #[serde(alias = "reactor")]
    Reactor(LegacyReactorCommand),
    #[serde(alias = "config")]
    Config(ConfigCommand),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LegacyReactorCommand {
    Layout(LayoutCommand),
    Metrics(MetricsCommand),
    Reactor(ReactorCommand),
}

impl From<TypedRiftCommand> for RiftCommand {
    fn from(command: TypedRiftCommand) -> Self {
        match command {
            TypedRiftCommand::Layout(command) => Self::Layout(command),
            TypedRiftCommand::Metrics(command) => Self::Metrics(command),
            TypedRiftCommand::Reactor(command) => Self::Reactor(command),
            TypedRiftCommand::Config(command) => Self::Config(command),
        }
    }
}

impl<'de> Deserialize<'de> for RiftCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum CommandInput {
            Typed(TypedRiftCommand),
            LegacyJson(String),
        }

        match CommandInput::deserialize(deserializer)? {
            CommandInput::Typed(command) => Ok(command.into()),
            CommandInput::LegacyJson(command) => decode_legacy_command(&command),
        }
    }
}

fn decode_legacy_command<E>(command: &str) -> Result<RiftCommand, E>
where E: DeError {
    match serde_json::from_str::<LegacyCommand>(command)
        .map_err(|error| E::custom(format!("invalid legacy command JSON: {error}")))?
    {
        LegacyCommand::Config(command) => Ok(RiftCommand::Config(command)),
        LegacyCommand::Reactor(LegacyReactorCommand::Layout(command)) => {
            Ok(RiftCommand::Layout(command))
        }
        LegacyCommand::Reactor(LegacyReactorCommand::Metrics(command)) => {
            Ok(RiftCommand::Metrics(command))
        }
        LegacyCommand::Reactor(LegacyReactorCommand::Reactor(command)) => {
            Ok(RiftCommand::Reactor(command))
        }
    }
}
