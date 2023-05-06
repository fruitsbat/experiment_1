mod state;

use crate::{
    animation::{Frames, SpriteAnimation},
    input,
};
use bevy::prelude::*;
use bevy_easings::Lerp;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use state::State;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<State>()
            .add_system(kick_objects)
            .add_startup_system(init_player)
            .add_systems(
                (
                    check_velocity,
                    state::set_state,
                    state::exit,
                    state::enter,
                    set_can_jump,
                    set_input_direction,
                    switch_states,
                    move_around,
                    apply_gravity,
                    flip_sprite,
                    move_and_slide,
                )
                    .chain(),
            );
    }
}

#[derive(Default, Component)]
pub struct Player {
    /// dpad input direction
    pub input_direction: Vec2,
    /// how much the player should move
    pub velocity: Vec2,
    /// state that the player has now
    pub state: State,
    /// state that the player had in the last tick
    pub last_state: State,
    /// touched the floor in the last frame
    pub just_on_floor: bool,
    /// can the player jump?
    pub can_jump: bool,
}

impl Player {
    pub const fn gravity() -> f32 {
        -6.
    }
    pub const fn speed() -> f32 {
        6.
    }
    pub const fn acceleration() -> f32 {
        6.
    }
    pub const fn air_acceleration() -> f32 {
        0.5
    }
    pub const fn stopping_speed() -> f32 {
        6.
    }
    pub const fn jump_height() -> f32 {
        9.
    }
    pub const fn jump_downforce() -> f32 {
        -1.
    }
    pub const fn fall_acceleration() -> f32 {
        2.
    }

    /// has the player state changed in the last frame
    pub fn state_changed(&self) -> bool {
        !(self.last_state == self.state)
    }
}

fn set_can_jump(mut query: Query<(&mut Player, &KinematicCharacterControllerOutput)>) {
    for (mut player, controller) in query.iter_mut() {
        player.can_jump = controller.grounded;
    }
}

/// determine what direction the stick is being held in
/// and store it in the player struct
fn set_input_direction(mut query: Query<(&ActionState<input::PlayerAction>, &mut Player)>) {
    for (actions, mut player) in query.iter_mut() {
        let axis_pair = actions
            .axis_pair(input::PlayerAction::Move)
            .expect("failed to read axis pair for player!");
        player.input_direction = Vec2::new(axis_pair.x().round(), axis_pair.y().round());
    }
}

fn switch_states(
    query: Query<(
        &Player,
        &ActionState<input::PlayerAction>,
        &KinematicCharacterControllerOutput,
    )>,
    mut state_event: EventWriter<State>,
) {
    for (player, actions, controller_out) in query.iter() {
        let axis_pair = actions
            .axis_pair(input::PlayerAction::Move)
            .expect("failed to get movement axis");
        match player.state {
            State::Idle => {
                // jump
                if player.can_jump && actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(State::Jumping);
                    // fall if not on ground
                } else if !controller_out.grounded {
                    state_event.send(State::Falling);
                    // player is moving
                } else if axis_pair.x() != 0. {
                    state_event.send(State::Moving);
                } else if player.velocity.x != 0. {
                    state_event.send(State::Stopping);
                }
            }

            State::Stopping => {
                if player.can_jump && actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(State::Jumping);
                } else if player.input_direction.x != 0. {
                    state_event.send(State::Moving);
                } else if (player.velocity.x == 0.) && controller_out.grounded {
                    state_event.send(State::Idle);
                }
            }

            State::Falling => {
                if controller_out.grounded {
                    state_event.send(State::Landing);
                }
            }

            State::Jumping => {
                if player.velocity.y <= 0. || actions.just_released(input::PlayerAction::Jump) {
                    state_event.send(State::Falling);
                }
            }

            State::Landing => {
                if player.input_direction.x != 0. {
                    state_event.send(State::Moving);
                }
                // TODO add timer for how long landing takes
                state_event.send(State::Idle);
            }

            State::Moving => {
                if player.can_jump && actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(State::Jumping);
                } else if player.input_direction.x == 0. {
                    state_event.send(State::Stopping);
                }
            }
        }
    }
}

/// what to do every frame for each state
fn move_around(mut query: Query<&mut Player>, time: Res<Time>) {
    for mut player in query.iter_mut() {
        match player.state {
            State::Moving => {
                player.velocity.x = player.velocity.x.lerp(
                    &(player.input_direction.x * Player::speed()),
                    &(Player::acceleration() * time.delta_seconds()),
                );
            }

            State::Falling | State::Jumping => {
                player.velocity.x = player.velocity.x.lerp(
                    &(player.input_direction.x * Player::speed()),
                    &(Player::acceleration() / 0.5 * time.delta_seconds()),
                );
            }

            State::Stopping => {
                player.velocity.x = player
                    .velocity
                    .x
                    .lerp(&0., &(Player::stopping_speed() * time.delta_seconds()));
                if player.velocity.x.abs() < 0.1 {
                    player.velocity.x = 0.;
                }
            }

            State::Idle => (),

            _ => (),
        }
    }
}

/// apply gravity every frame
fn apply_gravity(mut players: Query<&mut Player>, time: Res<Time>) {
    for mut player in players.iter_mut() {
        player.velocity.y = player.velocity.y.lerp(
            &(Player::gravity()),
            &(Player::fall_acceleration() * time.delta_seconds()),
        );
    }
}

fn flip_sprite(mut query: Query<(&Player, &mut TextureAtlasSprite)>) {
    for (player, mut sprite) in query.iter_mut() {
        match player.state {
            State::Idle => (),
            _ => {
                if player.velocity.x == 0. {
                    return;
                }
                sprite.flip_x = player.velocity.x > 0.;
            }
        }
    }
}

/// applies player velocity
fn move_and_slide(mut query: Query<(&mut KinematicCharacterController, &Player)>, time: Res<Time>) {
    for (mut character_controller, player) in query.iter_mut() {
        character_controller.translation =
            Some(player.velocity * Vec2::splat(time.delta_seconds()) * Vec2::splat(60.));
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
        16,
        2,
        Some(Vec2::splat(1.)),
        None,
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn((
            Name::new("player"),
            SpriteAnimation::new(Frames::Constant(2, 0.1), 7, true),
            Collider::capsule_y(10.0, 4.0),
            TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
            KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1000.),
                ..Default::default()
            },
            input::player_input(),
            Player::default(),
        ))
        .insert(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            ..default()
        });
}

fn check_velocity(mut query: Query<(&mut Player, &KinematicCharacterControllerOutput)>) {
    for (mut player, controller_out) in query.iter_mut() {
        if controller_out.effective_translation.x == controller_out.desired_translation.x {
            return;
        }
        if controller_out.effective_translation.x == 0. {
            player.velocity.x = 0.;
        }
    }
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
