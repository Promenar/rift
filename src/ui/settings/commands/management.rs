//! Shortcut modes and reusable modifier combinations.
use super::*;

pub(super) fn save_sheet(model: &Weak<Model>, edit: SourceEdit, message: &Rc<ValidationMessage>) {
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
pub(super) fn show_sheet(ui: Ui, model: &Rc<Model>, title: &str, content: impl NativeView) {
    let sheet = Sheet::new(&ui, title, content);
    sheet.fit_content();
    if sheet.show(&model.env.window.borrow()) {
        *model.sheet.borrow_mut() = Some(sheet);
    }
}
pub(super) fn sheet_actions(ui: Ui, model: &Rc<Model>) -> Rc<SheetActions> {
    let weak = Rc::downgrade(model);
    let actions = Rc::new(SheetActions::new(&ui));
    actions.set_on_cancel(move || {
        if let Some(model) = weak.upgrade() {
            model.close_sheet();
        }
    });
    actions
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum KeymapOperation {
    Create,
    Rename,
    Delete,
}

pub(super) fn mode_sheet(ui: Ui, model: &Rc<Model>, old: String, operation: KeymapOperation) {
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
    let message = inline_message(&ui);
    let weak_input = Rc::downgrade(&input);
    let error = Rc::downgrade(&message);
    let weak_model = Rc::downgrade(model);
    cancel.set_primary_title(if operation == KeymapOperation::Delete {
        "删除模式"
    } else {
        "Save"
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
    let mut content = Style::sheet(&ui);
    if operation == KeymapOperation::Delete {
        content = content.push(Label::new(
            &ui,
            "删除此模式及其快捷键？相关引用将切换到默认模式。",
        ));
    } else {
        content = content.push(SettingsRow::new(&ui, "Name", input));
    }
    show_sheet(ui, model, "快捷键模式", content.push(message).push(cancel));
}
pub(super) fn modifier_combinations(f: &mut FormBuilder, model: &Rc<Model>) -> VStack {
    let ui = f.ui;
    let name = Rc::new(TextField::new(&ui).placeholder("Name"));
    let modifiers = Rc::new(RefCell::new(String::new()));
    let changed = modifiers.clone();
    let message = inline_message(&ui);
    let (capture_model, capture_error) = (Rc::downgrade(model), Rc::downgrade(&message));
    let recorder = ModifierRecorder::new(&ui)
        .capture(move |recording| {
            let Some(model) = capture_model.upgrade() else {
                return Err("设置已关闭。".into());
            };
            let result = model
                .env
                .shortcut_capture
                .borrow()
                .as_ref()
                .map_or(Ok(()), |capture| capture(recording));
            if recording {
                report(&capture_error, &result);
            }
            result
        })
        .on_change(move |flags| {
            let mut tokens = Vec::new();
            for (flag, name) in MODIFIER_NAMES {
                if flags.contains(flag) {
                    tokens.push(name);
                }
            }
            *changed.borrow_mut() = tokens.join(" + ");
        });
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
