use bevy_app::{App, PostUpdate, TaskPoolPlugin};
use bevy_asset::{AssetApp, AssetEvent, AssetEventSystems, AssetPlugin, Assets};
use bevy_color::Color;
use bevy_ecs::{message::Messages, schedule::IntoScheduleConfigs};
use bevy_lunex::{system_color, UiColor, UiState};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_sprite_render::ColorMaterial;

#[test]
fn unchanged_standard_material_is_not_modified() {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
    app.init_asset::<ColorMaterial>();
    app.init_asset::<StandardMaterial>();
    app.add_systems(PostUpdate, system_color.before(AssetEventSystems));

    let color = Color::hsla(210.0, 0.5, 0.4, 0.9);
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
    let material_id = material.id();

    app.update();
    app.world_mut()
        .resource_mut::<Messages<AssetEvent<StandardMaterial>>>()
        .drain()
        .for_each(drop);

    app.world_mut().spawn((
        MeshMaterial3d(material),
        UiColor::from(color),
        UiState::default(),
    ));
    app.update();

    let modified = app
        .world_mut()
        .resource_mut::<Messages<AssetEvent<StandardMaterial>>>()
        .drain()
        .filter(|event| event.is_modified(material_id))
        .count();

    assert_eq!(modified, 0);
}
