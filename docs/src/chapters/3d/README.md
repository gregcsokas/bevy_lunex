# 3D Usage

3D UI works the same as 2D, with a few differences in the setup. There are two patterns:

- **Worldspace UI** - Panels floating in your scene, like holographic screens.
- **Camera-attached HUD** - UI parented to the camera, like a cockpit interface.

Both use `UiLayoutRoot::new_3d()`, which scales the depth stacking by `0.001` so that the
typical "1 pixel per depth layer" of 2D becomes sane world units.

> [!IMPORTANT]
> A worldspace root is sized manually with the `Dimension` component in world units. A HUD root,
> on the other hand, needs `UiFetchFromCamera` paired with `UiSourceCamera` on the camera to keep
> itself synchronized with the camera's viewport — the same mechanism as in 2D.

## Worldspace UI

Spawn the root standalone and position it in the world like any other entity:

```rust, noplayground
// Spawn the floating UI panel
commands.spawn((
    // Required to mark this as 3D
    UiRoot3d,
    // Use this constructor to init 3D settings
    UiLayoutRoot::new_3d(),
    // Provide the size in world units instead of camera
    Dimension::from((0.818, 0.965)),
    // The location of the UI panel
    Transform::from_translation(Vec3::new(-1.5, 1.0, 0.0)),
)).with_children(|ui| {
    // ... spawn your UI here
});
```

## Camera-attached HUD

Spawn the root as a **child of a `Camera3d`** — it then follows the camera around, using a local
transform for the offset:

```rust, noplayground
commands.spawn((
    Camera3d::default(),
    // Mark the camera as the UI source for index 0
    UiSourceCamera::<0>,
    // ...
)).with_children(|camera| {

    // Spawn the HUD UI panel
    camera.spawn((
        // Required to mark this as 3D
        UiRoot3d,
        // Use this constructor to init 3D settings
        UiLayoutRoot::new_3d(),
        // Keep the UI synchronized with camera viewport size
        UiFetchFromCamera::<0>,
        // The location of the UI panel relative to the camera
        Transform::from_xyz(-0.25, 0.0, -0.8)
            .with_rotation(Quat::from_rotation_y(40.0_f32.to_radians())),
    )).with_children(|ui| {
        // ... spawn your UI here
    });
});
```

> [!NOTE]
> When the camera uses an **orthographic** projection, the fetched viewport size is multiplied
> by the projection scale, converting it into world units. For panels with a fixed world size,
> you can instead provide a fixed `Dimension::from((0.5, 0.2))` — that is what the `hud` example
> does.

## Node visuals

Since everything lives in the 3D world, nodes render through meshes:

- `UiMeshPlane3d` + `MeshMaterial3d<StandardMaterial>` - Panels, see [Meshes](meshes.md).
- `Text3d` - Text rendered via `bevy_rich_text3d`, see [Text](text.md).

> [!WARNING]
> `Sprite` does **not** work in 3D — sprites only render through a `Camera2d`. In 3D UIs, use
> meshes (`UiMeshPlane3d`) or `Text3d` for node visuals instead.

> [!TIP]
> The `UiRoot3d` marker is propagated down the whole hierarchy automatically. You can check any
> entity for it to tell whether it belongs to a 3D UI without walking up to the root.

## Where to go next

- [Layouts](layouts.md) - Depth stacking and 3D layout specifics.
- [Meshes](meshes.md) - Mesh-driven node visuals.
- [Text](text.md) - 3D text rendering.
