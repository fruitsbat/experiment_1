use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_easings::EasingsPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

pub mod animation;
mod camera;
mod input;
pub mod player;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                .set(ImagePlugin::default_nearest())
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EasingsPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F11)),
        )
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(animation::AnimationPlugin)
        .add_startup_systems((setup_physics, spawn_some_text))
        .run();
}

fn spawn_some_text(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load("fonts/atkinson.ttf");
    commands.spawn((Text2dBundle {
        text: Text::from_section(
            "hehe",
            TextStyle {
                font_size: 60.,
                font,
                ..default()
            },
        ),
        ..default()
    },));
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

    for i in 0..0 {
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
