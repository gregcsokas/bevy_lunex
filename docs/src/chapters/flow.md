# Flow Layout

The flow layout is a dynamic, flexbox-like layout model. Unlike the absolute layouts
(`Boundary`, `Window`, `Solid`), nodes with the `UiLayout::flow()` layout
**participate in the ui flow** - they interact with their siblings and can
react to their content.

```rust
use bevy_lunex::prelude::*;

commands.spawn((
    UiLayout::flow()
        .direction(UiFlowDirection::TopToBottom)
        .gap(Ab(10.0))
        .padding_all(Ab(20.0))
        .width(UiFlowSize::Grow)
        .height(Rl(50.0))
        .align_x(Align::CENTER)
        .pack(),
));
```

## Core concepts

A flow node is described by two things:

1. **How it takes up space in its parent's flow** - the `width`/`height` sizing.
2. **How its children are arranged** - the `direction`, `gap`, `padding` and alignment.

### Sizing

Each axis of a flow node is sized with one of the [`UiFlowSize`] variants:

| Size | Behavior |
|---|---|
| `UiFlowSize::Fit` | The node hugs its content (text, image or children). |
| `UiFlowSize::Grow` | The node fills the available space, sharing it with other `Grow` siblings. |
| `UiFlowSize::Fixed(...)` | The node is sized by an explicit [`UiValue`]. |

Sizing can be further constrained with `min_width`/`max_width`/`min_height`/`max_height` clamps.

```rust
UiLayout::flow()
    .width(UiFlowSize::Grow)          // fill the available width
    .max_width(Ab(600.0))            // ...but never more than 600px
    .height(UiFlowSize::Fit)          // hug the content vertically
    .pack()
```

### Direction, gap and padding

`direction` selects whether children are laid out left-to-right (`Row`) or
top-to-bottom (`Column`). `gap` is spacing between children along that axis and
`padding` is the spacing between the node's bounding box and its children.
Both accept any [`UiValue`], so they can be expressed in `Ab`, `Rl`, `Em`, `Vw`... units.

### Alignment

`align_x`/`align_y` control where children sit inside the leftover space
(`Align::START`, `Align::CENTER`, `Align::END` or any value in between).

## Units and the parent size

Flow parameters fully support the [`UiValue`] unit system, with one caveat:
relative units (`Rl`, `Rw`, `Rh`) in a node's sizing resolve against the
**parent's inner content box** (minus padding and gaps along the flow axis).
Because a `Fit` parent's size is derived from its children, relative-sized
children cannot contribute to it - they are excluded from the content-hugging
computation and resolve once the parent's size is known.

```rust
// Two children, each 50% of the parent's inner width - they exactly fill it.
ui.spawn(UiLayout::flow().gap(Ab(10.0)).pack()).with_children(|ui| {
    ui.spawn(UiLayout::flow().width(Rl(50.0)).pack());
    ui.spawn(UiLayout::flow().width(Rl(50.0)).pack());
});
```

## Coexistence with absolute layouts

Absolute layouts (`Window`, `Boundary`, `Solid`) inside a flow container keep
their absolute positioning - they are placed inside the flow container's
rectangle as usual and do not participate in the flow. A flow node whose
parent is *not* a flow container (for example a `Window` node) is sized inside
its parent according to its own `Fit`/`Grow`/`Fixed` sizing and aligned by
`align_x`/`align_y`.

The [`UiLayoutRoot`] itself acts as an implicit flow container (left to right,
no gap or padding) for its direct flow children.

## Text flow sizing

Attach the [`UiFlowText`] component to a flow text node to opt into flow-aware
text sizing. With `wrap` enabled, the width assigned by the flow engine is fed
back into the text's `TextBounds`, causing the text to re-wrap to its box. The
layout settles within one recompute cycle as the wrapped height feeds back in.

```rust
ui.spawn((
    UiLayout::flow().width(Rl(100.0)).pack(), // any width
    UiFlowText::wrapped(),
    Text2d::new("Some wrapping paragraph"),
));
```

## State machine integration

Flow parameters blend smoothly with the [state machine](state-machine.md).
A node's active flow configuration is the weighted blend of the flow layouts
of all its active states (gap, padding, sizing values, alignment), so states
can animate layout parameters. Non-blendable fields (the `direction`, or
mismatching sizing kinds) snap to the highest-weight state's value.

The node's *kind* (flow vs. absolute) is decided by its `UiBase` layout.

## How it works

1. **Bottom-up pass** - computes content-hugging sizes and minimum sizes from the leaves up.
2. **Top-down pass** - resolves relative sizing, then redistributes space:
   on overflow, children shrink water-level (largest first, floored at their
   minimums); on leftover space, `Grow` children expand water-fill (smallest
   first, capped at their maximums).
3. **Position pass** - assigns each child's position from padding, alignment,
   gap and child sizes.

Any change that can affect the flow (layout edits, hierarchy changes, text
re-measurements, image loads) automatically triggers a recompute.
