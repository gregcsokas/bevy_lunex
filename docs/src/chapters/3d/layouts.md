# Layouts 3D

The layout types are exactly the same as in 2D — [Boundary, Window and Solid](../2d/layouts.md)
all work identically in 3D. What differs is how the root is sized and how depth stacking works.

## Root sizing

A 3D root is sized manually through `Dimension` in **world units** (instead of being synced to a
camera viewport):

```rust, noplayground
commands.spawn((
    UiRoot3d,
    UiLayoutRoot::new_3d(),
    // 1.0 world unit wide, 0.5 world units tall
    Dimension::from((1.0, 0.5)),
    Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
));
```

All relative units like `Rl` or `Rh` inside the tree are then resolved against this dimension,
so your UI scales naturally with the panel.

## Depth stacking

Ui-Nodes are stacked on the Z axis based on their nesting level — every child is placed in front
of its parent. You can override this with the `UiDepth` component:

- `UiDepth::Add(f32)` - Offset the node relative to its parent's depth.
- `UiDepth::Set(f32)` - Set the absolute depth, ignoring the parent.

```rust, noplayground
ui.spawn((
    // Draw this node 5 depth layers in front of its parent
    UiDepth::Add(5.0),
    UiLayout::window().pos(Rl(50.0)).anchor(Anchor::Center).pack(),
    // ...
));
```

> [!NOTE]
> In a 3D root (`new_3d()`), the depth value is scaled by `0.001`, so 1 depth layer equals
> 1 millimeter in world units. This keeps the stacking subtle but visible enough to fix
> Z-fighting between overlapping nodes.

## Fixing Z-fighting

If two nodes overlap and flicker, give one of them a small depth offset:

```rust, noplayground
ui.spawn((
    // Offset the background image behind the panel content
    UiDepth::Add(-0.1),
    UiLayout::solid().size((1920.0, 1080.0)).pack(),
    // ...
));
```

This is also useful for offsetting background images behind panels, or making overlays
always render on top.
