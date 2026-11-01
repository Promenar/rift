// 简体中文界面修改：本文件的显示文案或相关界面测试已本地化。
//! Compile-time Settings metadata. Doc comments remain the source of help text;
//! config serialization and validation stay independent of UI generation.
pub use rift_config_derive::{ConfigEnum, ConfigSchema};

/// Native popup label and its documentation; variant identity is the typed index.
pub type EnumChoice = (&'static str, &'static str);
#[derive(Clone, Copy)]
pub enum FieldKind {
    Bool,
    Number {
        integer: bool,
    },
    Text,
    Choice(&'static [EnumChoice]),
    /// Specialized UI, such as inherited values, rules, or visual editors.
    Custom,
}
#[derive(Clone, PartialEq)]
pub enum FieldValue {
    Bool(bool),
    Number(f64),
    Text(String),
    Choice(usize),
}
pub struct ConfigField<T> {
    pub key: &'static str,
    pub title: &'static str,
    pub help: &'static str,
    pub group: &'static str,
    pub aliases: &'static str,
    pub unit: &'static str,
    pub scale: f64,
    pub kind: FieldKind,
    pub enabled: Option<fn(&T) -> bool>,
    pub read: Option<fn(&T) -> FieldValue>,
    pub write: Option<fn(&mut T, FieldValue) -> Result<(), String>>,
}
pub trait ConfigSchema: Sized + 'static {
    fn fields() -> &'static [ConfigField<Self>];
    fn field(key: &str) -> Option<&'static ConfigField<Self>> {
        Self::fields().iter().find(|field| field.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(ConfigSchema)]
    #[setting(group = "default")]
    struct Example {
        /// First paragraph.
        ///
        /// Second paragraph.
        #[setting(label = "Motion", group = "behavior")]
        animate: bool,
        #[setting(enabled_by = "animate", unit = "pt", scale = 100.0, order = 0)]
        window_count: usize,
        #[setting(ignore)]
        internal: Vec<String>,
        /// Uses a dedicated editor.
        #[setting(custom)]
        rules: Vec<String>,
    }

    #[test]
    fn schema_exposes_docs_and_typed_edits_without_exposing_ignored_fields() {
        let mut value = Example {
            animate: false,
            window_count: 1,
            internal: vec!["private".into()],
            rules: vec![],
        };
        assert_eq!(Example::fields()[0].key, "window_count");
        let motion = Example::field("animate").unwrap();
        assert_eq!(
            (motion.title, motion.group, motion.help),
            ("Motion", "behavior", "First paragraph.\n\nSecond paragraph.")
        );
        assert!(Example::field("internal").is_none());
        let custom = Example::field("rules").unwrap();
        assert!(custom.read.is_none() && custom.write.is_none());
        let count = Example::field("window_count").unwrap();
        assert_eq!(count.title, "Window count");
        assert_eq!(count.group, "default");
        assert_eq!((count.unit, count.scale), ("pt", 100.0));
        assert!(!count.enabled.unwrap()(&value));
        motion.write.unwrap()(&mut value, FieldValue::Bool(true)).unwrap();
        assert!(count.enabled.unwrap()(&value));
        count.write.unwrap()(&mut value, FieldValue::Number(3.0)).unwrap();
        assert_eq!(value.window_count, 3);
        for bad in [-1.0, 1.5, f64::NAN, f64::INFINITY] {
            assert!(count.write.unwrap()(&mut value, FieldValue::Number(bad)).is_err());
            assert_eq!(
                value.window_count, 3,
                "invalid writes leave the config untouched"
            );
        }
        assert!(motion.write.unwrap()(&mut value, FieldValue::Text("true".into())).is_err());
        assert_eq!(value.internal, ["private"]);
        assert!(value.rules.is_empty());
    }

    #[test]
    fn generated_enum_edits_preserve_existing_config_serialization() {
        use crate::common::config::{ActiveWorkspaceLabel, MenuBarSettings};
        let mut config = MenuBarSettings::default();
        let field = MenuBarSettings::field("active_label").unwrap();
        let FieldKind::Choice(choices) = field.kind else {
            panic!("expected native popup choices")
        };
        let index = choices.iter().position(|choice| choice.0 == "名称").unwrap();
        field.write.unwrap()(&mut config, FieldValue::Choice(index)).unwrap();
        assert_eq!(config.active_label, ActiveWorkspaceLabel::Name);
        assert_eq!(serde_json::to_value(&config).unwrap()["active_label"], "name");
        assert!(field.write.unwrap()(&mut config, FieldValue::Choice(choices.len())).is_err());
        assert_eq!(config.active_label, ActiveWorkspaceLabel::Name);
    }

    #[test]
    fn settings_metadata_uses_chinese_titles_and_help() {
        fn has_chinese(text: &str) -> bool {
            text.chars().any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
        }

        fn check<T: ConfigSchema>() {
            for field in T::fields() {
                assert!(has_chinese(field.title), "设置标题应包含中文：{}", field.key);
                assert!(
                    field.help.is_empty() || has_chinese(field.help),
                    "设置帮助应包含中文：{}",
                    field.key
                );
                assert!(field.key.is_ascii(), "配置键保持英文：{}", field.key);
                assert!(field.group.is_ascii(), "分组标识保持英文：{}", field.key);
                if let FieldKind::Choice(choices) = field.kind {
                    for (title, help) in choices {
                        assert!(has_chinese(title), "选项标题应包含中文：{title}");
                        assert!(
                            help.is_empty() || has_chinese(help),
                            "选项帮助应包含中文：{title}"
                        );
                    }
                }
            }
        }

        macro_rules! check_schemas {
            ($($name:ident),+ $(,)?) => {
                $(check::<crate::common::config::$name>();)+
            };
        }
        check_schemas!(
            VirtualWorkspaceSettings,
            AppWorkspaceRule,
            Settings,
            GestureSettings,
            DragDropSettings,
            MenuBarSettings,
            StackLineSettings,
            MissionControlSettings,
            BaseLayoutSettings,
            TraditionalLayoutSettings,
            BspLayoutSettings,
            LayoutSettings,
            ScrollingLayoutSettings,
            MasterStackSettings,
            ScrollingGestureSettings,
            StackSettings,
            OuterGaps,
            InnerGaps,
        );
    }

    #[test]
    fn localized_choices_keep_protocol_values_and_variant_order() {
        use crate::common::config::*;

        macro_rules! check_enum {
            ($name:ident, [$($serialized:literal),+ $(,)?]) => {{
                let expected = [$($serialized),+];
                assert_eq!($name::CONFIG_CHOICES.len(), expected.len());
                for (index, expected) in expected.iter().enumerate() {
                    let value = $name::from_config_choice(index).unwrap();
                    assert_eq!(value.config_choice_index(), index);
                    assert_eq!(serde_json::to_value(value).unwrap(), *expected);
                    let decoded: $name =
                        serde_json::from_value(serde_json::json!(expected)).unwrap();
                    assert_eq!(decoded, value);
                    assert!(
                        $name::CONFIG_CHOICES[index].0.chars().any(|character| {
                            ('\u{4e00}'..='\u{9fff}').contains(&character)
                        }),
                        "选项标题应包含中文：{}",
                        stringify!($name)
                    );
                }
                assert!($name::from_config_choice(expected.len()).is_none());
            }};
        }
        check_enum!(LayoutMode, [
            "traditional", "bsp", "stack", "master_stack", "scrolling", "floating",
        ]);
        check_enum!(MouseModifier, ["cmd", "alt", "shift", "ctrl", "fn"]);
        check_enum!(MouseAction, ["none", "move"]);
        check_enum!(MouseDropAction, ["swap", "stack"]);
        check_enum!(MenuBarDisplayMode, ["all", "active"]);
        check_enum!(ActiveWorkspaceLabel, ["index", "name"]);
        check_enum!(WorkspaceDisplayStyle, ["layout", "label"]);
        check_enum!(StackLineHoverMode, ["click", "hover"]);
        check_enum!(HorizontalPlacement, ["top", "bottom"]);
        check_enum!(VerticalPlacement, ["left", "right"]);
        check_enum!(MasterStackSide, ["left", "right", "top", "bottom"]);
        check_enum!(ScrollingAlignment, ["left", "center", "right"]);
        check_enum!(ScrollingFocusNavigationStyle, ["niri", "anchored"]);
        check_enum!(MasterStackNewWindowPlacement, ["master", "stack", "focused"]);
        check_enum!(StackDefaultOrientation, [
            "perpendicular", "same", "horizontal", "vertical",
        ]);
        check_enum!(HapticPattern, ["generic", "alignment", "level_change"]);
        check_enum!(AnimationEasing, [
            "ease_in_out", "linear", "ease_in_sine", "ease_out_sine", "ease_in_out_sine",
            "ease_in_quad", "ease_out_quad", "ease_in_out_quad", "ease_in_cubic", "ease_out_cubic",
            "ease_in_out_cubic", "ease_in_quart", "ease_out_quart", "ease_in_out_quart",
            "ease_in_quint", "ease_out_quint", "ease_in_out_quint", "ease_in_expo", "ease_out_expo",
            "ease_in_out_expo", "ease_in_circ", "ease_out_circ", "ease_in_out_circ",
        ]);
    }
}
