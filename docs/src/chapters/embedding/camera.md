# Camera

Displaying a camera's output inside a UI node is done through a **render target**: you render a
scene into an image, then use that image as the texture of a UI node. Lunex helps with the sizing —
the `UiEmbedding` component makes the rendered texture resize with the node's `Dimension`.

The pipeline looks like this:

1. Create an empty render texture (the "canvas").
2. Spawn a **scene camera** that renders your scene into the canvas.
3. Spawn a UI node that displays the canvas as a sprite.

## The canvas

```rust, noplayground
// Create the canvas texture
let canvas = images.add(Image::clear_render_texture());
```

`Image::clear_render_texture()` creates a transparent image with the correct texture usages for
rendering into it.

## The scene camera

A regular `Camera3d` that renders into the canvas instead of the window. The `order: -1` makes it
render **before** the UI composition camera, so the canvas is always up to date:

```rust, noplayground
commands.spawn((
    Camera3d::default(),
    // Render into the canvas instead of the window
    RenderTarget::Image(canvas.clone().into()),
    Camera {
        clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        order: -1,
        ..Default::default()
    },
    Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

## The UI node

Spawn the canvas as a node with a `Sprite`, marked with `UiEmbedding` so the texture resizes
with the node:

```rust, noplayground
// Spawn the composition camera
commands.spawn((
    Camera2d,
    // Configure it as UI source
    UiSourceCamera::<0>,
    // Set the camera location to capture spawned sprites
    Transform::from_translation(Vec3::Z * 1000.0),
    // Set the render layers to only see the canvas
    RenderLayers::from_layers(&[1]),
));

// Compose the secondary canvas camera infront of composition camera
commands.spawn((
    UiLayoutRoot::new_2d(),
    UiFetchFromCamera::<0>,
)).with_children(|ui| {

    // Plane with 3D camera canvas, 16:9 aspect ratio
    ui.spawn((
        UiLayout::solid().size((16.0, 9.0)).scaling(Scaling::Fit).pack(),
        Sprite::from_image(canvas.clone()),
        UiEmbedding,
        RenderLayers::from_layers(&[1]),
    ));
});
```

> [!NOTE]
> Both the UI and its source camera live on render layer `1` here — this keeps the composition
> camera from seeing anything else in your scene. The `Solid` layout with `Scaling::Fit` keeps
> the canvas at a fixed 16:9 aspect ratio no matter the window size.

## Pixelated canvas

For a retro pixelated look, render the scene into a **low-resolution** canvas and upscale it
with nearest-neighbor sampling:

```rust, noplayground
// Use nearest-neighbor sampling so upscaled pixels stay sharp
app.add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()));
```

Then create the canvas with a fixed size instead of using `Image::clear_render_texture()`:

```rust, noplayground
fn virtual_texture(width: u32, height: u32) -> Image {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
    use bevy::asset::RenderAssetUsages;

    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}
```

> [!TIP]
> Disable anti-aliasing on the scene camera with `Msaa::Off` for that raw pixelated look. The
> full working example lives in `examples/pixelated_dualcamera` in the repository.
