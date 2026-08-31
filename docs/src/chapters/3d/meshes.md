# Meshes

In 3D, Ui-Node visuals are driven by meshes. Lunex ships with a plane mesh that is automatically
reconstructed from the node's `Dimension` whenever the layout changes.

## Panel mesh

Attach `UiMeshPlane3d` and a `MeshMaterial3d` — the mesh geometry is then kept in sync with the
node's computed size for you:

```rust, noplayground
ui.spawn((
    Name::new("Panel"),
    // Set the layout of this mesh
    UiLayout::window().full().pack(),
    // Provide a material to this mesh
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("panel.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    })),
    // This component will tell Lunex to reconstruct this mesh as a plane on demand
    UiMeshPlane3d,
    // On hover change the cursor to this
    OnHoverSetCursor::new(SystemCursorIcon::Pointer),
));
```

> [!TIP]
> `unlit: true` with `AlphaMode::Blend` is the recommended material setup for UI panels —
> you usually don't want your UI affected by scene lighting.

## Coloring

The `UiColor` component (including its [state machine](../state-machine.md) blending) is written
into the material's `base_color` automatically, so plain colored panels need no texture at all:

```rust, noplayground
ui.spawn((
    UiLayout::window().full().pack(),
    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.5, 0.8))),
    UiMeshPlane3d,
));
```

## Custom meshes

You are not limited to planes. Define your own component that `require`s a `Mesh3d` and rebuild
the geometry whenever the node's `Dimension` changes:

```rust, noplayground
#[derive(Component, Default)]
#[require(Mesh3d)]
struct CustomUiNodeShape {
    top_left: Vec2,
    top_right: Vec2,
    bottom_left: Vec2,
    bottom_right: Vec2,
}

// Rebuild the mesh whenever a node resizes
fn system_construct_custom_shape(
    mut query: Query<(&Dimension, &CustomUiNodeShape, &mut Mesh3d), Changed<Dimension>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (dimension, shape, mut mesh) in &mut query {
        // ... build your mesh from `**dimension` and the corner offsets
        // mesh.0 = meshes.add(new_mesh);
    }
}

app.add_systems(PostUpdate, system_construct_custom_shape.in_set(UiSystems::PostCompute));
```

> [!NOTE]
> Schedule the rebuild system in `UiSystems::PostCompute` — that is the set where Lunex has
> already finished writing the `Dimension` of every node.

> [!IMPORTANT]
> By default, Lunex picking uses rectangles derived from the layout. If your custom mesh is
> smaller than its bounding rectangle (or you need precise hit testing), attach `NoLunexPicking`
> to the node and add Bevy's `MeshPickingPlugin` to switch to mesh raycasting for that node.
