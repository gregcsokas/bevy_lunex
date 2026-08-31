use crate::*;

// Exported prelude
pub mod prelude {
    // All standard exports
    pub use super::{
        Align,
        Scaling,
        UiFlowDirection,
        UiFlowSize,
        UiFlowAxis,
        UiFlowPadding,
        UiLayoutTypeFlow,
    };
}

// #============================#
// #=== MULTIPURPOSE STRUCTS ===#

/// **Rectangle 2D** - Contains computed values from node layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct Rectangle2D {
    pub pos : Vec2,
    pub size: Vec2,
}
impl Rectangle2D {
    pub fn lerp(self, rhs: Self, lerp: f32) -> Self {
        Rectangle2D {
            pos: self.pos.lerp(rhs.pos, lerp),
            size: self.size.lerp(rhs.size, lerp),
        }
    }
}
impl Rectangle2D {
    /// A new empty [`Rectangle2D`]. Has `0` size.
    pub const EMPTY: Rectangle2D = Rectangle2D { pos : Vec2::ZERO, size: Vec2::ZERO };
    /// Creates new empty Window layout.
    pub const fn new() -> Self {
        Rectangle2D::EMPTY
    }
    /// Replaces the position with the new value.
    pub fn with_pos(mut self, pos: impl Into<Vec2>) -> Self {
        self.pos = pos.into();
        self
    }
    /// Replaces the x position with the new value.
    pub fn with_x(mut self, width: f32) -> Self {
        self.pos.x = width;
        self
    }
    /// Replaces the y position with the new value.
    pub fn with_y(mut self, height: f32) -> Self {
        self.pos.y = height;
        self
    }
    /// Replaces the size with the new value.
    pub fn with_size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }
    /// Replaces the width with the new value.
    pub fn with_width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }
    /// Replaces the height with the new value.
    pub fn with_height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }
}

/// **Align** - A type used to define alignment in a node layout.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let align: Align = Align::START; // -> -1.0
/// let align: Align = Align(-1.0);  // -> -1.0
/// let align: Align = (-1.0).into();  // -> -1.0
/// ```
/// The expected range is `-1.0` to `1.0`, but you can extrapolate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct Align (pub f32);
impl Align {
    pub const START: Align = Align(-1.0);
    pub const LEFT: Align = Align(-1.0);
    pub const CENTER: Align = Align(0.0);
    pub const MIDDLE: Align = Align(0.0);
    pub const END: Align = Align(1.0);
    pub const RIGHT: Align = Align(1.0);
}
impl From<f32> for Align {
    fn from(val: f32) -> Self {
        Align(val)
    }
}


/// **Scaling** - A type used to define how should a Solid node layout scale relative to a parent.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let scaling: Scaling = Scaling::HorFill; // -> always cover the horizontal axis
/// let scaling: Scaling = Scaling::VerFill; // -> always cover the vertical axis
/// let scaling: Scaling = Scaling::Fit;  // -> always fit inside
/// let scaling: Scaling = Scaling::Fill; // -> always cover all
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub enum Scaling {
    /// Node layout should always cover the horizontal axis of the parent node.
    HorFill,
    /// Node layout should always cover the vertical axis of the parent node.
    VerFill,
    /// Node layout should always fit inside the parent node.
    #[default] Fit,
    /// Node layout should always cover all of the parent node.
    Fill,
}


// #=================#
// #=== FLOW TYPES ===#

/// **Ui Flow Direction** - A type used to define the direction in which child nodes
/// of a [`UiLayoutTypeFlow`] container are laid out.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let direction: UiFlowDirection = UiFlowDirection::TopToBottom; // -> stack children vertically
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum UiFlowDirection {
    /// Children are laid out from left to right with increasing x.
    #[default]
    LeftToRight,
    /// Children are laid out from top to bottom with increasing y.
    TopToBottom,
}

/// **Ui Flow Size** - A type used to define how a flow node takes up space inside its parent's flow.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let size: UiFlowSize = UiFlowSize::Fit;                          // -> hug the content
/// let size: UiFlowSize = UiFlowSize::Grow;                        // -> fill available space
/// let size: UiFlowSize = UiFlowSize::Fixed(Ab(50.0).into());      // -> exactly 50px
/// let size: UiFlowSize = UiFlowSize::Fixed(Rl(50.0).into());      // -> 50% of the parent's inner size
/// ```
///
/// Sizing with relative units ( [`Rl`], [`Rw`], [`Rh`] ) behaves like a percent of the parent's
/// inner content box and is resolved once the parent's size is known. Such nodes do not
/// contribute to the parent's content-hugging size.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub enum UiFlowSize {
    /// The node wraps tightly to the size of its contents.
    #[default]
    Fit,
    /// The node expands along this axis to fill available space in the parent, sharing it with other `Grow` nodes.
    Grow,
    /// The node is sized by an explicit [`UiValue`].
    Fixed(UiValue<f32>),
}
/// Conversions
impl From<UiValue<f32>> for UiFlowSize {
    fn from(value: UiValue<f32>) -> Self {
        UiFlowSize::Fixed(value)
    }
}
impl From<f32> for UiFlowSize {
    fn from(value: f32) -> Self {
        UiFlowSize::Fixed(Ab(value).into())
    }
}
/// Implement conversion of each unit into a fixed flow size
macro_rules! impl_flow_size_from_unit {
    ($($unit:ident), *) => {
        $(
            impl From<$unit<f32>> for UiFlowSize {
                fn from(value: $unit<f32>) -> Self {
                    UiFlowSize::Fixed(value.into())
                }
            }
        )*
    };
}
impl_flow_size_from_unit!(Ab, Rl, Rw, Rh, Em, Vp, Vw, Vh);

/// **Ui Flow Axis** - A type used to define the sizing of one axis of a [`UiLayoutTypeFlow`] node.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let axis: UiFlowAxis = UiFlowAxis::new(UiFlowSize::Grow).max(Rl(80.0).into());
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct UiFlowAxis {
    /// How the node takes up space along this axis.
    pub size: UiFlowSize,
    /// The smallest size the node is allowed to shrink to, overriding [`UiFlowSize::Fit`] and [`UiFlowSize::Grow`].
    pub min: Option<UiValue<f32>>,
    /// The largest size the node is allowed to grow to, overriding [`UiFlowSize::Fit`] and [`UiFlowSize::Grow`].
    pub max: Option<UiValue<f32>>,
}
/// Constructors
impl UiFlowAxis {
    /// Creates a new axis with the given sizing.
    pub const fn new(size: UiFlowSize) -> Self {
        Self { size, min: None, max: None }
    }
    /// Replaces the minimum clamp with a new value.
    pub const fn min(mut self, min: UiValue<f32>) -> Self {
        self.min = Some(min);
        self
    }
    /// Replaces the maximum clamp with a new value.
    pub const fn max(mut self, max: UiValue<f32>) -> Self {
        self.max = Some(max);
        self
    }
}
/// Conversions
impl From<UiFlowSize> for UiFlowAxis {
    fn from(value: UiFlowSize) -> Self {
        UiFlowAxis::new(value)
    }
}

/// **Ui Flow Padding** - A type used to define the padding of a [`UiLayoutTypeFlow`] node.
/// Padding is a gap between the bounding box of the node and where its children are placed.
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::*;
/// let padding: UiFlowPadding = UiFlowPadding::x(Ab(16.0));
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct UiFlowPadding {
    /// Gap between the node's left edge and its children.
    pub left: UiValue<f32>,
    /// Gap between the node's right edge and its children.
    pub right: UiValue<f32>,
    /// Gap between the node's top edge and its children.
    pub top: UiValue<f32>,
    /// Gap between the node's bottom edge and its children.
    pub bottom: UiValue<f32>,
}
/// Constructors
impl UiFlowPadding {
    /// Creates padding with the same value on all sides.
    pub fn all(value: impl Into<UiValue<f32>>) -> Self {
        let value = value.into();
        Self { left: value, right: value, top: value, bottom: value }
    }
    /// Creates padding on the horizontal axis only.
    pub fn x(value: impl Into<UiValue<f32>>) -> Self {
        let value = value.into();
        Self { left: value, right: value, ..Default::default() }
    }
    /// Creates padding on the vertical axis only.
    pub fn y(value: impl Into<UiValue<f32>>) -> Self {
        let value = value.into();
        Self { top: value, bottom: value, ..Default::default() }
    }
}

/// **Flow** - Dynamic layout type that participates in the ui flow. It is defined by how it takes
/// up space inside its parent's flow ([`UiFlowSize`]) and by how its children are arranged
/// ([`UiFlowDirection`], gap, padding, alignment). This is a flexbox-like layout model.
///
/// Nodes with this layout **are included in the ui flow** of their parent (if the parent also
/// uses a flow layout). Nodes with [`UiLayoutType::Boundary`], [`UiLayoutType::Window`] or
/// [`UiLayoutType::Solid`] layouts inside a flow container keep their absolute positioning.
///
/// ## Sizing semantics
/// - [`UiFlowSize::Fit`] - The node hugs its content (text, image or children).
/// - [`UiFlowSize::Grow`] - The node fills the available space, sharing it with other `Grow` siblings.
/// - [`UiFlowSize::Fixed`] - The node is sized explicitly. Relative units ([`Rl`], [`Rw`], [`Rh`])
///   resolve against the parent's inner content box (minus padding and gaps) and behave like
///   percentages. They do not contribute to a `Fit` parent's content-hugging size.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_lunex::{UiLayout, UiFlowSize, UiFlowDirection, Ab, Rl, Align};
/// let layout: UiLayout = UiLayout::flow()
///     .direction(UiFlowDirection::TopToBottom)
///     .gap(Ab(8.0))
///     .padding_all(Ab(16.0))
///     .width(UiFlowSize::Grow)
///     .height(UiFlowSize::Fit)
///     .align_x(Align::CENTER)
///     .pack();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct UiLayoutTypeFlow {
    /// The direction in which child nodes are laid out.
    pub direction: UiFlowDirection,
    /// The width sizing of the node inside its parent's flow.
    pub width: UiFlowAxis,
    /// The height sizing of the node inside its parent's flow.
    pub height: UiFlowAxis,
    /// The gap between the node's bounding box and its children.
    pub padding: UiFlowPadding,
    /// The gap between child nodes along the layout direction.
    pub gap: UiValue<f32>,
    /// The alignment of children along the horizontal axis.
    pub align_x: Align,
    /// The alignment of children along the vertical axis.
    pub align_y: Align,
}
impl Default for UiLayoutTypeFlow {
    fn default() -> Self {
        Self::new()
    }
}
/// Constructors
impl UiLayoutTypeFlow {
    /// Creates new empty Flow node layout.
    pub const fn new() -> Self {
        Self {
            direction: UiFlowDirection::LeftToRight,
            width: UiFlowAxis::new(UiFlowSize::Fit),
            height: UiFlowAxis::new(UiFlowSize::Fit),
            padding: UiFlowPadding { left: UiValue::new(), right: UiValue::new(), top: UiValue::new(), bottom: UiValue::new() },
            gap: UiValue::new(),
            align_x: Align::START,
            align_y: Align::START,
        }
    }
    /// Replaces the direction with a new value.
    pub const fn direction(mut self, direction: UiFlowDirection) -> Self {
        self.direction = direction;
        self
    }
    /// Replaces the width sizing with a new value.
    pub fn width(mut self, size: impl Into<UiFlowSize>) -> Self {
        self.width.size = size.into();
        self
    }
    /// Replaces the height sizing with a new value.
    pub fn height(mut self, size: impl Into<UiFlowSize>) -> Self {
        self.height.size = size.into();
        self
    }
    /// Replaces the width minimum clamp with a new value.
    pub fn min_width(mut self, min: impl Into<UiValue<f32>>) -> Self {
        self.width.min = Some(min.into());
        self
    }
    /// Replaces the width maximum clamp with a new value.
    pub fn max_width(mut self, max: impl Into<UiValue<f32>>) -> Self {
        self.width.max = Some(max.into());
        self
    }
    /// Replaces the height minimum clamp with a new value.
    pub fn min_height(mut self, min: impl Into<UiValue<f32>>) -> Self {
        self.height.min = Some(min.into());
        self
    }
    /// Replaces the height maximum clamp with a new value.
    pub fn max_height(mut self, max: impl Into<UiValue<f32>>) -> Self {
        self.height.max = Some(max.into());
        self
    }
    /// Replaces the gap between children with a new value.
    pub fn gap(mut self, gap: impl Into<UiValue<f32>>) -> Self {
        self.gap = gap.into();
        self
    }
    /// Replaces the padding with a new value on all sides.
    pub fn padding_all(mut self, padding: impl Into<UiValue<f32>>) -> Self {
        self.padding = UiFlowPadding::all(padding);
        self
    }
    /// Replaces the padding with a new value on the horizontal axis.
    pub fn padding_x(mut self, padding: impl Into<UiValue<f32>>) -> Self {
        self.padding = UiFlowPadding::x(padding);
        self
    }
    /// Replaces the padding with a new value on the vertical axis.
    pub fn padding_y(mut self, padding: impl Into<UiValue<f32>>) -> Self {
        self.padding = UiFlowPadding::y(padding);
        self
    }
    /// Replaces the horizontal alignment with a new value.
    pub fn align_x(mut self, align: impl Into<Align>) -> Self {
        self.align_x = align.into();
        self
    }
    /// Replaces the vertical alignment with a new value.
    pub fn align_y(mut self, align: impl Into<Align>) -> Self {
        self.align_y = align.into();
        self
    }
    /// Pack the layout type into UiLayout
    pub fn pack(self) -> UiLayout {
        UiLayout::from(self)
    }
    /// Wrap the layout type into UiLayoutType
    pub fn wrap(self) -> UiLayoutType {
        UiLayoutType::from(self)
    }
    /// Computes the layout based on given parameters. Since flow layout depends on the whole tree,
    /// this computes only the node's own box from its resolved size (relative units against the parent),
    /// aligned within the parent as a fallback for when the flow engine is not available.
    pub(crate) fn compute(&self, parent: &Rectangle2D, absolute_scale: f32, viewport_size: Vec2, font_size: f32) -> Rectangle2D {
        let mut size = Vec2::new(
            match self.width.size {
                UiFlowSize::Fit => 0.0,
                UiFlowSize::Grow => parent.size.x,
                UiFlowSize::Fixed(v) => v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 0),
            },
            match self.height.size {
                UiFlowSize::Fit => 0.0,
                UiFlowSize::Grow => parent.size.y,
                UiFlowSize::Fixed(v) => v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 1),
            },
        );
        // Apply min/max clamps
        if let Some(v) = self.width.min { size.x = size.x.max(v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 0)) }
        if let Some(v) = self.width.max { size.x = size.x.min(v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 0)) }
        if let Some(v) = self.height.min { size.y = size.y.max(v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 1)) }
        if let Some(v) = self.height.max { size.y = size.y.min(v.evaluate_axis(absolute_scale, parent.size, viewport_size, font_size, 1)) }

        // Align within parent (top-left relative)
        let pos = Vec2::new(
            (parent.size.x - size.x) * (self.align_x.0 + 1.0) / 2.0,
            (parent.size.y - size.y) * (self.align_y.0 + 1.0) / 2.0,
        );
        Rectangle2D {
            pos: -parent.size / 2.0 + pos + size / 2.0,
            size,
        }
    }
}


// #====================#
// #=== LAYOUT TYPES ===#

/// **Ui Layout Type** - Enum holding all UI layout variants.
/// The `Flow` variant is several times larger than the others because it carries
/// a full set of `UiValue` parameters; the enum stays `Copy` for API ergonomics.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum UiLayoutType {
    Boundary(UiLayoutTypeBoundary),
    Window(UiLayoutTypeWindow),
    Solid(UiLayoutTypeSolid),
    Flow(UiLayoutTypeFlow),
}
impl UiLayoutType {
    /// Computes the layout based on given parameters.
    pub(crate) fn compute(&self, parent: &Rectangle2D, absolute_scale: f32, viewport_size: Vec2, font_size: f32) -> Rectangle2D {
        match self {
            UiLayoutType::Boundary(layout) => layout.compute(parent, absolute_scale, viewport_size, font_size),
            UiLayoutType::Window(layout) => layout.compute(parent, absolute_scale, viewport_size, font_size),
            UiLayoutType::Solid(layout) => layout.compute(parent, absolute_scale, viewport_size, font_size),
            UiLayoutType::Flow(layout) => layout.compute(parent, absolute_scale, viewport_size, font_size),
        }
    }
}
impl From<UiLayoutTypeBoundary> for UiLayoutType {
    fn from(value: UiLayoutTypeBoundary) -> Self {
        UiLayoutType::Boundary(value)
    }
}
impl From<UiLayoutTypeWindow> for UiLayoutType {
    fn from(value: UiLayoutTypeWindow) -> Self {
        UiLayoutType::Window(value)
    }
}
impl From<UiLayoutTypeSolid> for UiLayoutType {
    fn from(value: UiLayoutTypeSolid) -> Self {
        UiLayoutType::Solid(value)
    }
}
impl From<UiLayoutTypeFlow> for UiLayoutType {
    fn from(value: UiLayoutTypeFlow) -> Self {
        UiLayoutType::Flow(value)
    }
}


/// **Boundary** - Declarative layout type that is defined by its top-left corner and bottom-right corner.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct UiLayoutTypeBoundary {
    /// Position of the top-left corner.
    pub pos1: UiValue<Vec2>,
    /// Position of the bottom-right corner.
    pub pos2: UiValue<Vec2>,
}
impl UiLayoutTypeBoundary {
    /// Creates new empty Boundary node layout.
    pub const fn new() -> Self {
        Self {
            pos1: UiValue::new(),
            pos2: UiValue::new(),
        }
    }
    /// Replaces the position of the top-left corner with a new value.
    pub fn pos1(mut self, pos: impl Into<UiValue<Vec2>>) -> Self {
        self.pos1 = pos.into();
        self
    }
    /// Replaces the position of the bottom-right corner with a new value.
    pub fn pos2(mut self, pos: impl Into<UiValue<Vec2>>) -> Self {
        self.pos2 = pos.into();
        self
    }
    /// Replaces the x position of the top-left corner with a new value.
    pub fn x1(mut self, x: impl Into<UiValue<f32>>) -> Self {
        self.pos1.set_x(x);
        self
    }
    /// Replaces the y position of the top-left corner with a new value.
    pub fn y1(mut self, y: impl Into<UiValue<f32>>) -> Self {
        self.pos1.set_y(y);
        self
    }
    /// Replaces the x position of the bottom-right corner with a new value.
    pub fn x2(mut self, x: impl Into<UiValue<f32>>) -> Self {
        self.pos2.set_x(x);
        self
    }
    /// Replaces the y position of the bottom-right corner with a new value.
    pub fn y2(mut self, y: impl Into<UiValue<f32>>) -> Self {
        self.pos2.set_y(y);
        self
    }
    /// Sets the position of the top-left corner to a new value.
    pub fn set_pos1(&mut self, pos: impl Into<UiValue<Vec2>>) {
        self.pos1 = pos.into();
    }
    /// Sets the position of the bottom-right corner to a new value.
    pub fn set_pos2(&mut self, pos: impl Into<UiValue<Vec2>>) {
        self.pos2 = pos.into();
    }
    /// Sets the x position of the top-left corner to a new value.
    pub fn set_x1(&mut self, x: impl Into<UiValue<f32>>) {
        self.pos1.set_x(x);
    }
    /// Sets the y position of the top-left corner to a new value.
    pub fn set_y1(&mut self, y: impl Into<UiValue<f32>>) {
        self.pos1.set_y(y);
    }
    /// Sets the x position of the bottom-right corner to a new value.
    pub fn set_x2(&mut self, x: impl Into<UiValue<f32>>) {
        self.pos2.set_x(x);
    }
    /// Sets the y position of the bottom-right corner to a new value.
    pub fn set_y2(&mut self, y: impl Into<UiValue<f32>>) {
        self.pos2.set_y(y);
    }
    /// Pack the layout type into UiLayout
    pub fn pack(self) -> UiLayout {
        UiLayout::from(self)
    }
    /// Wrap the layout type into UiLayout
    pub fn wrap(self) -> UiLayoutType {
        UiLayoutType::from(self)
    }
    /// Computes the layout based on given parameters.
    pub(crate) fn compute(&self, parent: &Rectangle2D, absolute_scale: f32, viewport_size: Vec2, font_size: f32) -> Rectangle2D {
        let pos1 = self.pos1.evaluate(Vec2::splat(absolute_scale), parent.size, viewport_size, Vec2::splat(font_size));
        let pos2 = self.pos2.evaluate(Vec2::splat(absolute_scale), parent.size, viewport_size, Vec2::splat(font_size));
        let size = pos2 - pos1;
        Rectangle2D {
            pos: -parent.size / 2.0 + pos1 + size/2.0,
            size,
        }
    }
}

/// **Window** - Declarative layout type that is defined by its size and position.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct UiLayoutTypeWindow {
    /// Position of the node.
    pub pos : UiValue<Vec2>,
    /// Decides where position should be applied at.
    pub anchor: Anchor,
    /// Size of the node layout.
    pub size: UiValue<Vec2>,
}
impl UiLayoutTypeWindow {
    /// Creates new empty Window node layout.
    pub const fn new() -> Self {
        Self {
            pos: UiValue::new(),
            anchor: Anchor::TOP_LEFT,
            size: UiValue::new(),
        }
    }
    /// Replaces the size to make the window fully cover the parent.
    pub fn full(self) -> Self {
        self.size(Rl(100.0))
    }
    /// Replaces the position with a new value.
    pub fn pos(mut self, pos: impl Into<UiValue<Vec2>>) -> Self {
        self.pos = pos.into();
        self
    }
    /// Replaces the x position with a new value.
    pub fn x(mut self, x: impl Into<UiValue<f32>>) -> Self {
        self.pos.set_x(x);
        self
    }
    /// Replaces the y position with a new value.
    pub fn y(mut self, y: impl Into<UiValue<f32>>) -> Self {
        self.pos.set_y(y);
        self
    }
    /// Replaces the size with a new value.
    pub fn size(mut self, size: impl Into<UiValue<Vec2>>) -> Self {
        self.size = size.into();
        self
    }
    /// Replaces the width with a new value.
    pub fn width(mut self, width: impl Into<UiValue<f32>>) -> Self {
        self.size.set_x(width);
        self
    }
    /// Replaces the height with a new value.
    pub fn height(mut self, height: impl Into<UiValue<f32>>) -> Self {
        self.size.set_y(height);
        self
    }
    /// Replaces the anchor with a new value.
    pub fn anchor(mut self, anchor: impl Into<Anchor>) -> Self {
        self.anchor = anchor.into();
        self
    }
    /// Sets the position to a new value.
    pub fn set_pos(&mut self, pos: impl Into<UiValue<Vec2>>){
        self.pos = pos.into();
    }
    /// Sets the x position to a new value.
    pub fn set_x(&mut self, x: impl Into<UiValue<f32>>){
        self.pos.set_x(x);
    }
    /// Sets the y position to a new value.
    pub fn set_y(&mut self, y: impl Into<UiValue<f32>>){
        self.pos.set_y(y);
    }
    /// Sets the size to a new value.
    pub fn set_size(&mut self, size: impl Into<UiValue<Vec2>>){
        self.size = size.into();
    }
    /// Sets the width to a new value.
    pub fn set_width(&mut self, width: impl Into<UiValue<f32>>){
        self.size.set_x(width);
    }
    /// Sets the height to a new value.
    pub fn set_height(&mut self, height: impl Into<UiValue<f32>>){
        self.size.set_y(height);
    }
    /// Sets the anchor to a new value.
    pub fn set_anchor(&mut self, anchor: impl Into<Anchor>){
        self.anchor = anchor.into();
    }
    /// Pack the layout type into UiLayout
    pub fn pack(self) -> UiLayout {
        UiLayout::from(self)
    }
    /// Wrap the layout type into UiLayout
    pub fn wrap(self) -> UiLayoutType {
        UiLayoutType::from(self)
    }
    /// Computes the layout based on given parameters.
    pub(crate) fn compute(&self, parent: &Rectangle2D, absolute_scale: f32, viewport_size: Vec2, font_size: f32) -> Rectangle2D {
        let pos = self.pos.evaluate(Vec2::splat(absolute_scale), parent.size, viewport_size, Vec2::splat(font_size));
        let size = self.size.evaluate(Vec2::splat(absolute_scale), parent.size, viewport_size, Vec2::splat(font_size));
        let mut anchor = self.anchor.as_vec();
        anchor.y *= -1.0;
        Rectangle2D {
            pos: -parent.size / 2.0 + pos - size * (anchor),
            size,
        }
    }
}

/// **Solid** - Declarative layout type that is defined by its width and height ratio.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct UiLayoutTypeSolid {
    /// Aspect ratio of the width and height. `1:1 == 10:10 == 100:100`.
    pub size: UiValue<Vec2>,
    /// Horizontal alignment within parent.
    pub align_x: Align,
    /// Vertical alignment within parent.
    pub align_y: Align,
    /// Specifies container scaling.
    pub scaling: Scaling,
}
impl UiLayoutTypeSolid {
    /// Creates new empty Solid node layout.
    pub fn new() -> Self {
        Self {
            size: Ab(Vec2::ONE).into(),
            align_x: Align::CENTER,
            align_y: Align::CENTER,
            scaling: Scaling::Fit,
        }
    }
    /// Replaces the size with a new value.
    pub fn size(mut self, size: impl Into<UiValue<Vec2>>) -> Self {
        self.size = size.into();
        self
    }
    /// Replaces the width with a new value.
    pub fn width(mut self, width: impl Into<UiValue<f32>>) -> Self {
        self.size.set_x(width);
        self
    }
    /// Replaces the height with a new value.
    pub fn height(mut self, height: impl Into<UiValue<f32>>) -> Self {
        self.size.set_y(height);
        self
    }
    /// Replaces the x alignment with a new value.
    pub fn align_x(mut self, align: impl Into<Align>) -> Self {
        self.align_x = align.into();
        self
    }
    /// Replaces the y alignment with a new value.
    pub fn align_y(mut self, align: impl Into<Align>) -> Self {
        self.align_y = align.into();
        self
    }
    /// Replaces the scaling mode with a new value.
    pub fn scaling(mut self, scaling: Scaling) -> Self {
        self.scaling = scaling;
        self
    }
    /// Sets the size to a new value.
    pub fn set_size(&mut self, size: impl Into<UiValue<Vec2>>) {
        self.size = size.into();
    }
    /// Sets the width to a new value.
    pub fn set_width(&mut self, width: impl Into<UiValue<f32>>) {
        self.size.set_x(width);
    }
    /// Sets the height to a new value.
    pub fn set_height(&mut self, height: impl Into<UiValue<f32>>) {
        self.size.set_y(height);
    }
    /// Sets the x alignment to a new value.
    pub fn set_align_x(&mut self, align: impl Into<Align>) {
        self.align_x = align.into();
    }
    /// Sets the y alignment to a new value.
    pub fn set_align_y(&mut self, align: impl Into<Align>) {
        self.align_y = align.into();
    }
    /// Sets the scaling mode to a new value.
    pub fn set_scaling(&mut self, scaling: Scaling) {
        self.scaling = scaling;
    }
    /// Pack the layout type into UiLayout
    pub fn pack(self) -> UiLayout {
        UiLayout::from(self)
    }
    /// Wrap the layout type into UiLayout
    pub fn wrap(self) -> UiLayoutType {
        UiLayoutType::from(self)
    }
    /// Computes the layout based on given parameters.
    pub(crate) fn compute(&self, parent: &Rectangle2D, absolute_scale: f32, viewport_size: Vec2, font_size: f32) -> Rectangle2D {

        let size = self.size.evaluate(Vec2::splat(absolute_scale), parent.size, viewport_size, Vec2::splat(font_size));

        let scale = match self.scaling {
            Scaling::HorFill => parent.size.x / size.x,
            Scaling::VerFill => parent.size.y / size.y,
            Scaling::Fit => f32::min(parent.size.x / size.x, parent.size.y / size.y),
            Scaling::Fill => f32::max(parent.size.x / size.x, parent.size.y / size.y),
        };

        let center_point = parent.size / 2.0;

        let computed_width = size.x * scale;
        let computed_height = size.y * scale;
        let computed_point = Vec2::new(center_point.x - computed_width / 2.0, center_point.y - computed_height / 2.0);

        Rectangle2D {
            pos: Vec2::new(
                computed_point.x * self.align_x.0,
                computed_point.y * self.align_y.0,
            ),
            size: (computed_width, computed_height).into(),
        }
    }
}
