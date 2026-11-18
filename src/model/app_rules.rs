use objc2_core_foundation::{CGPoint, CGRect};
use regex::{Regex, RegexBuilder};
use tracing::warn;

use crate::actor::app::WindowId;
use crate::common::collections::HashMap;
use crate::common::config::{AppRulePosition, AppRuleSize, AppWorkspaceRule, WorkspaceSelector};
use crate::model::VirtualWorkspaceId;
use crate::sys::screen::SpaceId;

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowRuleContext<'a> {
    pub app_bundle_id: Option<&'a str>,
    pub app_name: Option<&'a str>,
    pub window_title: Option<&'a str>,
    pub ax_role: Option<&'a str>,
    pub ax_subrole: Option<&'a str>,
    pub window_tags: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppRuleDecision {
    pub manage: Option<bool>,
    pub workspace: Option<WorkspaceSelector>,
    pub floating: bool,
    pub position: Option<AppRulePosition>,
    pub size: Option<AppRuleSize>,
    pub focus: bool,
}

impl AppRuleDecision {
    pub(crate) fn management_override(&self) -> Option<bool> { self.manage }
}

/// Complete result of applying a managed app rule to workspace policy.
///
/// The layout engine consumes this as one unit so rule effects do not get
/// independently re-derived at each integration point.
#[derive(Debug, Clone, Copy)]
pub struct AppRuleEffects {
    /// Source captured by the membership mutation, used only to update layout projections.
    pub source_workspace: Option<VirtualWorkspaceId>,
    pub workspace_id: VirtualWorkspaceId,
    pub floating: bool,
    pub position: Option<AppRulePosition>,
    pub size: Option<AppRuleSize>,
    pub focus: bool,
    pub was_rule_floating: bool,
}

impl AppRuleEffects {
    pub(crate) fn should_float(self, was_floating: bool) -> bool {
        self.floating || (!self.was_rule_floating && was_floating)
    }

    pub(crate) fn floating_placement(
        self,
        window: WindowId,
        space: SpaceId,
    ) -> Option<AppRulePlacement> {
        (self.floating && (self.position.is_some() || self.size.is_some())).then_some(
            AppRulePlacement {
                window,
                space,
                position: self.position,
                size: self.size,
            },
        )
    }

    pub(crate) fn tiled_resize(
        self,
        window: WindowId,
        space: SpaceId,
        was_floating: bool,
    ) -> Option<AppRuleResize> {
        (!self.should_float(was_floating)).then_some(AppRuleResize {
            window,
            space,
            workspace_id: self.workspace_id,
            size: self.size?,
        })
    }
}

/// Workspace-policy result for one evaluated window.
#[derive(Debug, Clone, Copy)]
pub enum AppRuleResult {
    Managed(AppRuleEffects),
    Rejected(AppRuleRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRuleRejection {
    ExplicitRule,
    Heuristic,
}

/// Follow-up integration work produced while applying a batch of app rules.
///
/// Like the reactor's `EventOutcome`, this is returned to the owning layer and
/// consumed explicitly rather than stored as transient engine state.
#[derive(Debug, Default)]
pub(crate) struct AppRuleOutcome {
    placements: Vec<AppRulePlacement>,
    resizes: Vec<AppRuleResize>,
    workspace_focus: Option<AppRuleWorkspaceFocus>,
}

impl AppRuleOutcome {
    pub(crate) fn push_placement(&mut self, placement: AppRulePlacement) {
        self.placements.push(placement);
    }

    pub(crate) fn push_resize(&mut self, resize: AppRuleResize) { self.resizes.push(resize); }

    pub(crate) fn has_resizes(&self) -> bool { !self.resizes.is_empty() }

    pub(crate) fn set_workspace_focus(&mut self, focus: AppRuleWorkspaceFocus) {
        self.workspace_focus = Some(focus);
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<AppRulePlacement>,
        Vec<AppRuleResize>,
        Option<AppRuleWorkspaceFocus>,
    ) {
        (self.placements, self.resizes, self.workspace_focus)
    }
}

/// One-shot frame request derived from a floating app rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AppRulePlacement {
    pub(crate) window: WindowId,
    pub(crate) space: SpaceId,
    pub(crate) position: Option<AppRulePosition>,
    pub(crate) size: Option<AppRuleSize>,
}

impl AppRulePlacement {
    pub(crate) fn resolve_frame(self, current: CGRect, screen: CGRect) -> CGRect {
        let mut frame = current;
        if let Some(size) = self.size {
            if let Some(width) = size.w {
                frame.size.width = width;
            }
            if let Some(height) = size.h {
                frame.size.height = height;
            }
        }
        if let Some(position) = self.position {
            let travel_x = (screen.size.width - frame.size.width).max(0.0);
            let travel_y = (screen.size.height - frame.size.height).max(0.0);
            frame.origin = CGPoint::new(
                screen.origin.x + travel_x * position.x,
                screen.origin.y + travel_y * position.y,
            );
        }
        frame
    }
}

/// One-time tiled resize applied after the window has entered its layout tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AppRuleResize {
    pub(crate) window: WindowId,
    pub(crate) space: SpaceId,
    pub(crate) workspace_id: VirtualWorkspaceId,
    pub(crate) size: AppRuleSize,
}

/// Reactor-owned part of a focus rule: switching workspaces requires saving
/// the currently visible floating frames before the engine activates the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AppRuleWorkspaceFocus {
    pub(crate) window: WindowId,
    pub(crate) space: SpaceId,
    pub(crate) workspace: VirtualWorkspaceId,
}

#[derive(Debug, Clone)]
struct Matchers {
    app_id: Option<String>,
    app_name: Option<String>,
    title_regex: Option<Regex>,
    title_substring: Option<String>,
    ax_role: Option<String>,
    ax_subrole: Option<String>,
    window_tags: Option<u64>,
}

impl Matchers {
    /// Compiles the matcher fields, returning `None` when the rule cannot match
    /// anything (invalid regex, zero window tags, or no matcher at all).
    fn compile(
        app_id: Option<String>,
        app_name: Option<String>,
        title_regex: Option<String>,
        title_substring: Option<String>,
        ax_role: Option<String>,
        ax_subrole: Option<String>,
        window_tags: Option<u64>,
    ) -> Option<Self> {
        if window_tags == Some(0) {
            return None;
        }
        let app_id = nonempty(app_id).map(|value| value.to_ascii_lowercase());
        let app_name = nonempty(app_name).map(|value| value.to_lowercase());
        let title_substring = nonempty(title_substring).map(|value| value.to_lowercase());
        let ax_role = nonempty(ax_role);
        let ax_subrole = nonempty(ax_subrole);
        let title_regex = match nonempty(title_regex) {
            Some(pattern) => Some(RegexBuilder::new(&pattern).case_insensitive(true).build().ok()?),
            None => None,
        };
        let matchers = Self {
            app_id,
            app_name,
            title_regex,
            title_substring,
            ax_role,
            ax_subrole,
            window_tags,
        };
        matchers.has_matcher().then_some(matchers)
    }

    fn has_matcher(&self) -> bool {
        self.app_id.is_some()
            || self.app_name.is_some()
            || self.title_regex.is_some()
            || self.title_substring.is_some()
            || self.ax_role.is_some()
            || self.ax_subrole.is_some()
            || self.window_tags.is_some()
    }

    fn matches_title(&self) -> bool { self.title_regex.is_some() || self.title_substring.is_some() }

    fn matches_prepared(
        &self,
        context: WindowRuleContext<'_>,
        app_id: Option<&str>,
        app_name: Option<&str>,
        title: Option<&str>,
    ) -> bool {
        self.app_id.as_deref().is_none_or(|rule| app_id == Some(rule))
            && self.app_name.as_deref().is_none_or(|rule| {
                app_name.is_some_and(|actual| rule.contains(actual) || actual.contains(rule))
            })
            && self.title_regex.as_ref().is_none_or(|regex| {
                context.window_title.is_some_and(|actual| regex.is_match(actual))
            })
            && self
                .title_substring
                .as_deref()
                .is_none_or(|rule| title.is_some_and(|actual| actual.contains(rule)))
            && self.ax_role.as_deref().is_none_or(|rule| context.ax_role == Some(rule))
            && self.ax_subrole.as_deref().is_none_or(|rule| context.ax_subrole == Some(rule))
            && self
                .window_tags
                .is_none_or(|mask| context.window_tags.is_some_and(|tags| tags & mask == mask))
    }
}

#[derive(Debug, Clone)]
struct CompiledRule {
    action: AppRuleDecision,
    matchers: Matchers,
    index: usize,
}

/// Compiles and evaluates app policy without depending on workspaces, windows,
/// the reactor, or the layout engine.
#[derive(Debug, Clone, Default)]
pub struct AppRuleEngine {
    rules_by_app_id: HashMap<String, Vec<CompiledRule>>,
    wildcard_rules: Vec<CompiledRule>,
}

impl AppRuleEngine {
    pub fn new(rules: &[AppWorkspaceRule]) -> Self {
        let mut engine = Self::default();
        for (index, rule) in rules.iter().cloned().enumerate() {
            let Some(rule) = CompiledRule::new(index, rule) else {
                warn!(index, "Ignoring invalid app rule");
                continue;
            };
            if let Some(app_id) = rule.matchers.app_id.clone() {
                engine.rules_by_app_id.entry(app_id).or_default().push(rule);
            } else {
                engine.wildcard_rules.push(rule);
            }
        }
        engine
    }

    pub fn uses_window_tags(&self) -> bool {
        self.wildcard_rules
            .iter()
            .chain(self.rules_by_app_id.values().flatten())
            .any(|rule| rule.matchers.window_tags.is_some())
    }

    pub fn evaluate(&self, context: WindowRuleContext<'_>) -> Option<AppRuleDecision> {
        self.evaluate_with_index(context).map(|(_, decision)| decision)
    }

    /// Evaluate the rule set and report which configuration entry won, so the
    /// settings UI can explain the outcome.
    pub fn evaluate_with_index(
        &self,
        context: WindowRuleContext<'_>,
    ) -> Option<(usize, AppRuleDecision)> {
        self.matching_rule(context).map(|rule| (rule.index, rule.action.clone()))
    }

    /// Re-evaluate rules after a title change without letting a title-independent
    /// fallback reassert its initial workspace placement.
    pub fn evaluate_for_title_change(
        &self,
        context: WindowRuleContext<'_>,
    ) -> Option<AppRuleDecision> {
        self.matching_rule(context).map(|rule| {
            let mut decision = rule.action.clone();
            if !rule.matchers.matches_title() {
                decision.workspace = None;
                decision.focus = false;
            }
            decision
        })
    }

    fn matching_rule(&self, context: WindowRuleContext<'_>) -> Option<&CompiledRule> {
        let app_id = context.app_bundle_id.map(str::to_ascii_lowercase);
        let app_name = context.app_name.map(str::to_lowercase);
        let title = context.window_title.map(str::to_lowercase);
        self.wildcard_rules
            .iter()
            .chain(
                app_id
                    .as_ref()
                    .and_then(|id| self.rules_by_app_id.get(id))
                    .into_iter()
                    .flatten(),
            )
            .filter(|rule| {
                rule.matchers.matches_prepared(
                    context,
                    app_id.as_deref(),
                    app_name.as_deref(),
                    title.as_deref(),
                )
            })
            // The first enabled rule in configuration order wins.
            .min_by_key(|rule| rule.index)
    }
}

/// Evaluate a draft rule set in configuration order and return the index of the
/// first enabled rule that matches. Used by the settings preview, where the
/// rules may not have been saved yet.
pub fn preview_rule_index(
    rules: &[rift_protocol::AppRuleMatcher],
    context: WindowRuleContext<'_>,
) -> Option<usize> {
    let app_id = context.app_bundle_id.map(str::to_ascii_lowercase);
    let app_name = context.app_name.map(str::to_lowercase);
    let title = context.window_title.map(str::to_lowercase);
    rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.enabled)
        .find_map(|(index, rule)| {
            let matchers = Matchers::compile(
                rule.app_id.clone(),
                rule.app_name.clone(),
                rule.title_regex.clone(),
                rule.title_substring.clone(),
                rule.ax_role.clone(),
                rule.ax_subrole.clone(),
                rule.window_tags,
            )?;
            matchers
                .matches_prepared(context, app_id.as_deref(), app_name.as_deref(), title.as_deref())
                .then_some(index)
        })
}

impl CompiledRule {
    fn new(index: usize, rule: AppWorkspaceRule) -> Option<Self> {
        let AppWorkspaceRule {
            enabled,
            app_id,
            workspace,
            floating,
            position,
            size,
            focus,
            manage,
            app_name,
            title_regex,
            title_substring,
            ax_role,
            ax_subrole,
            window_tags,
        } = rule;
        if !enabled {
            return None;
        }
        let matchers = Matchers::compile(
            app_id,
            app_name,
            title_regex,
            title_substring,
            ax_role,
            ax_subrole,
            window_tags,
        )?;
        Some(Self {
            action: AppRuleDecision {
                manage,
                workspace,
                floating,
                position,
                size,
                focus,
            },
            matchers,
            index,
        })
    }
}

fn nonempty(value: Option<String>) -> Option<String> { value.filter(|value| !value.is_empty()) }

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(app_id: &str, workspace: usize) -> AppWorkspaceRule {
        AppWorkspaceRule {
            app_id: Some(app_id.into()),
            workspace: Some(WorkspaceSelector::Index(workspace)),
            ..Default::default()
        }
    }

    #[test]
    fn tag_mask_requires_all_bits_and_known_tags() {
        let engine = AppRuleEngine::new(&[AppWorkspaceRule {
            window_tags: Some(130),
            floating: true,
            ..Default::default()
        }]);
        for (tags, matches) in [
            (Some(130), true),
            (Some(131), true),
            (Some(2), false),
            (Some(128), false),
            (Some(0), false),
            (None, false),
        ] {
            assert_eq!(
                engine
                    .evaluate(WindowRuleContext {
                        window_tags: tags,
                        ..Default::default()
                    })
                    .is_some(),
                matches
            );
        }
        let encoded = toml::to_string(&AppWorkspaceRule {
            window_tags: Some(130),
            ..Default::default()
        })
        .unwrap();
        assert!(encoded.contains("window_tags = 130"));
        assert_eq!(
            toml::from_str::<AppWorkspaceRule>(&encoded).unwrap().window_tags,
            Some(130)
        );
    }

    #[test]
    fn evaluates_without_workspace_or_layout_state() {
        let rule = AppWorkspaceRule {
            app_id: Some("com.example.Editor".into()),
            workspace: None,
            floating: true,
            position: Some(AppRulePosition { x: 0.4, y: 0.7 }),
            size: Some(AppRuleSize { w: Some(640.0), h: Some(480.0) }),
            focus: true,
            manage: Some(true),
            title_regex: Some("project \\d+".into()),
            ..Default::default()
        };
        let engine = AppRuleEngine::new(&[rule]);
        assert_eq!(
            engine.evaluate(WindowRuleContext {
                app_bundle_id: Some("COM.EXAMPLE.EDITOR"),
                window_title: Some("Project 42"),
                ..Default::default()
            }),
            Some(AppRuleDecision {
                manage: Some(true),
                workspace: None,
                floating: true,
                position: Some(AppRulePosition { x: 0.4, y: 0.7 }),
                size: Some(AppRuleSize { w: Some(640.0), h: Some(480.0) }),
                focus: true,
            })
        );
    }

    #[test]
    fn first_enabled_rule_in_configuration_order_wins() {
        let first = rule("com.example.Editor", 0);
        let mut more_specific = rule("com.example.Editor", 2);
        more_specific.title_substring = Some("project".into());
        more_specific.ax_role = Some("AXWindow".into());
        let engine = AppRuleEngine::new(&[first, more_specific]);

        let decision = engine
            .evaluate(WindowRuleContext {
                app_bundle_id: Some("com.example.editor"),
                window_title: Some("Project notes"),
                ax_role: Some("AXWindow"),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            decision.workspace,
            Some(WorkspaceSelector::Index(0)),
            "an earlier generic rule wins over a later, more specific one"
        );
    }

    #[test]
    fn disabled_rules_are_skipped_and_the_winner_index_is_reported() {
        let mut first = rule("com.example.Editor", 0);
        first.enabled = false;
        let second = rule("com.example.Editor", 1);
        let engine = AppRuleEngine::new(&[first, second]);

        let (index, decision) = engine
            .evaluate_with_index(WindowRuleContext {
                app_bundle_id: Some("com.example.editor"),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(index, 1);
        assert_eq!(decision.workspace, Some(WorkspaceSelector::Index(1)));

        let mut only_disabled = rule("com.example.Editor", 3);
        only_disabled.enabled = false;
        assert!(
            AppRuleEngine::new(&[only_disabled])
                .evaluate(WindowRuleContext {
                    app_bundle_id: Some("com.example.editor"),
                    ..Default::default()
                })
                .is_none()
        );
    }

    #[test]
    fn invalid_or_empty_matchers_never_become_wildcards() {
        let mut invalid_regex = rule("com.example.Editor", 1);
        invalid_regex.title_regex = Some("[".into());
        let mut empty = rule("", 2);
        empty.app_id = Some(String::new());
        let engine = AppRuleEngine::new(&[invalid_regex, empty]);

        assert!(
            engine
                .evaluate(WindowRuleContext {
                    app_bundle_id: Some("com.example.Editor"),
                    window_title: Some("anything"),
                    ..Default::default()
                })
                .is_none()
        );
    }

    #[test]
    fn title_change_strips_placement_only_when_the_winner_has_no_title_matcher() {
        let generic = rule("com.example.Editor", 1);
        let mut titled = rule("com.example.Editor", 2);
        titled.title_substring = Some("special".into());
        titled.size = Some(AppRuleSize { w: Some(80.0), h: None });

        // A generic winner first: placement is stripped after a title change.
        let generic_first = AppRuleEngine::new(&[generic.clone(), titled.clone()]);
        let fallback = generic_first
            .evaluate_for_title_change(WindowRuleContext {
                app_bundle_id: Some("com.example.Editor"),
                window_title: Some("special window"),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(fallback.workspace, None);

        // Title rules must be listed before the generic fallback to win.
        let titled_first = AppRuleEngine::new(&[titled, generic]);
        let title_match = titled_first
            .evaluate_for_title_change(WindowRuleContext {
                app_bundle_id: Some("com.example.Editor"),
                window_title: Some("special window"),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(title_match.workspace, Some(WorkspaceSelector::Index(2)));
        assert_eq!(title_match.size, Some(AppRuleSize { w: Some(80.0), h: None }));

        let ordinary = titled_first
            .evaluate_for_title_change(WindowRuleContext {
                app_bundle_id: Some("com.example.Editor"),
                window_title: Some("ordinary window"),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(ordinary.workspace, None);
    }
}
