use crate::{
    animation::{Frames, SpriteAnimation},
    input,
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((update_system, kick_objects))
            .add_startup_system(init_player);
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn update_system(
    mut controllers: Query<(
        &mut KinematicCharacterController,
        &ActionState<input::PlayerAction>,
    )>,
    time: Res<Time>,
) {
    for (mut controller, actions) in controllers.iter_mut() {
        let axis_pair = actions
            .clamped_axis_pair(input::PlayerAction::Move)
            .expect("failed to read left axis from player");
        controller.translation = Some(Vec2::new(
            time.delta_seconds() * axis_pair.x() * 200.,
            if actions.just_pressed(input::PlayerAction::Jump) {
                // jump button pressed
                1.0
            } else {
                -1.0
            } * time.delta_seconds()
                * 200.,
        ));
    }
}

fn init_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("sprites/player.png");
    let atlas = TextureAtlas::from_grid(
        image,
        Vec2::new(32., 32.),
        5,
        1,
        None,
        Some(Vec2::new(0., 32.)),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn((
            Name::new("player"),
            SpriteAnimation::new(Frames::Constant(5, 0.06), 2, true),
            Collider::capsule_y(10.0, 4.0),
            TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
            KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1000.),
                ..Default::default()
            },
            input::player_input(),
            Player,
        ))
        .insert(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            ..default()
        });
}

fn kick_objects(
    outputs: Query<&KinematicCharacterControllerOutput>,
    mut balls: Query<(Entity, &mut ExternalImpulse), With<crate::Ball>>,
) {
    for output in outputs.iter() {
        for collision in output.collisions.iter() {
            for (ball, mut impulse) in balls.iter_mut() {
                if ball == collision.entity {
                    impulse.impulse = collision.toi.normal1 * Vec2::new(-200., -200.);
                }
            }
        }
    }
}
