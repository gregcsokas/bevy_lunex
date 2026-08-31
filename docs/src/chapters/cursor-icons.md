# Cursor Icons

Lunex provides utilities for changing the cursor icon when hovering Ui-Nodes.
There are two paths: changing the **native** window cursor, or spawning a fully custom
**software cursor** that you render yourself.

Both are part of the `CursorPlugin`, which is automatically added by `UiLunexPlugins`.

## Native cursor

Attaching `OnHoverSetCursor` to a node is all you need. While the pointer hovers the node, the
window cursor changes to the requested icon and reverts back once it leaves.

### Example

```rust, noplayground
ui.spawn((
    Name::new("Button"),
    UiLayout::window().pos(Rl((50.0, 50.0))).size((200.0, 50.0)).pack(),
    Sprite::from_image(asset_server.load("images/button.png")),
    // When hovered, it will request the cursor icon to be changed
    OnHoverSetCursor::new(SystemCursorIcon::Pointer),
));
```

`SystemCursorIcon` is re-exported from Bevy and offers all the usual variants like `Pointer`,
`Crosshair`, `Text`, `Grab` and more.

## Software cursor

If you want a fully custom cursor (for example an in-game themed cursor or a gamepad-driven one),
spawn a `SoftwareCursor` entity **as a child of a camera** with a `Sprite`:

```rust, noplayground
camera.spawn((
    SoftwareCursor::new(),
    Sprite::from_image(asset_server.load("images/cursor.png")),
));
```

While a software cursor exists, the native window cursor is hidden automatically and the software
cursor takes over — it moves with the mouse, keeps the correct position even when the camera is
zoomed, and emits picking events just like a real pointer.

> [!NOTE]
> `SoftwareCursor` automatically attaches `PointerId` and `Pickable::IGNORE`, so the cursor
> entity itself never interferes with picking.

> [!WARNING]
> The software cursor is rendered as a `Sprite`, which only renders through a `Camera2d`.
> Spawn it under your 2D UI camera, or under a dedicated 2D overlay camera when your UI is 3D.

### Texture atlas cursors

If your cursor sprite is a texture atlas, you can bind specific cursor icons to atlas indices
using `set_index`. The offset (the hotspot of the icon) is subtracted from the cursor position:

```rust, noplayground
camera.spawn((
    SoftwareCursor::new()
        .set_index(SystemCursorIcon::Default, 0, (0.0, 0.0))
        .set_index(SystemCursorIcon::Pointer, 1, (8.0, 0.0)),
    Sprite::from_image(atlas_image),
    TextureAtlas::from(atlas_layout),
));
```

`OnHoverSetCursor` works with software cursors out of the box — the requested icon is looked up
in the atlas map and the sprite switches accordingly.

## Gamepad cursor

Attaching `GamepadCursor` makes the software cursor controllable by a gamepad:

```rust, noplayground
camera.spawn((
    SoftwareCursor::new(),
    GamepadCursor::new(),
    Sprite::from_image(asset_server.load("images/cursor.png")),
));
```

- The cursor moves with the **left stick** and the speed scales with `GamepadCursor::speed`.
- The first free gamepad is bound to the first free cursor automatically.
- Button presses are translated into pointer presses:
    - `South` → primary button
    - `East` → secondary button
    - `West` → middle button

> [!TIP]
> While a gamepad cursor exists, the native window cursor is kept visible so you can still see
> which gamepad is bound to which cursor.
