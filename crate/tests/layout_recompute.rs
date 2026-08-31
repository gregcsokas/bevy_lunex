use bevy_app::{App, PostUpdate};
use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;
use bevy_math::Vec2;
use bevy_mesh::{Mesh, Mesh3d};
use bevy_transform::components::Transform;
use bevy_lunex::{
    Dimension, UiLayout, UiLayoutRoot, UiMeshPlane3d, system_layout_compute,
    system_mesh_3d_reconstruct_from_dimension,
};

#[test]
fn unchanged_layout_recompute_keeps_3d_panel_mesh() {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<Image>>();
    app.add_systems(
        PostUpdate,
        (
            system_layout_compute,
            system_mesh_3d_reconstruct_from_dimension,
        )
            .chain(),
    );

    let root = app
        .world_mut()
        .spawn((
            UiLayoutRoot::new_3d(),
            Transform::default(),
            Dimension(Vec2::new(1920.0, 1080.0)),
        ))
        .id();

    let panel = app
        .world_mut()
        .spawn((
            UiLayout::solid().size((818.0, 171.0)).pack(),
            UiMeshPlane3d,
            ChildOf(root),
        ))
        .id();

    // The first layout pass computes Dimension and constructs the panel mesh.
    app.update();
    let initial_mesh = app.world().entity(panel).get::<Mesh3d>().unwrap().0.id();

    // Text animation triggers the same kind of root touch through RecomputeUiLayout.
    // No layout inputs changed, so this pass should not reconstruct the panel mesh.
    app.world_mut()
        .entity_mut(root)
        .get_mut::<UiLayoutRoot>()
        .unwrap()
        .as_mut();
    app.update();

    let recomputed_mesh = app.world().entity(panel).get::<Mesh3d>().unwrap().0.id();
    assert_eq!(
        initial_mesh, recomputed_mesh,
        "an unchanged layout recompute replaced the fixed 3D panel mesh"
    );
}
