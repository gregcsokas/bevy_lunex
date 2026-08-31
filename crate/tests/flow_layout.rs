use bevy_app::{App, PostUpdate};
use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;
use bevy_math::{Vec2, Vec3};
use bevy_mesh::Mesh;
use bevy_transform::components::Transform;
use bevy_lunex::{
    Ab, Dimension, Rl, UiFlowSize, UiLayout, UiLayoutRoot, observer_recompute_on_hierarchy_add,
    observer_recompute_on_hierarchy_remove, observer_touch_layout_root, system_layout_compute,
};

/// A minimal app with the layout compute system and the recompute observers.
fn test_app() -> App {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<Image>>();
    app.add_observer(observer_touch_layout_root);
    app.add_observer(observer_recompute_on_hierarchy_add);
    app.add_observer(observer_recompute_on_hierarchy_remove);
    app.add_systems(PostUpdate, system_layout_compute);
    app
}

/// Spawns a 2d UI root of the given size.
fn spawn_root(app: &mut App, size: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            UiLayoutRoot::new_2d(),
            Transform::default(),
            Dimension(size),
        ))
        .id()
}

fn dimension_of(app: &App, entity: Entity) -> Vec2 {
    **app.world().entity(entity).get::<Dimension>().unwrap()
}

fn translation_of(app: &App, entity: Entity) -> Vec3 {
    app.world().entity(entity).get::<Transform>().unwrap().translation
}

fn assert_vec2(actual: Vec2, expected: Vec2) {
    assert!(
        (actual.x - expected.x).abs() <= 0.1 && (actual.y - expected.y).abs() <= 0.1,
        "expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn flow_row_dimensions_and_positions() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let sidebar = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(300.0)).height(UiFlowSize::Grow).pack(),
            ChildOf(root),
        ))
        .id();
    let main = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(UiFlowSize::Grow).height(UiFlowSize::Grow).pack(),
            ChildOf(root),
        ))
        .id();

    app.update();

    // The fixed sidebar keeps its width and fills the height, the main area takes the rest.
    assert_vec2(dimension_of(&app, sidebar), Vec2::new(300.0, 600.0));
    assert_vec2(dimension_of(&app, main), Vec2::new(700.0, 600.0));

    // Positions are center-relative to the parent (y-down, flipped into bevy's y-up space).
    let sidebar_t = translation_of(&app, sidebar);
    assert!((sidebar_t.x - (-350.0)).abs() <= 0.1, "sidebar x: {sidebar_t}");
    assert!(sidebar_t.y.abs() <= 0.1, "sidebar y: {sidebar_t}");
    let main_t = translation_of(&app, main);
    assert!((main_t.x - 150.0).abs() <= 0.1, "main x: {main_t}");
    assert!(main_t.y.abs() <= 0.1, "main y: {main_t}");
}

#[test]
fn flow_tracks_root_dimension_change() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let sidebar = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(300.0)).height(UiFlowSize::Grow).pack(),
            ChildOf(root),
        ))
        .id();
    let main = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(UiFlowSize::Grow).height(UiFlowSize::Grow).pack(),
            ChildOf(root),
        ))
        .id();

    app.update();
    assert_vec2(dimension_of(&app, main), Vec2::new(700.0, 600.0));

    // Resize the root - the flow should follow on the next recompute.
    *app.world_mut().entity_mut(root).get_mut::<Dimension>().unwrap() = Dimension(Vec2::new(500.0, 400.0));
    app.update();

    assert_vec2(dimension_of(&app, sidebar), Vec2::new(300.0, 400.0));
    assert_vec2(dimension_of(&app, main), Vec2::new(200.0, 400.0));
}

#[test]
fn flow_absolute_layout_inside_flow_container() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(400.0)).height(Ab(300.0)).pack(),
            ChildOf(root),
        ))
        .id();
    // A flow child (fixed size, first in flow) and an absolute window child coexisting.
    let flow_child = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();
    let window_child = app
        .world_mut()
        .spawn((
            UiLayout::window().size(Rl(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();

    // The flow container is laid out inside the root.
    assert_vec2(dimension_of(&app, container), Vec2::new(400.0, 300.0));
    // The flow child sits at the container's top-left (translations are parent-relative).
    assert_vec2(dimension_of(&app, flow_child), Vec2::new(100.0, 50.0));
    let t = translation_of(&app, flow_child);
    assert!((t.x - (-150.0)).abs() <= 0.1, "flow child x: {t}");
    assert!((t.y - 125.0).abs() <= 0.1, "flow child y: {t}");
    // The window child resolves relative units against the flow container's box.
    assert_vec2(dimension_of(&app, window_child), Vec2::new(200.0, 150.0));
}

#[test]
fn flow_under_absolute_parent() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    // An absolute window node containing a flow subtree.
    let window = app
        .world_mut()
        .spawn((
            UiLayout::window().size((500.0, 400.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let flow = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(UiFlowSize::Grow).height(Ab(100.0)).pack(),
            ChildOf(window),
        ))
        .id();

    app.update();

    assert_vec2(dimension_of(&app, window), Vec2::new(500.0, 400.0));
    // The flow node fills the window's width and is 100 tall.
    assert_vec2(dimension_of(&app, flow), Vec2::new(500.0, 100.0));
}

#[test]
fn flow_recompute_on_child_insert() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().gap(Ab(10.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let first = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();
    // A fit container hugs its only child.
    assert_vec2(dimension_of(&app, container), Vec2::new(100.0, 50.0));
    assert_vec2(dimension_of(&app, first), Vec2::new(100.0, 50.0));

    // Inserting a new child into the flow container has to trigger a recompute.
    let second = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(80.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();
    app.update();

    assert_vec2(dimension_of(&app, container), Vec2::new(190.0, 50.0));
    assert_vec2(dimension_of(&app, second), Vec2::new(80.0, 50.0));
    // The new child is positioned after the first one, with the gap in between
    // (both translations are relative to the shared flow container).
    let first_t = translation_of(&app, first);
    let second_t = translation_of(&app, second);
    assert!((second_t.x - first_t.x - 100.0).abs() <= 0.1, "first {first_t}, second {second_t}");
}

#[test]
fn flow_recompute_on_child_despawn() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().gap(Ab(10.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let first = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();
    let second = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(80.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();
    assert_vec2(dimension_of(&app, container), Vec2::new(190.0, 50.0));

    // Despawning a child has to trigger a recompute of the container.
    app.world_mut().despawn(second);
    app.update();

    assert_vec2(dimension_of(&app, container), Vec2::new(100.0, 50.0));
    assert_vec2(dimension_of(&app, first), Vec2::new(100.0, 50.0));
}

#[test]
fn flow_nested_column_and_alignment() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let sidebar = app
        .world_mut()
        .spawn((
            UiLayout::flow()
                .width(Ab(300.0))
                .height(UiFlowSize::Grow)
                .direction(bevy_lunex::UiFlowDirection::TopToBottom)
                .gap(Ab(10.0))
                .padding_all(Ab(20.0))
                .pack(),
            ChildOf(root),
        ))
        .id();
    let item_1 = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(UiFlowSize::Grow).height(Ab(100.0)).pack(),
            ChildOf(sidebar),
        ))
        .id();
    let item_2 = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(UiFlowSize::Grow).height(Ab(100.0)).pack(),
            ChildOf(sidebar),
        ))
        .id();

    app.update();

    // Items fill the sidebar's inner width and stack vertically with gap and padding.
    assert_vec2(dimension_of(&app, item_1), Vec2::new(260.0, 100.0));
    assert_vec2(dimension_of(&app, item_2), Vec2::new(260.0, 100.0));
    let t1 = translation_of(&app, item_1);
    let t2 = translation_of(&app, item_2);
    // Vertical distance of 110 (100 height + 10 gap), second below the first (bevy y-up space).
    assert!((t1.y - t2.y - 110.0).abs() <= 0.1, "item1 {t1}, item2 {t2}");
    // The first item is horizontally centered and starts 20px under the sidebar's top edge.
    assert!(t1.x.abs() <= 0.1, "item1 x: {t1}");
    assert!((t1.y - 230.0).abs() <= 0.1, "item1 y: {t1}");
}

#[test]
fn flow_relative_sizing_in_hierarchy() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(400.0)).height(Ab(200.0)).gap(Ab(20.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let half = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Rl(50.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();
    let quarter = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Rl(50.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();

    // The percent base is the parent's inner size minus the gap: (400 - 20) = 380.
    assert_vec2(dimension_of(&app, half), Vec2::new(190.0, 50.0));
    assert_vec2(dimension_of(&app, quarter), Vec2::new(190.0, 50.0));
    // Together they exactly fill the container (190 + 20 + 190 = 400).
    let half_t = translation_of(&app, half);
    let quarter_t = translation_of(&app, quarter);
    assert!((quarter_t.x - half_t.x - 210.0).abs() <= 0.1, "half {half_t}, quarter {quarter_t}");
}
