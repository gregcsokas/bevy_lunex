use crate::*;

// Exported prelude
pub mod prelude {
    // All standard exports
    pub use super::UiFlowText;
}


// #===========================#
// #=== TEXT FLOW SIZING CONTROL ===#

/// **Ui Flow Text** - Attach this component to a flow text node to opt into flow-aware text sizing.
/// When `wrap` is enabled, the node's assigned width is fed back into [`TextBounds`] each recompute,
/// causing the text to re-wrap to the width the flow engine gave it. The layout then settles
/// on the next recompute (the wrapped height feeds back into the flow).
///
/// Affected components:
/// - [`TextBounds`] - The maximum text width is overwritten to match the flow-assigned width
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*;
/// # fn spawn_text(mut commands: Commands) {
/// commands.spawn((
///     UiLayout::flow().width(UiFlowSize::Fit).pack(),
///     UiFlowText::wrapped(),
///     Text2d::new("Some wrapping paragraph"),
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct UiFlowText {
    /// Whether the text should wrap to the width assigned by the flow engine.
    pub wrap: bool,
}
impl UiFlowText {
    /// Creates text flow sizing with wrapping enabled.
    pub fn wrapped() -> Self {
        Self { wrap: true }
    }
    /// Creates text flow sizing with wrapping disabled.
    pub fn unwrapped() -> Self {
        Self { wrap: false }
    }
}


// #=========================#
// #=== THE FLOW ALGORITHM ===#

/// Epsilon used for float comparisons.
const FLOW_EPSILON: f32 = 0.01;

/// Evaluation context passed through the flow algorithm.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UiFlowContext {
    /// Scale applied to absolute units (from [`UiLayoutRoot::abs_scale`]).
    pub abs_scale: f32,
    /// Size of the layout root (viewport) used by viewport units.
    pub viewport: Vec2,
    /// Font size used by the `Em` unit.
    pub font_size: f32,
}

/// Gets a component of a vector by axis index (`0` = x, `1` = y).
fn axis_get(v: Vec2, axis: usize) -> f32 {
    if axis == 0 { v.x } else { v.y }
}
/// Sets a component of a vector by axis index (`0` = x, `1` = y).
fn axis_set(v: &mut Vec2, axis: usize, value: f32) {
    if axis == 0 { v.x = value } else { v.y = value }
}
/// Converts an [`Align`] in the `-1.0` to `1.0` range into a `0.0` to `1.0` factor.
fn align_factor(align: Align) -> f32 {
    (align.0 + 1.0) / 2.0
}

/// Evaluates padding with relative units dropped (used before the node's own size is known).
fn eval_padding_intrinsic(config: &UiLayoutTypeFlow, ctx: &UiFlowContext) -> (f32, f32, f32, f32) {
    let l = config.padding.left.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 0);
    let r = config.padding.right.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 0);
    let t = config.padding.top.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 1);
    let b = config.padding.bottom.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 1);
    (l, r, t, b)
}

/// Evaluates padding fully, resolving relative units against the node's own size.
fn eval_padding_full(config: &UiLayoutTypeFlow, own: Vec2, ctx: &UiFlowContext) -> (f32, f32, f32, f32) {
    let l = config.padding.left.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, 0);
    let r = config.padding.right.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, 0);
    let t = config.padding.top.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, 1);
    let b = config.padding.bottom.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, 1);
    (l, r, t, b)
}

/// One node inside the flow tree. The tree is a flat arena of these, linked by indices.
#[doc(hidden)]
pub struct FlowItem {
    /// The entity this item was extracted from.
    pub entity: Entity,
    /// Index of the flow parent, if any. `None` marks a maximal flow subtree root.
    pub parent: Option<usize>,
    /// Indices of the flow children.
    pub children: Vec<usize>,
    /// The state-blended flow configuration snapshot.
    pub config: UiLayoutTypeFlow,
    /// Measured content size (text/image). `None` for plain containers.
    pub intrinsic: Option<Vec2>,
    /// Whether the intrinsic size comes from wrap-enabled text.
    pub wrap_text: bool,
    /// Bottom-up content-hugging size.
    pub content: Vec2,
    /// Bottom-up minimum size (shrink floor propagated from children).
    pub min: Vec2,
    /// Final computed size.
    pub size: Vec2,
    /// Final computed position, relative to the parent's top-left corner (y-down).
    pub pos: Vec2,
}
impl FlowItem {
    /// Creates a new flow item from a configuration.
    pub(crate) fn new(config: UiLayoutTypeFlow) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            parent: None,
            children: Vec::new(),
            config,
            intrinsic: None,
            wrap_text: false,
            content: Vec2::ZERO,
            min: Vec2::ZERO,
            size: Vec2::ZERO,
            pos: Vec2::ZERO,
        }
    }
    /// Attaches an intrinsic (measured) size to the item.
    pub(crate) fn with_intrinsic(mut self, intrinsic: Vec2) -> Self {
        self.intrinsic = Some(intrinsic);
        self
    }
    /// Marks the item's intrinsic size as wrap-enabled text.
    pub(crate) fn with_wrap_text(mut self) -> Self {
        self.wrap_text = true;
        self
    }
}

/// The flow layout engine. Holds a flat arena of [`FlowItem`]s extracted from the entity hierarchy
/// and computes their sizes and positions:
///
/// 1. **Bottom-up pass** - computes content-hugging sizes and minimum sizes from the leaves up.
/// 2. **Root resolution** - sizes the subtree root inside its (non-flow) parent's box.
/// 3. **Top-down pass** (BFS) - resolves relative sizing, then redistributes space:
///    on overflow children shrink water-level (largest first, floored at their minimums),
///    on leftover space `Grow` children expand water-fill (smallest first, capped at their maximums).
/// 4. **Position pass** - assigns each child's position from padding, alignment, gap and child sizes.
#[derive(Default)]
#[doc(hidden)]
pub struct FlowLayout {
    items: Vec<FlowItem>,
}
impl FlowLayout {
    /// Clears the arena, retaining its capacity for reuse across frames.
    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }
    /// Pushes a new item into the arena, attaching it to `parent` if given.
    pub(crate) fn push(&mut self, parent: Option<usize>, mut item: FlowItem) -> usize {
        let index = self.items.len();
        item.parent = parent;
        if let Some(parent) = parent {
            self.items[parent].children.push(index);
        }
        self.items.push(item);
        index
    }
    /// Attaches an existing item to a new parent (used for the virtual root container).
    pub(crate) fn reparent(&mut self, child: usize, parent: usize) {
        self.items[child].parent = Some(parent);
        self.items[parent].children.push(child);
    }
    /// Whether the item is the root of a maximal flow subtree (has no flow parent).
    pub(crate) fn is_root(&self, index: usize) -> bool {
        self.items[index].parent.is_none()
    }
    /// Returns the computed `(position, size)` of an item. The position is relative to the
    /// parent's top-left corner, with y pointing down.
    pub(crate) fn result(&self, index: usize) -> (Vec2, Vec2) {
        let item = &self.items[index];
        (item.pos, item.size)
    }
    /// Returns the entity of an item.
    pub(crate) fn entity(&self, index: usize) -> Entity {
        self.items[index].entity
    }
    /// Returns the wrap-text width assigned to an item, if it is wrap-enabled text.
    pub(crate) fn wrap_text_width(&self, index: usize) -> Option<f32> {
        let item = &self.items[index];
        if item.wrap_text { Some(item.size.x.max(0.0)) } else { None }
    }
    /// Iterates all item indices in the subtree rooted at `index` (preorder).
    pub(crate) fn subtree(&self, index: usize) -> Vec<usize> {
        let mut order = Vec::new();
        let mut stack = vec![index];
        while let Some(i) = stack.pop() {
            order.push(i);
            stack.extend(self.items[i].children.iter().rev().copied());
        }
        order
    }

    /// Computes sizes and positions of the whole subtree rooted at `index`.
    /// `parent_size` is the size of the subtree root's (non-flow) parent's box.
    pub(crate) fn compute(&mut self, index: usize, parent_size: Vec2, ctx: &UiFlowContext) {
        // === Phase 1: bottom-up content pass (children before parents) ===
        let order = self.subtree(index);
        for &i in order.iter().rev() {
            self.compute_content(i, ctx);
        }

        // === Phase 2: resolve the subtree root inside its parent ===
        self.resolve_root(index, parent_size, ctx);

        // === Phase 3: top-down sizing pass (parents before children) ===
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(index);
        while let Some(i) = queue.pop_front() {
            self.process(i, ctx);
            for &child in &self.items[i].children.clone() {
                queue.push_back(child);
            }
        }

        // === Phase 4: position pass ===
        self.place(index, ctx);
    }

    /// Computes the content-hugging size and minimum size of an item from its children.
    fn compute_content(&mut self, index: usize, ctx: &UiFlowContext) {
        let config = self.items[index].config;
        let axis = match config.direction { UiFlowDirection::LeftToRight => 0, UiFlowDirection::TopToBottom => 1 };
        let cross = 1 - axis;

        // Relative units cannot be resolved yet (own size unknown) - drop them.
        let (pl, pr, pt, pb) = eval_padding_intrinsic(&config, ctx);
        let pad_x = pl + pr;
        let pad_y = pt + pb;
        let gap = config.gap.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, axis);
        let child_count = self.items[index].children.len();
        let gap_total = gap * child_count.saturating_sub(1) as f32;

        let (mut content, mut min) = if let Some(intrinsic) = self.items[index].intrinsic {
            // Measured leaf (text/image): content is the measured size, minimum is the same
            // unless the text can re-wrap (then it can shrink to zero width).
            let min = if self.items[index].wrap_text { Vec2::new(0.0, intrinsic.y) } else { intrinsic };
            (intrinsic, min)
        } else {
            // Container: hug the children.
            let mut along_c: f32 = 0.0; let mut along_m: f32 = 0.0;
            let mut cross_c: f32 = 0.0; let mut cross_m: f32 = 0.0;
            for &child in &self.items[index].children.clone() {
                along_c += axis_get(self.items[child].content, axis);
                along_m += axis_get(self.items[child].min, axis);
                cross_c = cross_c.max(axis_get(self.items[child].content, cross));
                cross_m = cross_m.max(axis_get(self.items[child].min, cross));
            }
            let mut content = Vec2::ZERO;
            let mut min = Vec2::ZERO;
            // Along the flow axis, padding always applies.
            axis_set(&mut content, axis, along_c + gap_total + axis_get(Vec2::new(pad_x, pad_y), axis));
            axis_set(&mut min, axis, along_m + gap_total + axis_get(Vec2::new(pad_x, pad_y), axis));
            // Across the flow axis, an empty container hugs to zero (padding does not inflate it).
            if child_count > 0 {
                axis_set(&mut content, cross, cross_c + axis_get(Vec2::new(pad_x, pad_y), cross));
                axis_set(&mut min, cross, cross_m + axis_get(Vec2::new(pad_x, pad_y), cross));
            }
            (content, min)
        };

        // Fixed sizing overrides the content size (relative parts dropped here).
        // (min = max = value), the minimum equals the fixed size.
        if let UiFlowSize::Fixed(v) = config.width.size {
            content.x = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 0);
            min.x = content.x;
        }
        if let UiFlowSize::Fixed(v) = config.height.size {
            content.y = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 1);
            min.y = content.y;
        }

        // Clamp to own min/max (only the always-resolvable parts of the units).
        if let Some(v) = config.width.min { let m = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 0); content.x = content.x.max(m); min.x = min.x.max(m); }
        if let Some(v) = config.width.max { let m = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 0); content.x = content.x.min(m); min.x = min.x.min(m); }
        if let Some(v) = config.height.min { let m = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 1); content.y = content.y.max(m); min.y = min.y.max(m); }
        if let Some(v) = config.height.max { let m = v.evaluate_intrinsic(ctx.abs_scale, ctx.viewport, ctx.font_size, 1); content.y = content.y.min(m); min.y = min.y.min(m); }

        self.items[index].content = content.max(Vec2::ZERO);
        self.items[index].min = min.max(Vec2::ZERO);
        // Initial size is the content size; the top-down pass resolves the rest.
        self.items[index].size = self.items[index].content;
    }

    /// Resolves the subtree root's own size and position inside its non-flow parent's box.
    fn resolve_root(&mut self, index: usize, parent_size: Vec2, ctx: &UiFlowContext) {
        let config = self.items[index].config;
        for axis in 0..2 {
            let sizing = if axis == 0 { config.width } else { config.height };
            let value = match sizing.size {
                UiFlowSize::Fit => axis_get(self.items[index].content, axis),
                UiFlowSize::Grow => axis_get(parent_size, axis),
                UiFlowSize::Fixed(v) => v.evaluate_axis(ctx.abs_scale, parent_size, ctx.viewport, ctx.font_size, axis),
            };
            let mut value = value.max(0.0);
            if let Some(v) = sizing.min { value = value.max(v.evaluate_axis(ctx.abs_scale, parent_size, ctx.viewport, ctx.font_size, axis)) }
            if let Some(v) = sizing.max { value = value.min(v.evaluate_axis(ctx.abs_scale, parent_size, ctx.viewport, ctx.font_size, axis)) }
            axis_set(&mut self.items[index].size, axis, value);
        }
        // Place the root inside the parent according to alignment (top-left relative, unclamped).
        let size = self.items[index].size;
        self.items[index].pos = Vec2::new(
            (parent_size.x - size.x) * align_factor(config.align_x),
            (parent_size.y - size.y) * align_factor(config.align_y),
        );
    }

    /// Evaluates a child's effective minimum along an axis (bottom-up minimum raised by its min clamp).
    fn effective_min(&self, child: usize, axis: usize, base: Vec2, ctx: &UiFlowContext) -> f32 {
        let item = &self.items[child];
        let mut value = axis_get(item.min, axis);
        let clamp = if axis == 0 { item.config.width.min } else { item.config.height.min };
        if let Some(v) = clamp {
            value = value.max(v.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, axis));
        }
        value
    }
    /// Evaluates a child's effective maximum along an axis (its max clamp, or infinity).
    fn effective_max(&self, child: usize, axis: usize, base: Vec2, ctx: &UiFlowContext) -> f32 {
        let item = &self.items[child];
        let clamp = if axis == 0 { item.config.width.max } else { item.config.height.max };
        match clamp {
            Some(v) => v.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, axis),
            None => f32::INFINITY,
        }
    }

    /// The top-down BFS visit of one container. Resolves `Fixed` children, redistributes space
    /// along the flow axis (shrink on overflow, grow on leftover) and clamps/fills the cross axis.
    fn process(&mut self, index: usize, ctx: &UiFlowContext) {
        let config = self.items[index].config;
        let axis = match config.direction { UiFlowDirection::LeftToRight => 0, UiFlowDirection::TopToBottom => 1 };
        let cross = 1 - axis;
        let own = self.items[index].size;
        let (pl, pr, pt, pb) = eval_padding_full(&config, own, ctx);
        let inner = Vec2::new((own.x - pl - pr).max(0.0), (own.y - pt - pb).max(0.0));
        let gap = config.gap.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, axis).max(0.0);
        let children = self.items[index].children.clone();
        let child_count = children.len();
        let total_gaps = gap * child_count.saturating_sub(1) as f32;

        // Relative units of children resolve against the parent's inner content box,
        // minus the gaps along the flow axis.
        let base_along = (axis_get(inner, axis) - total_gaps).max(0.0);
        let base_cross = axis_get(inner, cross);
        let base = Vec2::new(
            if axis == 0 { base_along } else { base_cross },
            if axis == 1 { base_along } else { base_cross },
        );

        // === Resolve Fixed children (relative units now resolvable) ===
        for &child in &children {
            let child_config = self.items[child].config;
            if let UiFlowSize::Fixed(v) = child_config.width.size {
                let mut value = v.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 0).max(0.0);
                if let Some(m) = child_config.width.min { value = value.max(m.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 0)) }
                if let Some(m) = child_config.width.max { value = value.min(m.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 0)) }
                self.items[child].size.x = value;
            }
            if let UiFlowSize::Fixed(v) = child_config.height.size {
                let mut value = v.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 1).max(0.0);
                if let Some(m) = child_config.height.min { value = value.max(m.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 1)) }
                if let Some(m) = child_config.height.max { value = value.min(m.evaluate_axis(ctx.abs_scale, base, ctx.viewport, ctx.font_size, 1)) }
                self.items[child].size.y = value;
            }
        }

        // === Along-axis redistribution ===
        let mut content = total_gaps;
        for &child in &children {
            content += axis_get(self.items[child].size, axis);
        }
        let to_distribute = axis_get(inner, axis) - content;
        if to_distribute < 0.0 {
            // Overflow: shrink water-level, largest children first.
            let resizable = self.resizable_children(&children, axis);
            self.redistribute_shrink(&resizable, axis, to_distribute, base, ctx);
        } else if to_distribute > 0.0 {
            // Leftover: grow Grow children water-fill, smallest first.
            let grow: Vec<usize> = self.resizable_children(&children, axis)
                .into_iter()
                .filter(|&child| matches!(
                    if axis == 0 { self.items[child].config.width.size } else { self.items[child].config.height.size },
                    UiFlowSize::Grow
                ))
                .collect();
            if !grow.is_empty() {
                self.redistribute_grow(&grow, axis, to_distribute, base, ctx);
            }
        }

        // === Cross-axis resolution ===
        let resizable_cross = self.resizable_children(&children, cross);
        for &child in &resizable_cross {
            let max_available = axis_get(inner, cross);
            let min_floor = self.effective_min(child, cross, base, ctx);
            let mut value = axis_get(self.items[child].size, cross);
            let is_grow = matches!(
                if cross == 0 { self.items[child].config.width.size } else { self.items[child].config.height.size },
                UiFlowSize::Grow
            );
            if is_grow {
                value = max_available.min(self.effective_max(child, cross, base, ctx));
            }
            value = min_floor.max(value.min(max_available));
            axis_set(&mut self.items[child].size, cross, value);
        }
    }

    /// Children eligible for redistribution along an axis: `Fit` or `Grow` sizing
    /// (measured nodes only if their text can wrap).
    fn resizable_children(&self, children: &[usize], axis: usize) -> Vec<usize> {
        children.iter().copied().filter(|&child| {
            let item = &self.items[child];
            let sizing = if axis == 0 { item.config.width.size } else { item.config.height.size };
            let sized = matches!(sizing, UiFlowSize::Fit | UiFlowSize::Grow);
            let shrinkable_intrinsic = item.intrinsic.is_none() || item.wrap_text;
            sized && shrinkable_intrinsic
        }).collect()
    }

    /// Shrinks the largest children first towards the size of the next largest ("water level"),
    /// flooring each at its minimum size.
    fn redistribute_shrink(&mut self, resizable: &[usize], axis: usize, mut to_distribute: f32, base: Vec2, ctx: &UiFlowContext) {
        let mut list: Vec<usize> = resizable.to_vec();
        while to_distribute < -FLOW_EPSILON && !list.is_empty() {
            let mut largest = 0.0;
            let mut second_largest = 0.0;
            let mut width_to_add = to_distribute;
            for &child in &list {
                let size = axis_get(self.items[child].size, axis);
                if (size - largest).abs() < FLOW_EPSILON { continue }
                if size > largest { second_largest = largest; largest = size }
                if size < largest { second_largest = second_largest.max(size); width_to_add = second_largest - largest }
            }
            width_to_add = width_to_add.max(to_distribute / list.len() as f32);

            let mut progress = false;
            let mut k = 0;
            while k < list.len() {
                let child = list[k];
                let previous = axis_get(self.items[child].size, axis);
                if (previous - largest).abs() < FLOW_EPSILON {
                    let min_size = self.effective_min(child, axis, base, ctx);
                    let mut new = previous + width_to_add;
                    if new <= min_size {
                        new = min_size;
                        list.swap_remove(k);
                    } else {
                        k += 1;
                    }
                    axis_set(&mut self.items[child].size, axis, new);
                    to_distribute -= new - previous;
                    progress = true;
                } else {
                    k += 1;
                }
            }
            if !progress { break }
        }
    }

    /// Grows the smallest children first towards the size of the next smallest ("water fill"),
    /// capping each at its maximum size.
    fn redistribute_grow(&mut self, grow: &[usize], axis: usize, mut to_distribute: f32, base: Vec2, ctx: &UiFlowContext) {
        let mut list: Vec<usize> = grow.to_vec();
        while to_distribute > FLOW_EPSILON && !list.is_empty() {
            let mut smallest = f32::MAX;
            let mut second_smallest = f32::MAX;
            let mut width_to_add = to_distribute;
            for &child in &list {
                let size = axis_get(self.items[child].size, axis);
                if (size - smallest).abs() < FLOW_EPSILON { continue }
                if size < smallest { second_smallest = smallest; smallest = size }
                if size > smallest { second_smallest = second_smallest.min(size); width_to_add = second_smallest - smallest }
            }
            width_to_add = width_to_add.min(to_distribute / list.len() as f32);

            let mut progress = false;
            let mut k = 0;
            while k < list.len() {
                let child = list[k];
                let previous = axis_get(self.items[child].size, axis);
                if (previous - smallest).abs() < FLOW_EPSILON {
                    let max_size = self.effective_max(child, axis, base, ctx);
                    let mut new = previous + width_to_add;
                    if new >= max_size {
                        new = max_size;
                        list.swap_remove(k);
                    } else {
                        k += 1;
                    }
                    axis_set(&mut self.items[child].size, axis, new);
                    to_distribute -= new - previous;
                    progress = true;
                } else {
                    k += 1;
                }
            }
            if !progress { break }
        }
    }

    /// Assigns positions to all children in the subtree (relative to their parent's top-left, y-down).
    fn place(&mut self, index: usize, ctx: &UiFlowContext) {
        let mut stack = vec![index];
        while let Some(i) = stack.pop() {
            let config = self.items[i].config;
            let axis = match config.direction { UiFlowDirection::LeftToRight => 0, UiFlowDirection::TopToBottom => 1 };
            let cross = 1 - axis;
            let own = self.items[i].size;
            let (pl, pr, pt, pb) = eval_padding_full(&config, own, ctx);
            let inner = Vec2::new(own.x - pl - pr, own.y - pt - pb);
            let gap = config.gap.evaluate_axis(ctx.abs_scale, own, ctx.viewport, ctx.font_size, axis).max(0.0);
            let children = self.items[i].children.clone();
            let pad_start = Vec2::new(pl, pt);

            // On-axis: distribute leftover space by alignment (clamped to zero).
            let mut content = gap * children.len().saturating_sub(1) as f32;
            for &child in &children {
                content += axis_get(self.items[child].size, axis);
            }
            let extra = (axis_get(inner, axis) - content).max(0.0);
            let align = if axis == 0 { config.align_x } else { config.align_y };
            let mut offset = axis_get(pad_start, axis) + extra * align_factor(align);

            for &child in &children {
                let child_size = self.items[child].size;
                // Cross-axis: align this child within the leftover whitespace (unclamped).
                let whitespace = axis_get(inner, cross) - axis_get(child_size, cross);
                let align_cross = if cross == 0 { config.align_x } else { config.align_y };
                let mut pos = Vec2::ZERO;
                axis_set(&mut pos, axis, offset);
                axis_set(&mut pos, cross, axis_get(pad_start, cross) + whitespace * align_factor(align_cross));
                self.items[child].pos = pos;
                offset += axis_get(child_size, axis) + gap;
            }
            stack.extend(children.iter().rev().copied());
        }
    }
}


// #========================#
// #=== STATE BLENDING CONTROL ===#

/// Resolves the active flow configuration of a [`UiLayout`] by blending the flow parameters of all
/// active states weighted by their transition values. The node participates in the flow only if
/// its [`UiBase`] layout is of the flow type; state layouts of other kinds are ignored.
pub(crate) fn resolve_flow_config(layout: &UiLayout, state: &UiState) -> Option<UiLayoutTypeFlow> {
    let UiLayoutType::Flow(base) = layout.layouts.get(&UiBase::id())? else { return None };

    // Collect active states that define flow layouts.
    let mut entries: Vec<(f32, UiLayoutTypeFlow)> = Vec::new();
    for (state_id, layout_type) in &layout.layouts {
        let UiLayoutType::Flow(config) = layout_type else { continue };
        if let Some(weight) = state.weight(state_id) && weight > 0.0 {
            entries.push((weight, *config));
        }
    }

    // No active states - fall back to the base layout (mirrors the absolute layout blending).
    if entries.is_empty() { return Some(*base) }
    Some(blend_flow_config(&entries))
}

/// Blends multiple weighted flow configurations into one by interpolating numeric parameters.
/// Non-blendable fields (direction, mismatching sizing kinds) are taken from the highest-weight entry.
fn blend_flow_config(entries: &[(f32, UiLayoutTypeFlow)]) -> UiLayoutTypeFlow {
    let (_, dominant) = entries.iter().max_by(|a, b| a.0.total_cmp(&b.0)).expect("entries are non-empty");

    // Incremental weighted mean: acc = lerp(acc, entry, w / (acc_w + w)).
    let mut acc = UiLayoutTypeFlow::new();
    let mut acc_weight = 0.0;
    for (weight, config) in entries {
        let t = if acc_weight + weight > 0.0 { weight / (acc_weight + weight) } else { 0.0 };
        acc = lerp_flow_config(&acc, config, t);
        acc_weight += weight;
    }

    // Non-blendable fields come from the dominant entry.
    acc.direction = dominant.direction;
    let all_same = |predicate: &dyn Fn(&UiLayoutTypeFlow) -> bool| entries.iter().all(|(_, config)| predicate(config));
    if !all_same(&|c| size_kind(&c.width.size) == size_kind(&dominant.width.size)) { acc.width.size = dominant.width.size }
    if !all_same(&|c| size_kind(&c.height.size) == size_kind(&dominant.height.size)) { acc.height.size = dominant.height.size }
    if !all_same(&|c| c.width.min.is_some()) { acc.width.min = dominant.width.min }
    if !all_same(&|c| c.width.max.is_some()) { acc.width.max = dominant.width.max }
    if !all_same(&|c| c.height.min.is_some()) { acc.height.min = dominant.height.min }
    if !all_same(&|c| c.height.max.is_some()) { acc.height.max = dominant.height.max }
    acc
}

/// Kind discriminator for [`UiFlowSize`] used to detect unblendable variant mixes.
fn size_kind(size: &UiFlowSize) -> u8 {
    match size {
        UiFlowSize::Fit => 0,
        UiFlowSize::Grow => 1,
        UiFlowSize::Fixed(_) => 2,
    }
}

/// Interpolates between two flow configurations.
fn lerp_flow_config(a: &UiLayoutTypeFlow, b: &UiLayoutTypeFlow, t: f32) -> UiLayoutTypeFlow {
    let lerp_option = |a: &Option<UiValue<f32>>, b: &Option<UiValue<f32>>| match (a, b) {
        (Some(a), Some(b)) => Some(a.lerp(b, t)),
        (Some(a), None) => Some(*a),
        (None, Some(b)) => Some(*b),
        (None, None) => None,
    };
    UiLayoutTypeFlow {
        direction: if t >= 0.5 { b.direction } else { a.direction },
        width: UiFlowAxis {
            size: match (&a.width.size, &b.width.size) {
                (UiFlowSize::Fixed(va), UiFlowSize::Fixed(vb)) => UiFlowSize::Fixed(va.lerp(vb, t)),
                _ => if t >= 0.5 { b.width.size } else { a.width.size },
            },
            min: lerp_option(&a.width.min, &b.width.min),
            max: lerp_option(&a.width.max, &b.width.max),
        },
        height: UiFlowAxis {
            size: match (&a.height.size, &b.height.size) {
                (UiFlowSize::Fixed(va), UiFlowSize::Fixed(vb)) => UiFlowSize::Fixed(va.lerp(vb, t)),
                _ => if t >= 0.5 { b.height.size } else { a.height.size },
            },
            min: lerp_option(&a.height.min, &b.height.min),
            max: lerp_option(&a.height.max, &b.height.max),
        },
        padding: UiFlowPadding {
            left: a.padding.left.lerp(&b.padding.left, t),
            right: a.padding.right.lerp(&b.padding.right, t),
            top: a.padding.top.lerp(&b.padding.top, t),
            bottom: a.padding.bottom.lerp(&b.padding.bottom, t),
        },
        gap: a.gap.lerp(&b.gap, t),
        align_x: Align(a.align_x.0 + (b.align_x.0 - a.align_x.0) * t),
        align_y: Align(a.align_y.0 + (b.align_y.0 - a.align_y.0) * t),
    }
}


// #================================================================#
// #=== UNIT TESTS - verifying the flow algorithm in isolation  ===#

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> UiFlowContext {
        UiFlowContext { abs_scale: 1.0, viewport: Vec2::new(1000.0, 600.0), font_size: 16.0 }
    }
    /// A leaf with an explicit fixed size.
    fn fixed(size: Vec2) -> FlowItem {
        FlowItem::new(UiLayoutTypeFlow::new().width(size.x).height(size.y))
    }
    /// A `Fit`-sized container whose content hugs the given size (via a fixed child).
    fn fit(flow: &mut FlowLayout, parent: usize, size: Vec2) -> usize {
        let item = flow.push(Some(parent), FlowItem::new(UiLayoutTypeFlow::new()));
        flow.push(Some(item), fixed(size));
        item
    }
    /// A `Fit`-sized wrap-enabled text leaf with a measured size (shrinkable to zero width).
    fn wrap_text(size: Vec2) -> FlowItem {
        FlowItem::new(UiLayoutTypeFlow::new()).with_intrinsic(size).with_wrap_text()
    }
    /// A `Grow`-sized container with a content size of zero.
    fn grow() -> FlowItem {
        FlowItem::new(UiLayoutTypeFlow::new().width(UiFlowSize::Grow).height(UiFlowSize::Grow))
    }
    fn approx(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() <= epsilon
    }
    fn assert_vec2(actual: Vec2, expected: Vec2) {
        assert!(approx(actual.x, expected.x, 0.1) && approx(actual.y, expected.y, 0.1),
            "expected {expected:?}, got {actual:?}");
    }

    #[test]
    fn row_hugs_content() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new()));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), fixed(Vec2::new(200.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        let (pos, size) = flow.result(root);
        assert_vec2(pos, Vec2::ZERO);
        assert_vec2(size, Vec2::new(300.0, 50.0));
        let (pos, size) = flow.result(a);
        assert_vec2(pos, Vec2::ZERO);
        assert_vec2(size, Vec2::new(100.0, 50.0));
        let (pos, _) = flow.result(b);
        assert_vec2(pos, Vec2::new(100.0, 0.0));
    }

    #[test]
    fn row_gap_and_padding() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new()
            .gap(Ab(10.0)).padding_all(Ab(20.0))));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // width = 20 + 100 + 10 + 100 + 20, height = 20 + 50 + 20
        let (_, size) = flow.result(root);
        assert_vec2(size, Vec2::new(250.0, 90.0));
        let (pos, _) = flow.result(a);
        assert_vec2(pos, Vec2::new(20.0, 20.0));
        let (pos, _) = flow.result(b);
        assert_vec2(pos, Vec2::new(130.0, 20.0));
    }

    #[test]
    fn column_layout() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new()
            .direction(UiFlowDirection::TopToBottom).gap(Ab(10.0))));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), fixed(Vec2::new(60.0, 30.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        let (_, size) = flow.result(root);
        assert_vec2(size, Vec2::new(100.0, 90.0));
        let (pos, _) = flow.result(a);
        assert_vec2(pos, Vec2::ZERO);
        let (pos, _) = flow.result(b);
        assert_vec2(pos, Vec2::new(0.0, 60.0));
    }

    #[test]
    fn grow_children_share_leftover() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(400.0).height(100.0)));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), grow());
        let c = flow.push(Some(root), grow());
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        assert_vec2(flow.result(a).1, Vec2::new(100.0, 50.0));
        // 300 leftover split between two grow children
        assert_vec2(flow.result(b).1, Vec2::new(150.0, 100.0));
        assert_vec2(flow.result(c).1, Vec2::new(150.0, 100.0));
        assert_vec2(flow.result(a).0, Vec2::ZERO);
        assert_vec2(flow.result(b).0, Vec2::new(100.0, 0.0));
        assert_vec2(flow.result(c).0, Vec2::new(250.0, 0.0));
    }

    #[test]
    fn grow_respects_max_clamp() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(400.0).height(100.0)));
        let _a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().width(UiFlowSize::Grow).max_width(Ab(50.0))));
        let c = flow.push(Some(root), grow());
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        assert_vec2(flow.result(b).1, Vec2::new(50.0, 0.0));
        // The grow child without a max clamp takes the rest (and fills the cross axis).
        assert_vec2(flow.result(c).1, Vec2::new(250.0, 100.0));
    }

    #[test]
    fn shrink_reduces_largest_first() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(250.0).height(100.0)));
        let a = flow.push(Some(root), wrap_text(Vec2::new(200.0, 50.0)));
        let b = flow.push(Some(root), wrap_text(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Deficit of 50: the larger child yields until both are ~150 wide.
        assert_vec2(flow.result(a).1, Vec2::new(150.0, 50.0));
        assert_vec2(flow.result(b).1, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn shrink_floors_at_minimum() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(100.0).height(100.0)));
        // Wrapping text with a minimum width clamp above the available space
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().min_width(Ab(120.0))).with_intrinsic(Vec2::new(200.0, 50.0)).with_wrap_text());
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // The child shrinks only down to its minimum of 120.
        assert_vec2(flow.result(a).1, Vec2::new(120.0, 50.0));
    }

    #[test]
    fn fixed_content_does_not_shrink() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(250.0).height(100.0)));
        // A fit container of fixed content is rigid: its minimum equals its content.
        let a = fit(&mut flow, root, Vec2::new(200.0, 50.0));
        let b = fit(&mut flow, root, Vec2::new(100.0, 50.0));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Neither child can shrink below its fixed content, so they overflow instead.
        assert_vec2(flow.result(a).1, Vec2::new(200.0, 50.0));
        assert_vec2(flow.result(b).1, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn fixed_percent_resolves_against_inner_size() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(100.0).height(100.0).gap(Ab(10.0))));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().width(Rl(50.0)).height(50.0)));
        let b = fit(&mut flow, root, Vec2::new(30.0, 40.0));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Percent base = (100 - 10 gap) = 90, so Rl(50) = 45.
        assert_vec2(flow.result(a).1, Vec2::new(45.0, 50.0));
        assert_vec2(flow.result(a).0, Vec2::ZERO);
        assert_vec2(flow.result(b).0, Vec2::new(55.0, 0.0));
    }

    #[test]
    fn relative_sizing_excluded_from_fit_hug() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new()));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().width(Rl(50.0)).height(10.0)));
        let b = fit(&mut flow, root, Vec2::new(40.0, 10.0));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // The Fit root hugs only the non-relative child (40 wide), then the
        // relative child resolves to 50% of that (20) and overflows alongside it.
        assert_vec2(flow.result(root).1, Vec2::new(40.0, 10.0));
        assert_vec2(flow.result(a).1, Vec2::new(20.0, 10.0));
        assert_vec2(flow.result(b).1, Vec2::new(40.0, 10.0));
    }

    #[test]
    fn min_max_clamps_on_fit_container() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().min_width(Ab(80.0)).max_width(Ab(120.0))));
        let _child = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());
        assert_vec2(flow.result(root).1, Vec2::new(100.0, 50.0));

        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().max_width(Ab(60.0))));
        let _child = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());
        assert_vec2(flow.result(root).1, Vec2::new(60.0, 50.0));
    }

    #[test]
    fn align_center_distributes_leftover() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(300.0).height(100.0).align_x(Align::CENTER).align_y(Align::CENTER)));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        let (pos, _) = flow.result(a);
        assert_vec2(pos, Vec2::new(100.0, 25.0));
    }

    #[test]
    fn align_end_shifts_content() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(300.0).height(100.0).align_x(Align::END)));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        let (pos, _) = flow.result(a);
        assert_vec2(pos, Vec2::new(200.0, 0.0));
    }

    #[test]
    fn cross_grow_fills_parent() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(300.0).height(100.0)));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().width(Ab(100.0)).height(UiFlowSize::Grow)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());
        assert_vec2(flow.result(a).1, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn cross_fit_clamped_to_parent() {
        let mut flow = FlowLayout::default();
        // A column parent with a narrow width: the wrapping text child is clamped to it.
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new()
            .direction(UiFlowDirection::TopToBottom).width(Ab(100.0)).height(Ab(100.0))));
        let a = flow.push(Some(root), wrap_text(Vec2::new(200.0, 20.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // The fit child is clamped down to the parent's inner width.
        assert_vec2(flow.result(a).1, Vec2::new(100.0, 20.0));
    }

    #[test]
    fn cross_min_floor_overflows_instead_of_clamping() {
        let mut flow = FlowLayout::default();
        // A row parent clamped to a small height by its own fixed size.
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(300.0).height(Ab(40.0))));
        let a = fit(&mut flow, root, Vec2::new(100.0, 50.0));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // The fit child is floored at its minimum and overflows instead of being clamped below it.
        assert_vec2(flow.result(a).1, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn nested_containers() {
        let mut flow = FlowLayout::default();
        // A classic sidebar layout: fixed sidebar (column with gap) + grow main area
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(1000.0).height(600.0)));
        let sidebar = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new()
            .direction(UiFlowDirection::TopToBottom).gap(Ab(10.0)).width(Ab(300.0)).height(UiFlowSize::Grow)));
        let main = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new().width(UiFlowSize::Grow).height(UiFlowSize::Grow)));
        let s1 = flow.push(Some(sidebar), fixed(Vec2::new(50.0, 100.0)));
        let s2 = flow.push(Some(sidebar), fixed(Vec2::new(50.0, 100.0)));

        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        assert_vec2(flow.result(sidebar).1, Vec2::new(300.0, 600.0));
        assert_vec2(flow.result(main).1, Vec2::new(700.0, 600.0));
        assert_vec2(flow.result(main).0, Vec2::new(300.0, 0.0));
        assert_vec2(flow.result(s1).0, Vec2::ZERO);
        assert_vec2(flow.result(s2).0, Vec2::new(0.0, 110.0));
    }

    #[test]
    fn intrinsic_leaf_sizing() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().gap(Ab(10.0))));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new()).with_intrinsic(Vec2::new(120.0, 30.0)));
        let b = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new()).with_intrinsic(Vec2::new(80.0, 30.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        assert_vec2(flow.result(root).1, Vec2::new(210.0, 30.0));
        assert_vec2(flow.result(a).1, Vec2::new(120.0, 30.0));
        assert_vec2(flow.result(b).0, Vec2::new(130.0, 0.0));
    }

    #[test]
    fn wrap_text_shrinks_and_keeps_height() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(100.0).height(100.0)));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new()).with_intrinsic(Vec2::new(200.0, 20.0)).with_wrap_text());
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Wrapping text has a zero-width minimum, so it shrinks to the container width.
        assert_vec2(flow.result(a).1, Vec2::new(100.0, 20.0));
    }

    #[test]
    fn unwrapped_text_does_not_shrink() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(100.0).height(100.0)));
        let a = flow.push(Some(root), FlowItem::new(UiLayoutTypeFlow::new()).with_intrinsic(Vec2::new(200.0, 20.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Non-wrapping text overflows instead of shrinking.
        assert_vec2(flow.result(a).1, Vec2::new(200.0, 20.0));
    }

    #[test]
    fn fit_root_placement_in_absolute_parent() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().align_x(Align::CENTER).align_y(Align::CENTER)));
        let _child = flow.push(Some(root), fixed(Vec2::new(300.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        let (pos, size) = flow.result(root);
        assert_vec2(size, Vec2::new(300.0, 50.0));
        assert_vec2(pos, Vec2::new(350.0, 275.0));
    }

    #[test]
    fn grow_root_fills_absolute_parent() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(UiFlowSize::Grow).height(UiFlowSize::Grow)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());
        assert_vec2(flow.result(root).1, Vec2::new(1000.0, 600.0));
        assert_vec2(flow.result(root).0, Vec2::ZERO);
    }

    #[test]
    fn units_resolve_in_flow() {
        let mut flow = FlowLayout::default();
        // gap of 1em (16px), padding of 10Vw (100px)
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().gap(Em(1.0)).padding_x(Vw(10.0))));
        let a = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        let b = flow.push(Some(root), fixed(Vec2::new(100.0, 50.0)));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // width = 100 + 100 + 100 + 16 + 100 = 416
        let (_, size) = flow.result(root);
        assert_vec2(size, Vec2::new(416.0, 50.0));
        let (pos, _) = flow.result(a);
        assert_vec2(pos, Vec2::new(100.0, 0.0));
        let (pos, _) = flow.result(b);
        assert_vec2(pos, Vec2::new(216.0, 0.0));
    }

    #[test]
    fn empty_container_has_zero_cross_size() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().padding_all(Ab(16.0))));
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());
        assert_vec2(flow.result(root).1, Vec2::new(32.0, 0.0));
    }

    #[test]
    fn grow_water_fill_levels_uneven_children() {
        let mut flow = FlowLayout::default();
        let root = flow.push(None, FlowItem::new(UiLayoutTypeFlow::new().width(400.0).height(100.0)));
        // Two grow children with different content sizes (100 and 200): they equalize.
        let a = fit(&mut flow, root, Vec2::new(100.0, 50.0));
        let b = fit(&mut flow, root, Vec2::new(200.0, 50.0));
        let a_config = flow.items[a].config;
        flow.items[a].config = UiLayoutTypeFlow { width: UiFlowAxis::new(UiFlowSize::Grow), ..a_config };
        let b_config = flow.items[b].config;
        flow.items[b].config = UiLayoutTypeFlow { width: UiFlowAxis::new(UiFlowSize::Grow), ..b_config };
        flow.compute(root, Vec2::new(1000.0, 600.0), &ctx());

        // Both end up at 200 (the smaller grows to the larger's level, then they share the rest).
        assert_vec2(flow.result(a).1, Vec2::new(200.0, 50.0));
        assert_vec2(flow.result(b).1, Vec2::new(200.0, 50.0));
    }

    #[test]
    fn state_blending_interpolates_parameters() {
        let base = UiLayoutTypeFlow::new().gap(Ab(10.0));
        let hover = UiLayoutTypeFlow::new().gap(Ab(20.0));
        let blended = blend_flow_config(&[(0.5, base), (0.5, hover)]);
        assert!(approx(blended.gap.evaluate_intrinsic(1.0, Vec2::new(1000.0, 600.0), 16.0, 0), 15.0, 0.001));
    }

    #[test]
    fn state_blending_picks_dominant_direction() {
        let row = UiLayoutTypeFlow::new();
        let column = UiLayoutTypeFlow::new().direction(UiFlowDirection::TopToBottom);
        let blended = blend_flow_config(&[(0.8, row), (0.2, column)]);
        assert_eq!(blended.direction, UiFlowDirection::LeftToRight);
        let blended = blend_flow_config(&[(0.2, row), (0.8, column)]);
        assert_eq!(blended.direction, UiFlowDirection::TopToBottom);
    }

    /// Mirrors the doc examples so they stay compile-checked without running the heavy doctests.
    #[test]
    fn public_api_doc_examples_compile() {
        let _size: UiFlowSize = UiFlowSize::Fit;
        let _size: UiFlowSize = UiFlowSize::Grow;
        let _size: UiFlowSize = UiFlowSize::Fixed(Ab(50.0).into());
        let _size: UiFlowSize = UiFlowSize::Fixed(Rl(50.0).into());
        let _axis: UiFlowAxis = UiFlowAxis::new(UiFlowSize::Grow).max(Rl(80.0).into());
        let _padding: UiFlowPadding = UiFlowPadding::x(Ab(16.0));
        let _layout: UiLayout = UiLayout::flow()
            .direction(UiFlowDirection::TopToBottom)
            .gap(Ab(8.0))
            .padding_all(Ab(16.0))
            .width(UiFlowSize::Grow)
            .height(Rl(50.0))
            .align_x(Align::CENTER)
            .pack();
    }
}
