// 简体中文界面修改：本文件的显示文案或相关界面测试已本地化。
//! Interface preferences. Previews are event-driven native views, never windows.
use objc2_app_kit::*;

use super::interface_preview::StackPreview;
use super::*;
use crate::common::config::*;
use crate::ui::menu_bar::MenuBarPreview;

pub(super) fn menu_bar(ui: Ui, model: &Rc<Model>) -> Page {
    let mut f = FormBuilder::new(ui, model);
    f.tooltips = true;
    let indicator = Rc::new(MenuBarPreview::new(&ui));
    indicator.height(24.0);
    let width = indicator.ns_view().widthAnchor().constraintEqualToConstant(100.0);
    width.setActive(true);
    let disabled = Rc::new(Caption::single_line(&ui, "Rift 指示器已隐藏"));
    let bar = HStack::new(&ui)
        .spacing(10.0)
        .insets(Insets {
            top: 12.0,
            left: 12.0,
            bottom: 12.0,
            right: 12.0,
        })
        .push(Label::new(&ui, "").font(&NSFont::systemFontOfSize(16.0)))
        .push(Label::new(&ui, "访达").font(&NSFont::systemFontOfSize(12.0)))
        .push(Spacer::new(&ui))
        .push(disabled.clone())
        .push(indicator.clone())
        .push(Label::new(&ui, "9:41").font(&NSFont::systemFontOfSize(12.0)));
    let bar = VisualEffect::new(&ui, NSVisualEffectMaterial::HeaderView, bar);
    bar.ns_visual_effect_view()
        .setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
    bar.ns_visual_effect_view().setState(NSVisualEffectState::Active);
    let disabled_copy = disabled.clone();
    f.sync_when(
        &indicator,
        |s| s.settings.ui.menu_bar.clone(),
        move |view, s| {
            let settings = &s.settings.ui.menu_bar;
            view.update(settings);
            width.setConstant(if settings.enabled { view.width() } else { 0.0 });
            disabled_copy.set_hidden(settings.enabled);
        },
    );
    bar.ns_view().setWantsLayer(true);
    if let Some(layer) = bar.ns_view().layer() {
        layer.setCornerRadius(10.0);
        layer.setMasksToBounds(true);
    }
    let preview = Section::form(&ui, "").content(bar).footer(Caption::new(
        &ui,
        "示例：开发工作区 · resize 快捷键模式",
    ));
    let visibility = f.fields(
        Section::new(&ui, "可见性"),
        &["enabled", "show_empty", "show_binding_mode"],
        |s| &s.settings.ui.menu_bar,
        |s| &mut s.settings.ui.menu_bar,
    );
    let appearance = f.fields(
        Section::new(&ui, "工作区"),
        &["mode", "active_label", "display_style"],
        |s| &s.settings.ui.menu_bar,
        |s| &mut s.settings.ui.menu_bar,
    );
    let message = Rc::new(ValidationMessage::new(&ui));
    let error = Rc::downgrade(&message);
    let weak_model = f.model.clone();
    let path = Rc::new(PathField::new(&ui).directories().on_change(move |path| {
        Model::submit(
            &weak_model,
            Box::new(move |s| {
                s.settings.ui.menu_bar.layout_folder = path;
                Ok(())
            }),
            error.clone(),
        );
    }));
    f.sync_when(
        &path,
        |s| s.settings.ui.menu_bar.layout_folder.clone(),
        |path, s| {
            path.set_value(&s.settings.ui.menu_bar.layout_folder);
        },
    );
    let row = f.row("布局文件夹", path, message);
    let row = f
        .schema_metadata(row, MenuBarSettings::field("layout_folder").unwrap(), |s| {
            &s.settings.ui.menu_bar
        })
        .wide();
    f.finish(
        SettingsPage::new(&ui, "").section(preview).section(visibility).section(
            appearance.row(row).footer(Caption::new(
                &ui,
                "布局文件会显示在菜单栏的已保存布局菜单中。",
            )),
        ),
    )
}

pub(super) fn overview(ui: Ui, model: &Rc<Model>) -> Page {
    let mut f = FormBuilder::new(ui, model);
    f.tooltips = true;
    let visibility = f.fields(
        Section::new(&ui, "工作区"),
        &["enabled", "show_empty_workspaces", "window_previews"],
        |s| &s.settings.ui.mission_control,
        |s| &mut s.settings.ui.mission_control,
    );
    let transitions = f.fields(
        Section::new(&ui, "过渡效果"),
        &["fade_enabled"],
        |s| &s.settings.ui.mission_control,
        |s| &mut s.settings.ui.mission_control,
    );
    let duration = f.field(
        "fade_duration_ms",
        |s| &s.settings.ui.mission_control,
        |s| &mut s.settings.ui.mission_control,
    );
    f.enabled(&duration, |s| {
        s.settings.ui.mission_control.enabled && s.settings.ui.mission_control.fade_enabled
    });
    f.finish(
        SettingsPage::new(&ui, "")
            .section(visibility)
            .section(transitions.row(duration))
            .section(Caption::new(
                &ui,
                "更改会立即保存，并在重新启动 Rift 后生效。",
            )),
    )
}

pub(super) fn stack_tabs(ui: Ui, model: &Rc<Model>) -> Page {
    let mut f = FormBuilder::new(ui, model);
    f.tooltips = true;
    let preview = Rc::new(StackPreview::new(&ui));
    preview.height(280.0);
    preview.accessibility_label("堆叠标签预览。点击标签可选择示例窗口。");
    f.sync_when(
        &preview,
        |s| s.settings.ui.stack_line.clone(),
        |view, s| view.update(&s.settings.ui.stack_line),
    );
    let caption = Rc::new(Caption::new(&ui, ""));
    f.sync_when(
        &caption,
        |s| s.settings.ui.stack_line.enabled,
        |caption, s| {
            caption.set_text(if s.settings.ui.stack_line.enabled {
                "macOS 应用程序示例。可选择或关闭标签。"
            } else {
                "标签已隐藏，示例窗口保持堆叠。"
            });
        },
    );
    let interaction = f.fields(
        Section::new(&ui, "行为"),
        &["enabled", "hover"],
        |s| &s.settings.ui.stack_line,
        |s| &mut s.settings.ui.stack_line,
    );
    let reset_preview = Rc::downgrade(&preview);
    let reset = Button::new(&ui, "重置预览").borderless().on_click(move || {
        if let Some(preview) = reset_preview.upgrade() {
            preview.reset();
        }
    });
    reset.ns_button().setControlSize(NSControlSize::Small);
    reset.ns_button().setFont(Some(&Font::caption()));
    let footer = HStack::new(&ui).spacing(12.0).push(caption).push(Spacer::new(&ui)).push(reset);
    let weak_preview = Rc::downgrade(&preview);
    f.live_preview = Some(Rc::new(move |source| {
        if let Some(preview) = weak_preview.upgrade() {
            preview.update(&source.settings.ui.stack_line);
        }
    }));
    let placement = f.fields(
        Section::new(&ui, "位置与尺寸"),
        &["horiz_placement", "vert_placement"],
        |s| &s.settings.ui.stack_line,
        |s| &mut s.settings.ui.stack_line,
    );
    let mut placement = placement;
    for (key, title, min, max, get, set) in [
        (
            "thickness",
            "厚度",
            20.0,
            64.0,
            (|s: &ConfigSource| s.settings.ui.stack_line.thickness) as fn(&ConfigSource) -> f64,
            (|s: &mut ConfigSource, v| s.settings.ui.stack_line.thickness = v)
                as fn(&mut ConfigSource, f64),
        ),
        (
            "spacing",
            "间距",
            0.0,
            100.0,
            (|s: &ConfigSource| s.settings.ui.stack_line.spacing) as fn(&ConfigSource) -> f64,
            (|s: &mut ConfigSource, v| s.settings.ui.stack_line.spacing = v)
                as fn(&mut ConfigSource, f64),
        ),
    ] {
        let (slider, input, message) =
            f.numeric_controls(title, 1.0, false, Some(Rc::new(move |_| (min, max))), get, set);
        let slider = slider.unwrap();
        if key == "thickness" {
            slider.set_step(4.0);
        }
        let row = f.row(title, NumberSlider::with_unit(&ui, slider, input, "pt"), message);
        placement = placement.row(f.schema_metadata(
            row,
            StackLineSettings::field(key).unwrap(),
            |s| &s.settings.ui.stack_line,
        ));
    }
    f.finish(
        SettingsPage::new(&ui, "")
            .section(Section::form(&ui, "").content(preview).footer(footer))
            .section(interaction)
            .section(placement),
    )
}
