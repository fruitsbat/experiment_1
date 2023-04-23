use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod input;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(input::InputPlugin)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F11)),
        )
        .add_plugin(player::PlayerPlugin)
        .add_startup_systems((setup_graphics, setup_physics))
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}

fn setup_physics(mut commands: Commands) {
    // spawn floor
    commands.spawn((
        Collider::cuboid(500.0, 50.0),
        TransformBundle::from(Transform::from_xyz(0.0, -200.0, 0.0)),
    ));

    commands.spawn((
        Collider::cuboid(50.0, 500.0),
        TransformBundle::from(Transform::from_xyz(550.0, -250.0, 0.0)),
    ));

    commands.spawn((
        Collider::cuboid(50.0, 500.0),
        TransformBundle::from(Transform::from_xyz(-550.0, -250.0, 0.0)),
    ));

    for i in 0..100 {
        commands.spawn((
            Ball,
            ExternalImpulse::default(),
            RigidBody::Dynamic,
            Dominance::group(-20),
            AdditionalMassProperties::Mass(0.1),
            Collider::ball(24.),
            TransformBundle::from(Transform::from_xyz(
                f32::sin(i as f32),
                (i as f32 * 20.) + 200.,
                0.,
            )),
        ));
    }
}

#[derive(Component)]
pub struct Ball;
