use std::sync::LazyLock;

use cgs::compositor::{
    CgsWindow, CgsWindowError, WindowSurface, WindowTransaction, backdrop_blur,
    with_disabled_actions,
};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_core_foundation::{CFString, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGColor;
use objc2_quartz_core::{CALayer, CATextLayer, CATransaction, kCAAlignmentLeft};

use crate::actor::drag::{DropTarget, preview_frame};
use crate::layout_engine::{WindowDropAction, ZoneColumn};
use crate::sys::geometry::SameAs;

static FILL: LazyLock<Retained<CGColor>> =
    LazyLock::new(|| CGColor::new_generic_rgb(0.13, 0.62, 1.0, 0.18).into());
static BORDER: LazyLock<Retained<CGColor>> =
    LazyLock::new(|| CGColor::new_generic_rgb(0.13, 0.62, 1.0, 0.95).into());
static STACK_FILL: LazyLock<Retained<CGColor>> =
    LazyLock::new(|| CGColor::new_generic_rgb(0.34, 0.64, 1.0, 0.16).into());
static ZONE_CELL_BORDER: LazyLock<Retained<CGColor>> =
    LazyLock::new(|| CGColor::new_generic_rgb(0.13, 0.62, 1.0, 0.32).into());
static ZONE_LABEL_BG: LazyLock<Retained<CGColor>> =
    LazyLock::new(|| CGColor::new_generic_rgb(0.04, 0.06, 0.09, 0.86).into());

const PREVIEW_CORNER_RADIUS: f64 = 12.0;
const PREVIEW_BLUR_RADIUS: f64 = 6.0;
const FADE_DURATION: f64 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewStyle {
    Tile,
    Stack,
}

/// A single reusable WindowServer overlay backed by a persistent compositor
/// surface. Layers are allocated once and only changed properties are flushed.
pub struct DragPreview {
    // The surface must be detached before its owning WindowServer window drops.
    surface: WindowSurface,
    window: CgsWindow,
    root: Retained<CALayer>,
    backdrop: Option<Retained<CALayer>>,
    tint: Retained<CALayer>,
    card: Retained<CALayer>,
    frame: CGRect,
    style: PreviewStyle,
    relative_window: Option<u32>,
    visible: bool,
}

// The preview is exclusively owned by Reactor. Reactor itself is moved once to
// its dedicated thread before a preview can be created; the CA objects are
// never accessed concurrently or moved again after creation.
unsafe impl Send for DragPreview {}

impl DragPreview {
    pub fn new(target: DropTarget) -> Result<Self, CgsWindowError> {
        let frame = preview_frame(target);
        let style = Self::style(target);
        let bounds = Self::bounds(frame.size);
        let window = CgsWindow::new_compositor(frame, PREVIEW_CORNER_RADIUS)?;
        window.set_alpha(1.0)?;
        window.set_tags(1 << 3)?;

        let root = CALayer::layer();
        let backdrop = backdrop_blur(PREVIEW_BLUR_RADIUS);
        let tint = CALayer::layer();
        let card = CALayer::layer();
        with_disabled_actions(|| {
            root.setBounds(bounds);
            root.setPosition(CGPoint::new(0.0, 0.0));
            root.setAnchorPoint(CGPoint::new(0.0, 0.0));
            root.setContentsScale(2.0);
            root.setOpaque(false);
            root.setGeometryFlipped(false);
            root.setMasksToBounds(true);
            root.setOpacity(0.0);
            root.setCornerRadius(PREVIEW_CORNER_RADIUS);
            root.setBorderColor(Some(&BORDER));
            root.setBorderWidth(2.0);

            if let Some(backdrop) = &backdrop {
                backdrop.setFrame(bounds);
                root.addSublayer(backdrop);
            }

            tint.setFrame(bounds);
            tint.setBackgroundColor(Some(&FILL));
            root.addSublayer(&tint);

            card.setOpacity(if style == PreviewStyle::Stack {
                1.0
            } else {
                0.0
            });
            card.setFrame(Self::card_frame(frame.size));
            card.setCornerRadius(8.0);
            card.setBackgroundColor(Some(&STACK_FILL));
            card.setBorderColor(Some(&BORDER));
            card.setBorderWidth(1.0);
            root.addSublayer(&card);
        });

        let surface = WindowSurface::new(window.id(), bounds, &root)?;
        surface.flush();

        Ok(Self {
            surface,
            window,
            root,
            backdrop,
            tint,
            card,
            frame,
            style,
            relative_window: None,
            visible: false,
        })
    }

    pub fn show(&mut self, target: DropTarget) -> Result<(), CgsWindowError> {
        let frame = preview_frame(target);
        let style = Self::style(target);
        let relative_window = target.intent.window.idx.get();
        let size_changed = !self.frame.size.same_as(frame.size);
        let origin_changed = !self.frame.origin.same_as(frame.origin);
        let frame_changed = size_changed || origin_changed;
        let style_changed = self.style != style;
        let ordering_changed = !self.visible || self.relative_window != Some(relative_window);

        if !frame_changed && !style_changed && !ordering_changed {
            return Ok(());
        }

        if size_changed {
            with_disabled_actions(|| {
                self.root.setBounds(Self::bounds(frame.size));
                if let Some(backdrop) = &self.backdrop {
                    backdrop.setFrame(Self::bounds(frame.size));
                }
                self.tint.setFrame(Self::bounds(frame.size));
                self.card.setFrame(Self::card_frame(frame.size));
            });
        }

        if frame_changed {
            let transaction = WindowTransaction::new()?;
            if size_changed {
                transaction.set_rounded_frame(&self.window, frame, PREVIEW_CORNER_RADIUS)?;
                self.surface.set_bounds_in(&transaction, Self::bounds(frame.size));
            } else {
                transaction.move_window(&self.window, frame.origin);
            }
            transaction.commit();
            self.frame = frame;
        }

        if ordering_changed {
            if !self.visible {
                self.window.set_alpha(1.0)?;
            }
            self.window.order_above(Some(relative_window))?;
            self.relative_window = Some(relative_window);
        }
        if style_changed || !self.visible {
            CATransaction::begin();
            CATransaction::setAnimationDuration(FADE_DURATION);
            if style_changed {
                self.card.setOpacity(if style == PreviewStyle::Stack {
                    1.0
                } else {
                    0.0
                });
            }
            if !self.visible {
                self.root.setOpacity(1.0);
            }
            CATransaction::commit();
            self.style = style;
        }
        if size_changed || style_changed || !self.visible {
            self.surface.flush();
        }
        self.visible = true;
        Ok(())
    }

    pub fn hide(&mut self) {
        if self.visible {
            with_disabled_actions(|| self.root.setOpacity(0.0));
            self.surface.flush();
            let _ = self.window.set_alpha(0.0);
            let _ = self.window.order_out();
            self.visible = false;
        }
    }

    #[inline]
    fn style(target: DropTarget) -> PreviewStyle {
        if target.intent.action == WindowDropAction::Stack {
            PreviewStyle::Stack
        } else {
            PreviewStyle::Tile
        }
    }

    #[inline]
    fn bounds(size: CGSize) -> CGRect { CGRect::new(CGPoint::new(0.0, 0.0), size) }

    #[inline]
    fn card_frame(size: CGSize) -> CGRect {
        const INSET: f64 = 9.0;
        CGRect::new(
            CGPoint::new(INSET, INSET),
            CGSize::new(
                (size.width - INSET * 2.0).max(0.0),
                (size.height - INSET * 2.0).max(0.0),
            ),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoneCellKind {
    Full,
    Top,
    Bottom,
}

struct ZoneGeometry {
    screen: CGRect,
    cells: Vec<(CGRect, ZoneCellKind)>,
    candidate: CGRect,
    label: String,
}

fn zone_geometry(target: &DropTarget) -> Option<ZoneGeometry> {
    let WindowDropAction::Zone(drop) = target.intent.action else {
        return None;
    };
    let active: Vec<ZoneColumn> = drop.columns.iter().flatten().copied().collect();
    if active.is_empty() {
        return None;
    }
    let cell_rect = target.preview_area;
    let mut index = 0;
    let mut candidate = None;
    for (column, kind) in active.iter().enumerate() {
        match kind {
            ZoneColumn::Full => {
                if index == drop.cell {
                    candidate = Some((column, 0, ZoneCellKind::Full));
                }
                index += 1;
            }
            ZoneColumn::Split => {
                if index == drop.cell {
                    candidate = Some((column, 0, ZoneCellKind::Top));
                }
                if index + 1 == drop.cell {
                    candidate = Some((column, 1, ZoneCellKind::Bottom));
                }
                index += 2;
            }
        }
    }
    let (column, row, kind) = candidate?;
    let columns = active.len() as f64;
    let screen_height = if kind == ZoneCellKind::Full {
        cell_rect.size.height
    } else {
        cell_rect.size.height * 2.0
    };
    let screen = CGRect::new(
        CGPoint::new(
            cell_rect.origin.x - column as f64 * cell_rect.size.width,
            cell_rect.origin.y - row as f64 * cell_rect.size.height,
        ),
        CGSize::new(cell_rect.size.width * columns, screen_height),
    );
    let width = screen.size.width / columns;
    let mut cells = Vec::new();
    for (column, kind) in active.iter().enumerate() {
        let x = screen.origin.x + column as f64 * width;
        match kind {
            ZoneColumn::Full => cells.push((
                CGRect::new(
                    CGPoint::new(x, screen.origin.y),
                    CGSize::new(width, screen_height),
                ),
                ZoneCellKind::Full,
            )),
            ZoneColumn::Split => {
                let half = screen_height / 2.0;
                cells.push((
                    CGRect::new(CGPoint::new(x, screen.origin.y), CGSize::new(width, half)),
                    ZoneCellKind::Top,
                ));
                cells.push((
                    CGRect::new(CGPoint::new(x, screen.origin.y + half), CGSize::new(width, half)),
                    ZoneCellKind::Bottom,
                ));
            }
        }
    }
    let label = match kind {
        ZoneCellKind::Full => format!("全高 · 1/{} 屏", active.len()),
        ZoneCellKind::Top => format!("上半 · 1/{} 屏", active.len() * 2),
        ZoneCellKind::Bottom => format!("下半 · 1/{} 屏", active.len() * 2),
    };
    Some(ZoneGeometry {
        screen,
        cells,
        candidate: cell_rect,
        label,
    })
}

/// Full-display overlay for simple-mode screen zones: a grid of the target
/// layout plus the highlighted candidate cell. This is deliberately separate
/// from `DragPreview` so window-target previews cannot leak into simple mode.
pub struct ZonePreview {
    surface: WindowSurface,
    window: CgsWindow,
    root: Retained<CALayer>,
    cells: Vec<Retained<CALayer>>,
    highlight: Retained<CALayer>,
    label: Retained<CATextLayer>,
    frame: CGRect,
    relative_window: Option<u32>,
    visible: bool,
}

// Same ownership contract as DragPreview: Reactor moves once before any
// preview exists; layers are never touched concurrently.
unsafe impl Send for ZonePreview {}

impl ZonePreview {
    pub fn new(target: DropTarget) -> Result<Option<Self>, CgsWindowError> {
        let Some(geometry) = zone_geometry(&target) else {
            return Ok(None);
        };
        let frame = geometry.screen;
        let bounds = CGRect::new(CGPoint::new(0.0, 0.0), frame.size);
        let window = CgsWindow::new_compositor(frame, 0.0)?;
        window.set_alpha(1.0)?;
        window.set_tags(1 << 3)?;
        let root = CALayer::layer();
        let mut cells = Vec::new();
        let highlight = CALayer::layer();
        let label = CATextLayer::layer();
        with_disabled_actions(|| {
            root.setBounds(bounds);
            root.setPosition(CGPoint::new(0.0, 0.0));
            root.setAnchorPoint(CGPoint::new(0.0, 0.0));
            root.setContentsScale(2.0);
            root.setOpaque(false);
            root.setMasksToBounds(false);
            root.setOpacity(0.0);

            for _ in 0..6 {
                let cell = CALayer::layer();
                cell.setBorderColor(Some(&ZONE_CELL_BORDER));
                cell.setBorderWidth(1.0);
                cell.setCornerRadius(7.0);
                cell.setBackgroundColor(None);
                cell.setOpacity(0.0);
                root.addSublayer(&cell);
                cells.push(cell);
            }

            highlight.setBackgroundColor(Some(&FILL));
            highlight.setBorderColor(Some(&BORDER));
            highlight.setBorderWidth(2.0);
            highlight.setCornerRadius(8.0);
            root.addSublayer(&highlight);

            label.setFontSize(13.0);
            label.setForegroundColor(Some(&CGColor::new_generic_rgb(0.9, 0.95, 1.0, 1.0)));
            label.setBackgroundColor(Some(&ZONE_LABEL_BG));
            label.setCornerRadius(6.0);
            label.setMasksToBounds(true);
            unsafe { label.setAlignmentMode(kCAAlignmentLeft) };
            root.addSublayer(&label);
        });
        let surface = WindowSurface::new(window.id(), bounds, &root)?;
        surface.flush();
        Ok(Some(Self {
            surface,
            window,
            root,
            cells,
            highlight,
            label,
            frame,
            relative_window: None,
            visible: false,
        }))
    }

    pub fn show(&mut self, target: DropTarget) -> Result<bool, CgsWindowError> {
        let Some(geometry) = zone_geometry(&target) else {
            self.hide();
            return Ok(false);
        };
        let frame_changed = !self.frame.same_as(geometry.screen);
        if frame_changed {
            let transaction = WindowTransaction::new()?;
            transaction.set_rounded_frame(&self.window, geometry.screen, 0.0)?;
            self.surface.set_bounds_in(
                &transaction,
                CGRect::new(CGPoint::new(0.0, 0.0), geometry.screen.size),
            );
            transaction.commit();
            self.frame = geometry.screen;
            with_disabled_actions(|| {
                self.root.setBounds(CGRect::new(CGPoint::new(0.0, 0.0), geometry.screen.size));
            });
        }
        with_disabled_actions(|| {
            for (index, cell) in self.cells.iter().enumerate() {
                let Some((rect, _)) = geometry.cells.get(index) else {
                    cell.setOpacity(0.0);
                    continue;
                };
                cell.setFrame(CGRect::new(
                    CGPoint::new(
                        rect.origin.x - geometry.screen.origin.x,
                        rect.origin.y - geometry.screen.origin.y,
                    ),
                    rect.size,
                ));
                cell.setOpacity(1.0);
            }
            let candidate = CGRect::new(
                CGPoint::new(
                    geometry.candidate.origin.x - geometry.screen.origin.x,
                    geometry.candidate.origin.y - geometry.screen.origin.y,
                ),
                geometry.candidate.size,
            );
            self.highlight.setFrame(candidate);
            let label_width = 150.0_f64.min(candidate.size.width - 16.0).max(80.0);
            self.label.setFrame(CGRect::new(
                CGPoint::new(candidate.origin.x + 8.0, candidate.origin.y + 8.0),
                CGSize::new(label_width, 26.0),
            ));
            let text = CFString::from_str(&geometry.label);
            unsafe {
                self.label.setString(Some(&*(text.as_ref() as *const AnyObject)));
            }
            self.label.setOpacity(1.0);
            self.root.setOpacity(1.0);
        });
        let relative_window = target.intent.window.idx.get();
        let ordering_changed = !self.visible || self.relative_window != Some(relative_window);
        if ordering_changed {
            if !self.visible {
                self.window.set_alpha(1.0)?;
            }
            self.window.order_above(Some(relative_window))?;
            self.relative_window = Some(relative_window);
        }
        self.surface.flush();
        self.visible = true;
        Ok(true)
    }

    pub fn hide(&mut self) {
        if self.visible {
            with_disabled_actions(|| self.root.setOpacity(0.0));
            self.surface.flush();
            let _ = self.window.set_alpha(0.0);
            let _ = self.window.order_out();
            self.visible = false;
        }
    }
}
