use bevy::prelude::*;
use bevy_lunex::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, UiLunexPlugins, UiLunexDebugPlugin::<0, 0>))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn((
        // This camera will become the source for all UI paired to index 0.
        Camera2d, UiSourceCamera::<0>,

        // Ui nodes start at 0 and move + on the Z axis with each depth layer.
        // This will ensure you will see up to 1000 nested children.
        Transform::from_translation(Vec3::Z * 1000.0),
    ));

    // Spawn the UI Root.
    // The root acts as an implicit flow container (left to right) for its flow children,
    // so the sidebar and the content area below share the window width automatically.
    commands.spawn((
        Name::new("Root"),
        UiLayoutRoot::new_2d(),
        // Make the UI synchronized with camera viewport size
        UiFetchFromCamera::<0>,
    ))
    .with_children(|ui| {
        // === SIDEBAR ===
        // A fixed-width column that fills the window height and stacks its items.
        ui.spawn((
            Name::new("Sidebar"),
            UiLayout::flow()
                .direction(UiFlowDirection::TopToBottom)
                .gap(Ab(10.0))
                .padding_all(Ab(10.0))
                .width(Ab(200.0))
                .height(UiFlowSize::Grow)
                .pack(),
            Sprite::from_color(Color::srgb(0.13, 0.14, 0.18), Vec2::ONE),
        ))
        .with_children(|sidebar| {
            // A header that hugs its content
            sidebar.spawn((
                Name::new("Header"),
                UiLayout::flow()
                    .padding_all(Ab(16.0))
                    .justify(UiJustify::Center)
                    .align(Align::CENTER)
                    .pack(),
                Sprite::from_color(Color::srgb(0.85, 0.55, 0.15), Vec2::ONE),
            ))
            .with_children(|header| {
                header.spawn((
                    Name::new("Title"),
                    UiLayout::flow().width(Ab(120.0)).height(Ab(40.0)).pack(),
                    Sprite::from_color(Color::srgb(0.95, 0.65, 0.25), Vec2::ONE),
                ));
            });

            // Menu items that fill the remaining sidebar height equally
            for i in 0..5 {
                sidebar.spawn((
                    Name::new(format!("Item {i}")),
                    UiLayout::flow()
                        .padding_all(Ab(8.0))
                        .width(UiFlowSize::Grow)
                        .height(UiFlowSize::Grow)
                        .pack(),
                    Sprite::from_color(Color::srgb(0.22, 0.25, 0.32), Vec2::ONE),
                ))
                .with_children(|item| {
                    item.spawn((
                        Name::new(format!("Item {i} icon")),
                        UiLayout::flow()
                            .width(Ab(32.0))
                            .height(Ab(32.0))
                            .pack(),
                        Sprite::from_color(Color::srgb(0.35, 0.55, 0.85), Vec2::ONE),
                    ));
                });
            }
        });

        // === MAIN CONTENT ===
        // A grow-fill area taking the rest of the window width.
        ui.spawn((
            Name::new("Content"),
            UiLayout::flow()
                .direction(UiFlowDirection::TopToBottom)
                .gap(Ab(10.0))
                .padding_all(Ab(10.0))
                .width(UiFlowSize::Grow)
                .height(UiFlowSize::Grow)
                .pack(),
            Sprite::from_color(Color::srgb(0.09, 0.10, 0.12), Vec2::ONE),
        ))
        .with_children(|content| {
            // A toolbar that hugs its height
            content.spawn((
                Name::new("Toolbar"),
                UiLayout::flow()
                    .gap(Ab(8.0))
                    .height(Ab(48.0))
                    .pack(),
                Sprite::from_color(Color::srgb(0.18, 0.20, 0.25), Vec2::ONE),
            ))
            .with_children(|toolbar| {
                for i in 0..4 {
                    toolbar.spawn((
                        Name::new(format!("Button {i}")),
                        UiLayout::flow()
                            .width(Ab(80.0))
                            .height(UiFlowSize::Grow)
                            .pack(),
                        Sprite::from_color(Color::srgb(0.28, 0.45, 0.72), Vec2::ONE),
                    ));
                }
            });

            // Two panels sharing the leftover space, one twice as important (max clamps)
            content.spawn((
                Name::new("Left Panel"),
                UiLayout::flow()
                    .width(Rl(60.0))
                    .height(UiFlowSize::Grow)
                    .pack(),
                Sprite::from_color(Color::srgb(0.16, 0.30, 0.26), Vec2::ONE),
            ));
            content.spawn((
                Name::new("Right Panel"),
                UiLayout::flow()
                    .width(UiFlowSize::Grow)
                    .height(UiFlowSize::Grow)
                    .pack(),
                Sprite::from_color(Color::srgb(0.30, 0.16, 0.26), Vec2::ONE),
            ));
        });
    });
}
