# Bevy UI

Lunex does **not** depend on or integrate with Bevy's own `bevy_ui` framework — there are no
shared components, no converters and no bridges between the two systems. This chapter explains
how the two coexist in a single app and how to combine their output.

## Two independent systems

- **Lunex UI** lives in the regular render world — nodes are entities with `Transform` that
  render through cameras as sprites, 2D meshes or 3D meshes.
- **`bevy_ui`** is its own layout and rendering stack, drawing through its UI camera.

Both can run in the same application without any conflicts, since they don't share state. This
makes Lunex a good fit for worldspace 3D UI while `bevy_ui` handles conventional screen-space
widgets — or the other way around.

## Layering with camera order

If both systems render to the same window, the camera `order` decides which one is drawn on top:

```rust, noplayground
// The 3D world with your Lunex worldspace UI
commands.spawn((
    Camera3d::default(),
    Camera { order: 0, ..Default::default() },
));

// The bevy_ui camera, drawn on top of the world
commands.spawn((
    Camera2d,
    Camera { order: 1, ..Default::default() },
    IsDefaultUiCamera,
));
```

Use `RenderLayers` on the cameras and their contents if you need finer control over what each
camera sees.

## Compositing through render targets

For tighter integration — for example displaying a Lunex UI **inside** a `bevy_ui` widget, or the
other way around — use the render target pattern from the [Camera](camera.md) chapter:

1. Render the Lunex UI with a dedicated camera into an image.
2. Use that image as a texture in the other system (`UiImage` in `bevy_ui`, or a Lunex
   `Sprite` node with `UiEmbedding`).

Since both frameworks ultimately render to images and cameras, anything you can do with a Bevy
camera composition works with either UI.

> [!NOTE]
> If you only need one UI framework, picking just one keeps things simple — Lunex positioning is
> ECS-driven, so mixing is only worth it when you genuinely need both (e.g. a `bevy_ui` settings
> menu over a Lunex worldspace HUD).
