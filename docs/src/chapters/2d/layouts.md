# Layouts 2D

There are multiple layouts that you can utilize to achieve the structure you are aiming for.

## Boundary
Defined by **point1** and **point2**, it is not influenced by UI flow and is absolutely positioned.
- **pos1** - Position of the top-left corner
- **pos2** - Position of the bottom-right corner

This will make a node start at `20%` and end at `80%` on both axis from the parent node.

```rust, noplayground
UiLayout::boundary()
    .pos1(Rl(20.0))
    .pos2(Rl(80.0))
    .pack()
```

## Window
Defined by **position** and **size**, it is not influenced by UI flow and is absolutely positioned.
- **pos** - Position of the node
- **anchor** - The origin point relative to the rest of the node
- **size** - Size of the node

This will make a node centered at `x: 53%`, `y: 15%` and with size `width: 60%` and `height: 65%`.

```rust, noplayground
UiLayout::window()
    .pos(Rl((53.0, 15.0)))
    .anchor(Anchor::Center)
    .size(Rl((60.0, 65.0)))
    .pack()
```

## Solid
Defined by **size** only, it will scale to fit the parenting node. It is not influenced by UI flow.
- **size** - Aspect ratio, it doesn't matter if it is `(10, 10)` or `(100, 100)`
- **align_x** - Horizontal alignment, `-1.0 to 1.0` with `0.0` as default
- **align_y** - Vertical alignment, `-1.0 to 1.0` with `0.0` as default
- **scaling** - If the container should `fit` inside parent or `fill` the parent

> [!TIP]
> This layout is ideal for images, because it preserves aspect ratio under all costs.

Here we will set aspect ratio to the size of our imaginary texture `(881.0, 1600.0)` in pixels.
Then we can align it horizontally.

```rust, noplayground
UiLayout::solid()
    .size((881.0, 1600.0))
    .align_x(-0.74)
    .pack(),
```

## Flow
Defined by **sizing** and **child arrangement**, it participates in the ui flow of its parent.
- **direction** - Whether children are laid out left-to-right, right-to-left, top-to-bottom or bottom-to-top
- **width**/**height** - `Fit` (hug content), `Grow` (claim a share of the leftover space) or `Fixed` (explicit size, `Sp` components claim leftover)
- **gap** - Spacing between children along the direction axis
- **padding** - Spacing between the node's bounding box and its children
- **margin** - Spacing around the node within its parent's flow (`Sp` claims leftover space)
- **align** - Cross-axis alignment of children, expanded into default `Sp` margins
- **justify** - Main-axis justification of children, expanded into default `Sp` margins
- **wrap** - Pack children onto multiple lines (`Fit` cross sizing then hugs the lines)
- **grid** - Explicit main-axis tracks (`Fit`/`Fixed`/`Sp`), items fill their track

> [!TIP]
> This is the layout to reach for when building menus, sidebars, toolbars or
> any structure that should react to its content or to the window size.
> See the [Flow Layout](../flow.md) chapter for details.

```rust, noplayground
UiLayout::flow()
    .direction(UiFlowDirection::TopToBottom)
    .gap(Ab(10.0))
    .padding_all(Ab(20.0))
    .width(Ab(300.0))
    .height(UiFlowSize::Grow)
    .pack(),
```
