use bevy_app::{App, PostUpdate};
use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;
use bevy_math::{Vec2, Vec3};
use bevy_mesh::Mesh;
use bevy_transform::components::Transform;
use bevy_lunex::{
    Ab, Align, Dimension, Rl, Sp, UiFlowDirection, UiFlowSize, UiJustify, UiLayout, UiLayoutRoot,
    observer_recompute_on_hierarchy_add, observer_recompute_on_hierarchy_remove,
    observer_touch_layout_root, system_layout_compute,
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

#[test]
fn flow_sp_sizing_proportional() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let a = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Sp(3.0)).height(Ab(50.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Sp(1.0)).height(Ab(50.0)).pack(),
            ChildOf(root),
        ))
        .id();

    app.update();

    // 3 Sp vs 1 Sp split the 1000px window 3:1.
    assert_vec2(dimension_of(&app, a), Vec2::new(750.0, 50.0));
    assert_vec2(dimension_of(&app, b), Vec2::new(250.0, 50.0));
    let a_t = translation_of(&app, a);
    let b_t = translation_of(&app, b);
    // `a` spans [0, 750] (center -125), `b` spans [750, 1000] (center +375).
    assert!((a_t.x - (-125.0)).abs() <= 0.1, "a: {a_t}");
    assert!((b_t.x - 375.0).abs() <= 0.1, "b: {b_t}");
}

#[test]
fn flow_justify_space_between() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(1000.0)).height(Ab(100.0)).justify(UiJustify::SpaceBetween).pack(),
            ChildOf(root),
        ))
        .id();
    let mut children = Vec::new();
    for _ in 0..3 {
        children.push(app
            .world_mut()
            .spawn((
                UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
                ChildOf(container),
            ))
            .id());
    }

    app.update();

    // 700 leftover split between two gaps: first at 0, middle at 450, last at 900.
    let t0 = translation_of(&app, children[0]);
    let t1 = translation_of(&app, children[1]);
    let t2 = translation_of(&app, children[2]);
    assert!((t0.x - (-450.0)).abs() <= 0.1, "first: {t0}");
    assert!(t1.x.abs() <= 0.1, "middle: {t1}");
    assert!((t2.x - 450.0).abs() <= 0.1, "last: {t2}");
}

#[test]
fn flow_margin_participates_in_layout() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let parent = app
        .world_mut()
        .spawn((
            UiLayout::flow().gap(Ab(20.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let a = app
        .world_mut()
        .spawn((
            UiLayout::flow().margin_x(Ab(10.0)).width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(parent),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(parent),
        ))
        .id();

    app.update();

    // The Fit parent hugs both children including margins and gap:
    // 10 + 100 + 10 + 20 + 100 = 240 wide, 50 tall.
    assert_vec2(dimension_of(&app, parent), Vec2::new(240.0, 50.0));
    // `a` starts at its left margin, `b` sits after `a`'s right margin plus the gap.
    let a_t = translation_of(&app, a);
    let b_t = translation_of(&app, b);
    // `a` spans [10, 110] within the 240-wide parent, `b` spans [140, 240].
    assert!((a_t.x - (-60.0)).abs() <= 0.1, "a: {a_t}");
    assert!((b_t.x - 70.0).abs() <= 0.1, "b: {b_t}");
}

#[test]
fn flow_right_to_left_direction() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(1000.0)).height(Ab(100.0)).direction(UiFlowDirection::RightToLeft).pack(),
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
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();

    // The first child sits at the right edge, the second to its left.
    let first_t = translation_of(&app, first);
    let second_t = translation_of(&app, second);
    assert!((first_t.x - 450.0).abs() <= 0.1, "first: {first_t}");
    assert!((second_t.x - 350.0).abs() <= 0.1, "second: {second_t}");
}

#[test]
fn flow_align_end_pushes_children_down() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(400.0)).height(Ab(100.0)).align(Align::END).pack(),
            ChildOf(root),
        ))
        .id();
    let child = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
            ChildOf(container),
        ))
        .id();

    app.update();

    // `align: END` injects `margin_top: 1sp` - the child sits at the bottom of the container.
    // Child spans y [50, 100] top-down -> 25px below the container center in bevy's y-up space.
    let child_t = translation_of(&app, child);
    assert!((child_t.y - (-25.0)).abs() <= 0.1, "child: {child_t}");
}

#[test]
fn flow_wrap_packs_lines() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow().width(Ab(250.0)).wrapping().gap(Ab(10.0)).pack(),
            ChildOf(root),
        ))
        .id();
    let mut items = Vec::new();
    for _ in 0..3 {
        items.push(
            app.world_mut()
                .spawn((
                    UiLayout::flow().width(Ab(100.0)).height(Ab(50.0)).pack(),
                    ChildOf(container),
                ))
                .id(),
        );
    }

    app.update();

    // 100 + 10 + 100 fits, the third item wraps to the second line.
    assert_vec2(dimension_of(&app, container), Vec2::new(250.0, 110.0));
    assert_vec2(dimension_of(&app, items[2]), Vec2::new(100.0, 50.0));

    // First item spans x [0, 100] -> its center is 75px left of the container's center (125).
    let first_t = translation_of(&app, items[0]);
    assert!((first_t.x - (-75.0)).abs() <= 0.1, "first: {first_t}");
    // Second item at x [110, 210] -> center 160 -> 35px right of the container center.
    let second_t = translation_of(&app, items[1]);
    assert!((second_t.x - 35.0).abs() <= 0.1, "second: {second_t}");
    // Third item at y [60, 110] -> center 85 -> 30px below the container center (y-up flip).
    let third_t = translation_of(&app, items[2]);
    assert!((third_t.y - (-30.0)).abs() <= 0.1, "third: {third_t}");
    assert!((third_t.x - (-75.0)).abs() <= 0.1, "third: {third_t}");
}

#[test]
fn flow_grid_tracks_and_lines() {
    let mut app = test_app();
    let root = spawn_root(&mut app, Vec2::new(1000.0, 600.0));

    let container = app
        .world_mut()
        .spawn((
            UiLayout::flow()
                .width(Ab(400.0))
                .gap(Ab(10.0))
                .grid([UiFlowSize::Grow, UiFlowSize::Grow])
                .pack(),
            ChildOf(root),
        ))
        .id();
    let mut items = Vec::new();
    for _ in 0..5 {
        items.push(
            app.world_mut()
                .spawn((
                    UiLayout::flow().width(Ab(50.0)).height(Ab(50.0)).pack(),
                    ChildOf(container),
                ))
                .id(),
        );
    }

    app.update();

    // Two 195-wide grow tracks per line; the fifth item's lone track takes the whole row.
    assert_vec2(dimension_of(&app, container), Vec2::new(400.0, 170.0));
    assert_vec2(dimension_of(&app, items[0]), Vec2::new(195.0, 50.0));
    assert_vec2(dimension_of(&app, items[4]), Vec2::new(400.0, 50.0));

    // Second item starts after the first track: x [205, 400] -> center 302.5 -> 102.5px
    // right of the container center (200).
    let second_t = translation_of(&app, items[1]);
    assert!((second_t.x - 102.5).abs() <= 0.1, "second: {second_t}");
    // Third item starts the second line: x [0, 195] -> center 97.5 -> 102.5px left of the
    // container center (200); y [60, 110] -> center 85 = the container's center (85).
    let third_t = translation_of(&app, items[2]);
    assert!((third_t.y - 0.0).abs() <= 0.1, "third: {third_t}");
    assert!((third_t.x - (-102.5)).abs() <= 0.1, "third: {third_t}");
}
