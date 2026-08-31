# Embedding

Sometimes your UI is not the whole screen — you want to display a rendered camera feed inside a
node, compose multiple UIs into one output, or run Lunex alongside other rendering systems.

This chapter covers the compositional patterns Lunex supports:

- [Camera](camera.md) - Render a camera's output into a UI node. This is the bread and butter of
  embedding — for example a minimap, a picture-in-picture viewport or a 3D scene preview displayed
  inside a fixed aspect ratio panel in your UI.

- [Bevy UI](bevy-ui.md) - How Lunex coexists with Bevy's own `bevy_ui` framework in a single app.

The core idea is always the same: Lunex UI is rendered by cameras into render targets like
anything else in Bevy, and a UI node can display any image — including one that another camera
renders into.

## Where to go next

- [Camera](camera.md) - The dual-camera pipeline.
- [Bevy UI](bevy-ui.md) - Coexistence with Bevy's UI framework.
