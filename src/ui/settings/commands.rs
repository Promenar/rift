// 简体中文界面修改：本文件的显示文案或相关界面测试已本地化。
use cgs::collection::{EditableList, SettingsList};
use cgs::controls::{AddRemoveControl, Button, IconButton, NumberField, Popup, SheetActions};
use cgs::input::{KeyRecorder, KeyShortcut, ModifierRecorder, Modifiers, SearchField, TextField};
use cgs::layout::{Disclosure, HStack, VStack};
use cgs::menu::{Menu, MenuItem};
use cgs::navigation::HeaderControls;
use cgs::settings::{Section, SettingsPage, SettingsRow, Validation, ValidationMessage};
use cgs::text::{Caption, Label, SecondaryLabel, SectionTitle};
use cgs::view::{NativeControl, NativeView, PageHost, WeakView};
use cgs::window::Sheet;
use cgs::{Insets, Ui};
use rift_protocol::{
    Direction, DisplaySelector, LayoutCommand as L, ReactorCommand as R, ResizeOrientation,
};

use super::*;
use crate::actor::reactor::Command;
use crate::actor::wm_controller::{ConfiguredLayoutCommand, ExecCmd, WmCmd, WmCommand};
use crate::common::config::{ConfigSource, WorkspaceSelector};

fn layout(cmd: L) -> WmCommand { WmCommand::ReactorCommand(Command::Layout(cmd)) }
fn reactor(cmd: R) -> WmCommand { WmCommand::ReactorCommand(Command::Reactor(cmd)) }
fn actions() -> &'static [(&'static str, WmCommand)] {
    static ACTIONS: std::sync::LazyLock<Vec<(&'static str, WmCommand)>> =
        std::sync::LazyLock::new(|| {
            vec![
                ("聚焦 · 移动焦点", layout(L::MoveFocus(Direction::Left))),
                ("聚焦 · 下一个窗口", layout(L::NextWindow)),
                ("聚焦 · 上一个窗口", layout(L::PrevWindow)),
                ("聚焦 · 上一级", layout(L::Ascend)),
                ("聚焦 · 下一级", layout(L::Descend)),
                ("窗口 · 移动窗口", layout(L::MoveNode(Direction::Left))),
                ("窗口 · 合并窗口", layout(L::JoinWindow(Direction::Left))),
                (
                    "窗口 · 纳入或移出堆叠",
                    layout(L::ConsumeOrExpelWindow(Direction::Left)),
                ),
                ("窗口 · 切换浮动状态", layout(L::ToggleWindowFloating)),
                ("窗口 · 切换平铺与浮动窗口焦点", layout(L::ToggleFocusFloating)),
                ("窗口 · 切换全屏", layout(L::ToggleFullscreen)),
                (
                    "窗口 · 在间距内切换全屏",
                    layout(L::ToggleFullscreenWithinGaps),
                ),
                (
                    "窗口 · 放大",
                    layout(L::ResizeWindowGrow(ResizeOrientation::Smart)),
                ),
                (
                    "窗口 · 缩小",
                    layout(L::ResizeWindowShrink(ResizeOrientation::Smart)),
                ),
                ("窗口 · 按指定量调整尺寸", layout(L::ResizeWindowBy { amount: 10.0 })),
                ("窗口 · 居中所选窗口", layout(L::CenterSelection)),
                ("窗口 · 关闭", WmCommand::Wm(WmCmd::CloseWindow)),
                (
                    "工作区 · 切换工作区",
                    WmCommand::Wm(WmCmd::SwitchToWorkspace(WorkspaceSelector::Index(0))),
                ),
                (
                    "工作区 · 将窗口移至工作区",
                    WmCommand::Wm(WmCmd::MoveWindowToWorkspace(WorkspaceSelector::Index(0))),
                ),
                ("工作区 · 下一个", WmCommand::Wm(WmCmd::NextWorkspace)),
                ("工作区 · 上一个", WmCommand::Wm(WmCmd::PrevWorkspace)),
                ("工作区 · 创建", WmCommand::Wm(WmCmd::CreateWorkspace)),
                (
                    "工作区 · 上次使用的工作区",
                    WmCommand::Wm(WmCmd::SwitchToLastWorkspace),
                ),
                (
                    "工作区 · 设置布局",
                    layout(L::SetWorkspaceLayout {
                        workspace: None,
                        mode: rift_protocol::LayoutMode::Traditional,
                    }),
                ),
                (
                    "显示器 · 聚焦显示器",
                    reactor(R::FocusDisplay(DisplaySelector::Direction(Direction::Left))),
                ),
                (
                    "显示器 · 将窗口移至显示器",
                    reactor(R::MoveWindowToDisplay {
                        selector: DisplaySelector::Direction(Direction::Left),
                        window_id: None,
                    }),
                ),
                (
                    "显示器 · 将工作区移至显示器",
                    reactor(R::MoveWorkspaceToDisplay {
                        selector: DisplaySelector::Direction(Direction::Left),
                        wrap_around: false,
                    }),
                ),
                (
                    "显示器 · 将指针移至显示器",
                    reactor(R::MoveMouseToDisplay(DisplaySelector::Direction(
                        Direction::Left,
                    ))),
                ),
                (
                    "空间 · 切换 macOS 空间",
                    reactor(R::SwitchSpace(Direction::Left)),
                ),
                (
                    "空间 · 切换平铺状态",
                    WmCommand::Wm(WmCmd::ToggleSpaceActivated),
                ),
                ("布局 · 切换堆叠", layout(L::ToggleStack)),
                ("布局 · 切换方向", layout(L::ToggleOrientation)),
                ("布局 · 均分拆分", layout(L::Balance)),
                ("布局 · 拆分窗口", layout(L::UnjoinWindows)),
                ("布局 · 滚动条带", layout(L::ScrollStrip { delta: 100.0 })),
                ("布局 · 对齐条带", layout(L::SnapStrip)),
                (
                    "布局 · 切换预设列宽",
                    WmCommand::ConfiguredLayout(ConfiguredLayoutCommand::SwitchPresetColumnWidth),
                ),
                (
                    "布局 · 调整主区域比例",
                    layout(L::AdjustMasterRatio(0.05)),
                ),
                (
                    "布局 · 调整主区域窗口数",
                    layout(L::AdjustMasterCount { delta: 1 }),
                ),
                ("布局 · 移至主区域", layout(L::PromoteToMaster)),
                ("布局 · 交换主区域与堆叠区域", layout(L::SwapMasterStack)),
                (
                    "总览 · 所有工作区",
                    WmCommand::Wm(WmCmd::ShowMissionControlAll),
                ),
                (
                    "总览 · 当前工作区",
                    WmCommand::Wm(WmCmd::ShowMissionControlCurrent),
                ),
                ("总览 · 关闭", WmCommand::Wm(WmCmd::DismissMissionControl)),
                (
                    "Rift · 快捷键模式",
                    WmCommand::Wm(WmCmd::BindingMode("default".into())),
                ),
                (
                    "Rift · 运行命令",
                    WmCommand::Wm(WmCmd::Exec(ExecCmd::String(String::new()))),
                ),
                ("Rift · 重新加载配置", WmCommand::Wm(WmCmd::ReloadConfig)),
                ("Rift · 打开设置", reactor(R::OpenSettings)),
                ("Rift · 保存并退出", reactor(R::SaveAndExit)),
            ]
        });
    &ACTIONS
}
fn same_action(a: &WmCommand, b: &WmCommand) -> bool {
    use std::mem::discriminant as d;
    match (a, b) {
        (WmCommand::Wm(a), WmCommand::Wm(b)) => d(a) == d(b),
        (WmCommand::ConfiguredLayout(a), WmCommand::ConfiguredLayout(b)) => d(a) == d(b),
        (
            WmCommand::ReactorCommand(Command::Layout(a)),
            WmCommand::ReactorCommand(Command::Layout(b)),
        ) => d(a) == d(b),
        (
            WmCommand::ReactorCommand(Command::Reactor(a)),
            WmCommand::ReactorCommand(Command::Reactor(b)),
        ) => d(a) == d(b),
        _ => false,
    }
}
fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Left => "左",
        Direction::Right => "右",
        Direction::Up => "上",
        Direction::Down => "下",
    }
}

fn action_name(cmd: &WmCommand) -> String {
    if let WmCommand::Sequence(commands) = cmd {
        return commands.iter().map(action_name).collect::<Vec<_>>().join(" → ");
    }
    let title = actions()
        .iter()
        .find(|(_, a)| same_action(a, cmd))
        .map(|(name, _)| name.split_once(" · ").map_or(*name, |(_, action)| action).to_string())
        .unwrap_or("自定义命令".into());
    let argument = match cmd {
        WmCommand::ReactorCommand(Command::Layout(
            L::MoveFocus(d) | L::MoveNode(d) | L::JoinWindow(d) | L::ConsumeOrExpelWindow(d),
        )) => direction_name(*d).into(),
        WmCommand::Wm(WmCmd::SwitchToWorkspace(w) | WmCmd::MoveWindowToWorkspace(w)) => match w {
            WorkspaceSelector::Index(i) => (i + 1).to_string(),
            WorkspaceSelector::Name(n) => n.clone(),
        },
        WmCommand::Wm(WmCmd::BindingMode(n)) => n.clone(),
        WmCommand::ReactorCommand(Command::Reactor(
            R::FocusDisplay(target)
            | R::MoveMouseToDisplay(target)
            | R::MoveWindowToDisplay { selector: target, .. }
            | R::MoveWorkspaceToDisplay { selector: target, .. },
        )) => match target {
            DisplaySelector::Direction(direction) => direction_name(*direction).into(),
            DisplaySelector::Index(index) => (index + 1).to_string(),
            DisplaySelector::Uuid(_) => "（指定显示器）".into(),
        },
        _ => String::new(),
    };
    if argument.is_empty() {
        title
    } else {
        format!("{title} {argument}")
    }
}

fn glyphs(key: &str, s: &ConfigSource) -> String {
    fn tokens(key: &str) -> String {
        key.split('+')
            .map(|part| match part.trim() {
                "Alt" | "Option" => "⌥",
                "Shift" => "⇧",
                "Ctrl" | "Control" => "⌃",
                "Meta" | "Cmd" | "Command" => "⌘",
                "Left" | "ArrowLeft" => "←",
                "Right" | "ArrowRight" => "→",
                "Up" | "ArrowUp" => "↑",
                "Down" | "ArrowDown" => "↓",
                "Comma" => ",",
                "Slash" => "/",
                "Equal" => "=",
                "Minus" => "−",
                "Enter" | "Return" => "↩",
                "Tab" => "⇥",
                "Space" => "␣",
                value => value,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    key.split('+')
        .map(|part| {
            s.modifier_combinations
                .get(part.trim())
                .map(|v| tokens(v))
                .unwrap_or_else(|| tokens(part))
        })
        .collect::<Vec<_>>()
        .join(" ")
}
const MODIFIER_NAMES: [(Modifiers, &str); 4] = [
    (Modifiers::Control, "Ctrl"),
    (Modifiers::Option, "Alt"),
    (Modifiers::Shift, "Shift"),
    (Modifiers::Command, "Meta"),
];

pub(super) fn to_rift_modifiers(flags: Modifiers) -> crate::sys::hotkey::Modifiers {
    let mut result = crate::sys::hotkey::Modifiers::empty();
    for (native, rift) in modifier_flags() {
        if flags.contains(native) {
            result.insert(rift);
        }
    }
    result
}

pub(super) fn from_rift_modifiers(flags: crate::sys::hotkey::Modifiers) -> Modifiers {
    let mut result = Modifiers::empty();
    for (native, rift) in modifier_flags() {
        if flags.intersects(rift) {
            result.insert(native);
        }
    }
    result
}

fn modifier_flags() -> [(Modifiers, crate::sys::hotkey::Modifiers); 4] {
    use crate::sys::hotkey::Modifiers as M;
    [
        (Modifiers::Control, M::CONTROL),
        (Modifiers::Option, M::ALT),
        (Modifiers::Shift, M::SHIFT),
        (Modifiers::Command, M::META),
    ]
}

#[derive(Clone, PartialEq)]
struct BindingRow {
    key: String,
    shortcut: String,
    action: String,
}

pub(super) fn recorded_key(key: &KeyShortcut) -> Option<String> {
    let code = crate::sys::hotkey::cg_keycode_to_keycode(key.key.code)?;
    let mut modifiers = Vec::new();
    for (flag, name) in MODIFIER_NAMES {
        if key.modifiers.contains(flag) {
            modifiers.push(name.to_string());
        }
    }
    modifiers.push(code.to_string());
    Some(modifiers.join(" + "))
}

pub(super) fn keyboard(ui: Ui, model: &Rc<Model>) -> Page {
    let mut f = FormBuilder::new(ui, model);
    let mode = Rc::new(RefCell::new("default".to_string()));
    let query = Rc::new(RefCell::new(String::new()));
    let table =
        SettingsList::<BindingRow>::new(&ui, |row| row.action.clone(), |row| row.shortcut.clone())
            .full_length()
            .trailing_summary()
            .empty_message("此快捷键集中没有快捷键");
    let weak_model = Rc::downgrade(model);
    let mode_edit = mode.clone();
    let editing_table = Rc::new(RefCell::new(Weak::<SettingsList<BindingRow>>::new()));
    let weak_editing_table = editing_table.clone();
    let edit_binding: Rc<dyn Fn(usize)> = Rc::new(move |i| {
        if let Some(model) = weak_model.upgrade() {
            let item =
                weak_editing_table.borrow().upgrade().and_then(|table| table.row(i)).and_then(
                    |row| {
                        model
                            .source
                            .borrow()
                            .keymap(&mode_edit.borrow())
                            .and_then(|map| map.get(&row.key))
                            .cloned()
                            .map(|cmd| (row.key, cmd))
                    },
                );
            if let Some((key, cmd)) = item {
                binding_sheet(ui, &model, mode_edit.borrow().clone(), Some(key), cmd);
            }
        }
    });
    let action = edit_binding.clone();
    let table = Rc::new(table.on_open(move |i| action(i)));
    *editing_table.borrow_mut() = Rc::downgrade(&table);
    table.min_height(80.0);
    table.flexible_height(160.0);
    let names = Rc::new(RefCell::new(vec!["default".to_string()]));
    let selected_mode = mode.clone();
    let mode_names = names.clone();
    let weak_model = Rc::downgrade(model);
    let weak_table = Rc::downgrade(&table);
    let mode_query = query.clone();
    let popup = Rc::new(Popup::new(&ui).on_change(move |i| {
        if let Some(name) = mode_names.borrow().get(i) {
            *selected_mode.borrow_mut() = name.clone();
        }
        if let (Some(model), Some(table)) = (weak_model.upgrade(), weak_table.upgrade()) {
            table.set_rows_if_changed(binding_rows(
                &model.source.borrow(),
                &selected_mode.borrow(),
                &mode_query.borrow(),
            ));
        }
    }));
    let weak_popup = Rc::downgrade(&popup);
    let weak_table = Rc::downgrade(&table);
    let current_mode = mode.clone();
    let sync_query = query.clone();
    f.sync.push(Box::new(move |s| {
        let mut modes = vec!["default".to_string()];
        modes.extend(s.binding_modes.keys().cloned());
        if !modes.contains(&current_mode.borrow()) {
            *current_mode.borrow_mut() = "default".into();
        }
        if let Some(popup) = weak_popup.upgrade() {
            popup.set_items(modes.iter().map(|v| {
                if v == "default" {
                    "默认"
                } else {
                    v.as_str()
                }
            }));
            popup
                .set_selected(modes.iter().position(|v| v == &*current_mode.borrow()).unwrap_or(0));
        }
        *names.borrow_mut() = modes;
        let rows = binding_rows(s, &current_mode.borrow(), &sync_query.borrow());
        if let Some(table) = weak_table.upgrade() {
            table.set_rows_if_changed(rows);
        }
    }));
    let weak_model = Rc::downgrade(model);
    let selected_mode = mode.clone();
    let controls = AddRemoveControl::new(&ui).on_add(move || {
        if let Some(model) = weak_model.upgrade() {
            binding_sheet(
                ui,
                &model,
                selected_mode.borrow().clone(),
                None,
                layout(L::MoveFocus(Direction::Left)),
            );
        }
    });
    let weak_table = Rc::downgrade(&table);
    let edit_action = edit_binding.clone();
    let edit = IconButton::new(&ui, "pencil", "编辑快捷键…").on_click(move || {
        if let Some(i) = weak_table.upgrade().and_then(|table| table.selection()) {
            edit_action(i);
        }
    });
    let error = Rc::new(ValidationMessage::new(&ui));
    let weak_error = Rc::downgrade(&error);
    let weak_model = Rc::downgrade(model);
    let weak_table = Rc::downgrade(&table);
    let selected_mode = mode.clone();
    let remove_binding: Rc<dyn Fn()> = Rc::new(move || {
        let Some(table) = weak_table.upgrade() else {
            return;
        };
        let Some(row) = table.selection().and_then(|i| table.row(i)) else {
            return;
        };
        let name = selected_mode.borrow().clone();
        Model::submit(
            &weak_model,
            Box::new(move |s| {
                s.keymap_mut(&name)?.remove(&row.key);
                Ok(())
            }),
            weak_error.clone(),
        );
    });
    let action = remove_binding.clone();
    let controls = Rc::new(controls.on_remove(move || action()));
    let weak_table = Rc::downgrade(&table);
    let duplicate_table = Rc::downgrade(&table);
    let duplicate_model = Rc::downgrade(model);
    let duplicate_mode = mode.clone();
    let menu = Menu::new(&ui)
        .item(MenuItem::new(&ui, "编辑快捷键…").on_click(move || {
            if let Some(index) = weak_table.upgrade().and_then(|table| table.selection()) {
                edit_binding(index);
            }
        }))
        .item(MenuItem::new(&ui, "复制").on_click(move || {
            if let (Some(model), Some(table)) =
                (duplicate_model.upgrade(), duplicate_table.upgrade())
            {
                let mode = duplicate_mode.borrow().clone();
                let command = table.selection().and_then(|index| {
                    table.row(index).and_then(|row| {
                        model
                            .source
                            .borrow()
                            .keymap(&mode)
                            .and_then(|map| map.get(&row.key))
                            .cloned()
                    })
                });
                if let Some(command) = command {
                    binding_sheet(ui, &model, mode, None, command);
                }
            }
        }))
        .item(MenuItem::new(&ui, "移除").on_click(move || remove_binding()));
    table.set_context_menu(menu);
    controls.set_remove_enabled(false);
    edit.set_enabled(false);
    let weak_controls = Rc::downgrade(&controls);
    let weak_edit = WeakView::new(&edit);
    table.set_on_select(move |selection| {
        if let Some(controls) = weak_controls.upgrade() {
            controls.set_remove_enabled(selection.is_some());
        }
        weak_edit.set_enabled(selection.is_some());
    });
    let menu = Menu::new(&ui);
    let mut managed = Vec::new();
    for (title, operation) in [
        ("新建快捷键集…", KeymapOperation::Create),
        ("重命名快捷键集…", KeymapOperation::Rename),
        ("删除快捷键集…", KeymapOperation::Delete),
    ] {
        let weak_model = Rc::downgrade(model);
        let mode = mode.clone();
        let item = MenuItem::new(&ui, title).on_click(move || {
            if let Some(model) = weak_model.upgrade() {
                mode_sheet(ui, &model, mode.borrow().clone(), operation);
            }
        });
        if operation != KeymapOperation::Create {
            managed.push(item.weak());
        }
        menu.add(item);
    }
    let current_mode = mode.clone();
    let menu = menu.on_tracking(move |_| {
        for item in &managed {
            item.set_enabled(*current_mode.borrow() != "default");
        }
    });
    let management = Popup::actions(&ui, "快捷键集操作", menu);
    let combinations = modifier_combinations(&mut f, model);
    let search_model = Rc::downgrade(model);
    let search_table = Rc::downgrade(&table);
    let search = SearchField::new(&ui).placeholder("搜索快捷键").on_change(move |value| {
        *query.borrow_mut() = value.to_lowercase();
        if let (Some(model), Some(table)) = (search_model.upgrade(), search_table.upgrade()) {
            table.set_rows_if_changed(binding_rows(
                &model.source.borrow(),
                &mode.borrow(),
                &query.borrow(),
            ));
        }
    });
    let header =
        Rc::new(HeaderControls::selector("快捷键集", popup, search).actions(management));
    let mut page = f.finish(
        SettingsPage::new(&ui, "")
            .section(
                Section::new(&ui, "键盘快捷键")
                    .content(table)
                    .footer(error)
                    .footer(Caption::new(&ui, "⌘ Command   ⌥ Option   ⌃ Control   ⇧ Shift")),
            )
            .bottom_bar(HStack::new(&ui).push(controls).spacer(&ui).push(edit))
            .section(Disclosure::new(
                &ui,
                "高级：可复用修饰键组合",
                combinations,
            )),
    );
    page.header = Some(header);
    page
}

fn binding_rows(s: &ConfigSource, mode: &str, query: &str) -> Vec<BindingRow> {
    s.keymap(mode)
        .into_iter()
        .flat_map(|map| map.iter())
        .filter_map(|(key, cmd)| {
            let action = action_name(cmd);
            let shortcut = glyphs(key, s);
            (query.is_empty()
                || action.to_lowercase().contains(query)
                || shortcut.to_lowercase().contains(query)
                || key.to_lowercase().contains(query))
            .then(|| BindingRow {
                key: key.to_string(),
                shortcut,
                action,
            })
        })
        .collect()
}

fn save_sheet(model: &Weak<Model>, edit: SourceEdit, message: &Rc<ValidationMessage>) {
    if let Some(model) = model.upgrade() {
        let weak = Rc::downgrade(&model);
        let error = Rc::downgrade(message);
        model.submit_edit(
            edit,
            Box::new(move |result| {
                report(&error, &result);
                if let Some(model) = weak.upgrade() {
                    if result.is_ok() {
                        model.close_sheet();
                    } else if let Some(sheet) = model.sheet.borrow().as_ref() {
                        sheet.fit_content();
                    }
                }
            }),
        );
    }
}
fn show_sheet(ui: Ui, model: &Rc<Model>, title: &str, content: impl NativeView) {
    let sheet = Sheet::new(&ui, title, content);
    sheet.fit_content();
    if sheet.show(&model.env.window.borrow()) {
        *model.sheet.borrow_mut() = Some(sheet);
    }
}
fn sheet_actions(ui: Ui, model: &Rc<Model>) -> Rc<SheetActions> {
    let weak = Rc::downgrade(model);
    let actions = Rc::new(SheetActions::new(&ui));
    actions.set_on_cancel(move || {
        if let Some(model) = weak.upgrade() {
            model.close_sheet();
        }
    });
    actions
}
fn binding_sheet(ui: Ui, model: &Rc<Model>, mode: String, old: Option<String>, command: WmCommand) {
    let cancel = sheet_actions(ui, model);
    let key = Rc::new(RefCell::new(old.clone()));
    let draft = Rc::new(RefCell::new(command.clone()));
    let dirty_key = key.clone();
    let dirty_command = draft.clone();
    let initial_key = old.clone();
    let initial_command = command.clone();
    let weak_cancel = Rc::downgrade(&cancel);
    let changed: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(cancel) = weak_cancel.upgrade() {
            cancel.set_changed(
                *dirty_key.borrow() != initial_key || *dirty_command.borrow() != initial_command,
            );
        }
    });
    let key_changed = changed.clone();
    let key_cb = key.clone();
    let recorder = KeyRecorder::new(&ui).on_change(move |v| {
        *key_cb.borrow_mut() = v.as_ref().and_then(recorded_key);
        key_changed();
    });
    if let Some(old) = &old {
        recorder.set_title(&glyphs(old, &model.source.borrow()));
    }
    let host = Rc::new(PageHost::new(&ui));
    host.set_page(argument_editor(ui, model, &draft, &changed));
    let weak_host = Rc::downgrade(&host);
    let weak_model = Rc::downgrade(model);
    let edited = draft.clone();
    let action_changed = changed.clone();
    let mut descriptors = actions().to_vec();
    let current =
        descriptors
            .iter()
            .position(|(_, c)| same_action(c, &command))
            .unwrap_or_else(|| {
                descriptors.push(("当前自定义操作", command.clone()));
                descriptors.len() - 1
            });
    let templates: Vec<_> = descriptors.iter().map(|(_, c)| c.clone()).collect();
    let popup =
        Popup::new(&ui)
            .items(descriptors.iter().map(|(title, _)| *title))
            .on_change(move |i| {
                *edited.borrow_mut() = templates[i].clone();
                action_changed();
                if let (Some(host), Some(model)) = (weak_host.upgrade(), weak_model.upgrade()) {
                    host.set_page(argument_editor(ui, &model, &edited, &action_changed));
                    if let Some(sheet) = model.sheet.borrow().as_ref() {
                        sheet.fit_content();
                    }
                }
            });
    popup.set_selected(current);
    let message = Rc::new(ValidationMessage::new(&ui));
    let error = Rc::downgrade(&message);
    let weak_model = Rc::downgrade(model);
    cancel.set_primary_title("保存快捷键");
    cancel.set_on_done(move || {
        let Some(message) = error.upgrade() else {
            return;
        };
        let Some(key) = key.borrow().clone() else {
            message.set_validation(&Validation::Error("请先录制快捷键".into()));
            if let Some(model) = weak_model.upgrade()
                && let Some(sheet) = model.sheet.borrow().as_ref()
            {
                sheet.fit_content();
            }
            return;
        };
        let cmd = draft.borrow().clone();
        let old = old.clone();
        let mode = mode.clone();
        save_sheet(
            &weak_model,
            Box::new(move |s| {
                let bindings = s.keymap_mut(&mode)?;
                if old.as_ref() != Some(&key) && bindings.contains_key(&key) {
                    return Err("此快捷键已在当前快捷键集中分配".into());
                }
                if let Some(old) = old {
                    bindings.remove(&old);
                }
                bindings.insert(key, cmd);
                Ok(())
            }),
            &message,
        );
    });
    show_sheet(
        ui,
        model,
        "快捷键",
        VStack::new(&ui)
            .insets(Insets {
                top: 20.0,
                left: 20.0,
                bottom: 20.0,
                right: 20.0,
            })
            .push(SectionTitle::new(&ui, "键盘快捷键"))
            .push(Caption::new(
                &ui,
                "点击快捷键，然后按下要使用的按键。",
            ))
            .push(
                Section::new(&ui, "")
                    .row(SettingsRow::new(&ui, "快捷键", recorder))
                    .row(SettingsRow::new(&ui, "操作", popup)),
            )
            .push(host)
            .push(message)
            .push(cancel),
    );
}

fn argument_editor(
    ui: Ui,
    model: &Rc<Model>,
    draft: &Rc<RefCell<WmCommand>>,
    changed: &Rc<dyn Fn()>,
) -> Section {
    let command = draft.borrow().clone();
    let mut view = Section::new(&ui, "");
    use WmCommand::{ReactorCommand as RcCommand, Wm};
    match command.clone() {
        RcCommand(Command::Layout(
            L::MoveFocus(d) | L::MoveNode(d) | L::JoinWindow(d) | L::ConsumeOrExpelWindow(d),
        ))
        | RcCommand(Command::Reactor(R::SwitchSpace(d))) => {
            let edited = draft.clone();
            let changed = changed.clone();
            let directions = [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ];
            let popup =
                Popup::new(&ui).items(["左", "右", "上", "下"]).on_change(move |i| {
                    let d = directions[i];
                    *edited.borrow_mut() = match &command {
                        RcCommand(Command::Layout(L::MoveFocus(_))) => layout(L::MoveFocus(d)),
                        RcCommand(Command::Layout(L::MoveNode(_))) => layout(L::MoveNode(d)),
                        RcCommand(Command::Layout(L::JoinWindow(_))) => layout(L::JoinWindow(d)),
                        RcCommand(Command::Layout(L::ConsumeOrExpelWindow(_))) => {
                            layout(L::ConsumeOrExpelWindow(d))
                        }
                        _ => reactor(R::SwitchSpace(d)),
                    };

                    changed();
                });
            popup.set_selected(directions.iter().position(|v| *v == d).unwrap_or(0));
            view = view.row(SettingsRow::new(&ui, "方向", popup));
        }
        RcCommand(Command::Layout(L::ResizeWindowGrow(o) | L::ResizeWindowShrink(o))) => {
            let edited = draft.clone();
            let changed = changed.clone();
            let values = [
                ResizeOrientation::Smart,
                ResizeOrientation::Horizontal,
                ResizeOrientation::Vertical,
            ];
            let popup =
                Popup::new(&ui).items(["智能", "水平", "垂直"]).on_change(move |i| {
                    *edited.borrow_mut() =
                        if matches!(command, RcCommand(Command::Layout(L::ResizeWindowGrow(_)))) {
                            layout(L::ResizeWindowGrow(values[i]))
                        } else {
                            layout(L::ResizeWindowShrink(values[i]))
                        };

                    changed();
                });
            popup.set_selected(values.iter().position(|v| *v == o).unwrap_or(0));
            view = view.row(SettingsRow::new(&ui, "方向", popup));
        }
        Wm(WmCmd::SwitchToWorkspace(target) | WmCmd::MoveWindowToWorkspace(target)) => {
            let names: Vec<_> =
                (0..model.source.borrow().virtual_workspaces.default_workspace_count)
                    .map(|i| super::editors::workspace_name(&model.source.borrow(), i))
                    .collect();
            let initial = match target {
                WorkspaceSelector::Index(i) => i,
                WorkspaceSelector::Name(name) => model
                    .source
                    .borrow()
                    .virtual_workspaces
                    .workspace_names
                    .iter()
                    .position(|v| *v == name)
                    .unwrap_or(0),
            };
            let edited = draft.clone();
            let changed = changed.clone();
            let popup = Popup::new(&ui).items(names).on_change(move |i| {
                let target = WorkspaceSelector::Index(i);
                *edited.borrow_mut() = Wm(if matches!(command, Wm(WmCmd::SwitchToWorkspace(_))) {
                    WmCmd::SwitchToWorkspace(target)
                } else {
                    WmCmd::MoveWindowToWorkspace(target)
                });

                changed();
            });
            popup.set_selected(initial);
            view = view.row(SettingsRow::new(&ui, "工作区", popup));
        }
        RcCommand(Command::Layout(L::SetWorkspaceLayout { mode, workspace })) => {
            let values = super::pages::layouts();
            let edited = draft.clone();
            let changed = changed.clone();
            let modes: Vec<_> = values.iter().map(|(_, v)| *v).collect();
            let selected = modes.iter().position(|v| *v == mode).unwrap_or(0);
            let popup =
                Popup::new(&ui).items(values.iter().map(|(name, _)| *name)).on_change(move |i| {
                    *edited.borrow_mut() =
                        layout(L::SetWorkspaceLayout { workspace, mode: modes[i] });

                    changed();
                });
            popup.set_selected(selected);
            view = view.row(SettingsRow::new(&ui, "布局", popup));
        }
        RcCommand(Command::Reactor(
            R::FocusDisplay(target)
            | R::MoveMouseToDisplay(target)
            | R::MoveWindowToDisplay { selector: target, .. }
            | R::MoveWorkspaceToDisplay { selector: target, .. },
        )) => {
            let values = vec![
                DisplaySelector::Direction(Direction::Left),
                DisplaySelector::Direction(Direction::Right),
                DisplaySelector::Direction(Direction::Up),
                DisplaySelector::Direction(Direction::Down),
            ];
            let mut values = values;
            let mut names = vec![
                "左".to_string(),
                "右".into(),
                "上".into(),
                "下".into(),
            ];
            for display in model.env.displays.borrow().iter() {
                values.push(DisplaySelector::Uuid(display.display_uuid.clone()));
                names.push(
                    display
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("显示器 {}", display.id.as_u32())),
                );
            }
            let selected = values.iter().position(|value| *value == target).unwrap_or_else(|| {
                values.push(target);
                names.push("当前目标显示器".into());
                values.len() - 1
            });
            let edited = draft.clone();
            let changed = changed.clone();
            let popup = Popup::new(&ui).items(names).on_change(move |i| {
                let selector = values[i].clone();
                *edited.borrow_mut() = reactor(match &command {
                    RcCommand(Command::Reactor(R::FocusDisplay(_))) => R::FocusDisplay(selector),
                    RcCommand(Command::Reactor(R::MoveMouseToDisplay(_))) => {
                        R::MoveMouseToDisplay(selector)
                    }
                    RcCommand(Command::Reactor(R::MoveWorkspaceToDisplay {
                        wrap_around, ..
                    })) => R::MoveWorkspaceToDisplay {
                        selector,
                        wrap_around: *wrap_around,
                    },
                    RcCommand(Command::Reactor(R::MoveWindowToDisplay { window_id, .. })) => {
                        R::MoveWindowToDisplay {
                            selector,
                            window_id: *window_id,
                        }
                    }
                    _ => unreachable!(),
                });

                changed();
            });
            popup.set_selected(selected);
            view = view.row(SettingsRow::new(&ui, "显示器方向", popup));
        }
        RcCommand(Command::Layout(
            L::ResizeWindowBy { amount }
            | L::ScrollStrip { delta: amount }
            | L::AdjustMasterRatio(amount),
        )) => {
            let edited = draft.clone();
            let changed = changed.clone();
            let input = NumberField::new(&ui).value(amount).on_edit(move |v| {
                *edited.borrow_mut() = match &command {
                    RcCommand(Command::Layout(L::ResizeWindowBy { .. })) => {
                        layout(L::ResizeWindowBy { amount: v })
                    }
                    RcCommand(Command::Layout(L::ScrollStrip { .. })) => {
                        layout(L::ScrollStrip { delta: v })
                    }
                    _ => layout(L::AdjustMasterRatio(v)),
                };

                changed();
            });
            view = view.row(SettingsRow::new(&ui, "调整量", input));
        }
        RcCommand(Command::Layout(L::AdjustMasterCount { delta })) => {
            let edited = draft.clone();
            let changed = changed.clone();
            view = view.row(SettingsRow::new(
                &ui,
                "窗口数量变化",
                NumberField::new(&ui).integer().value(delta as f64).on_edit(move |v| {
                    *edited.borrow_mut() = layout(L::AdjustMasterCount { delta: v as i32 });

                    changed();
                }),
            ));
        }
        Wm(WmCmd::BindingMode(name)) => {
            let mut names = vec!["default".to_string()];
            names.extend(model.source.borrow().binding_modes.keys().cloned());
            let selected = names.iter().position(|v| *v == name).unwrap_or(0);
            let choices = names.clone();
            let edited = draft.clone();
            let changed = changed.clone();
            let popup = Popup::new(&ui).items(names).on_change(move |i| {
                *edited.borrow_mut() = Wm(WmCmd::BindingMode(choices[i].clone()));

                changed();
            });
            popup.set_selected(selected);
            view = view.row(SettingsRow::new(&ui, "快捷键集", popup));
        }
        Wm(WmCmd::Exec(ExecCmd::String(text))) => {
            let edited = draft.clone();
            let changed = changed.clone();
            view = view.row(SettingsRow::new(
                &ui,
                "Shell 命令",
                TextField::new(&ui).value(&text).on_change(move |v| {
                    *edited.borrow_mut() = Wm(WmCmd::Exec(ExecCmd::String(v)));

                    changed();
                }),
            ));
        }
        _ if !actions().iter().any(|(_, template)| same_action(template, &command)) => {
            view = view.content(SecondaryLabel::new(
                &ui,
                "现有参数将被保留。选择其他操作可将其替换。",
            ));
        }
        _ => {}
    }
    view
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeymapOperation {
    Create,
    Rename,
    Delete,
}

fn mode_sheet(ui: Ui, model: &Rc<Model>, old: String, operation: KeymapOperation) {
    if operation != KeymapOperation::Create && old == "default" {
        return;
    }
    let cancel = sheet_actions(ui, model);
    cancel.set_changed(operation == KeymapOperation::Delete);
    let weak_cancel = Rc::downgrade(&cancel);
    let initial = if operation == KeymapOperation::Rename {
        old.clone()
    } else {
        String::new()
    };
    let input = Rc::new(TextField::new(&ui).value(&initial).on_change(move |value| {
        if let Some(cancel) = weak_cancel.upgrade() {
            cancel.set_changed(value != initial);
        }
    }));
    let message = Rc::new(ValidationMessage::new(&ui));
    let weak_input = Rc::downgrade(&input);
    let error = Rc::downgrade(&message);
    let weak_model = Rc::downgrade(model);
    cancel.set_primary_title(if operation == KeymapOperation::Delete {
        "删除快捷键集"
    } else {
        "保存"
    });
    cancel.set_on_done(move || {
        let (Some(input), Some(message)) = (weak_input.upgrade(), error.upgrade()) else {
            return;
        };
        let name = input.get_value().trim().to_string();
        let old = old.clone();
        save_sheet(
            &weak_model,
            Box::new(move |s| match operation {
                KeymapOperation::Create => s.create_keymap(name),
                KeymapOperation::Rename => s.rename_keymap(&old, name),
                KeymapOperation::Delete => s.delete_keymap(&old),
            }),
            &message,
        );
    });
    let mut content = VStack::new(&ui).insets(Insets {
        top: 20.0,
        left: 20.0,
        bottom: 20.0,
        right: 20.0,
    });
    if operation == KeymapOperation::Delete {
        content = content.push(Label::new(
            &ui,
            "删除此快捷键集及其快捷键？相关引用将切换到默认快捷键集。",
        ));
    } else {
        content = content.push(SettingsRow::new(&ui, "名称", input));
    }
    show_sheet(ui, model, "快捷键集", content.push(message).push(cancel));
}
fn modifier_combinations(f: &mut FormBuilder, model: &Rc<Model>) -> VStack {
    let ui = f.ui;
    let name = Rc::new(TextField::new(&ui).placeholder("名称"));
    let modifiers = Rc::new(RefCell::new(String::new()));
    let changed = modifiers.clone();
    let recorder = ModifierRecorder::new(&ui).on_change(move |flags| {
        let mut tokens = Vec::new();
        for (flag, name) in MODIFIER_NAMES {
            if flags.contains(flag) {
                tokens.push(name);
            }
        }
        *changed.borrow_mut() = tokens.join(" + ");
    });
    let message = Rc::new(ValidationMessage::new(&ui));
    let error = Rc::downgrade(&message);
    let weak_model = Rc::downgrade(model);
    let weak_name = Rc::downgrade(&name);
    let add = Button::new(&ui, "添加 / 更新").on_click(move || {
        if let Some(name) = weak_name.upgrade() {
            let name = name.get_value().trim().to_string();
            let value = modifiers.borrow().clone();
            Model::submit(
                &weak_model,
                Box::new(move |s| {
                    if name.is_empty() || name.contains('+') || value.is_empty() {
                        return Err("请输入名称并录制修饰键".into());
                    }
                    s.modifier_combinations.insert(name, value);
                    Ok(())
                }),
                error.clone(),
            );
        }
    });
    let weak_model = Rc::downgrade(model);
    let error = Rc::downgrade(&message);
    let list = Rc::new(
        EditableList::new(&ui, |v: &(String, String)| format!("{} · {}", v.0, v.1))
            .full_length()
            .on_remove(move |i| {
                if let Some(model) = weak_model.upgrade() {
                    let name = model.source.borrow().modifier_combinations.keys().nth(i).cloned();
                    if let Some(name) = name {
                        Model::submit(
                            &weak_model,
                            Box::new(move |s| {
                                s.modifier_combinations.remove(&name);
                                Ok(())
                            }),
                            error.clone(),
                        );
                    }
                }
            }),
    );
    let weak_list = Rc::downgrade(&list);
    f.sync.push(Box::new(move |s| {
        if let Some(list) = weak_list.upgrade() {
            list.set_items(
                s.modifier_combinations.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            );
        }
    }));
    VStack::new(&ui)
        .push(list)
        .push(HStack::new(&ui).push(name).push(recorder).push(add))
        .push(message)
}
