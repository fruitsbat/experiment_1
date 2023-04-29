use crate::{
    animation::{Frames, SpriteAnimation},
    input,
};
use bevy::prelude::*;
use bevy_easings::Lerp;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StateChanged>()
            .add_system(kick_objects)
            .add_startup_system(init_player)
            .add_systems(
                (
                    enter_state,
                    set_input_direction,
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
    pub input_direction: Vec2,
    pub velocity: Vec2,
    pub state: State,
}

impl Player {
    pub const fn gravity() -> f32 {
        -12.
    }
    pub const fn speed() -> f32 {
        3.
    }
    pub const fn acceleration() -> f32 {
        0.6
    }
    pub const fn stopping_speed() -> f32 {
        10.
    }
    pub const fn jump_height() -> f32 {
        24.
    }
}

/// event that happens when the player state changes
pub struct StateChange(State);

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Resource)]
pub enum State {
    Idle,
    Moving,
    Jumping,
    Falling,
    Landing,
    Stopping,
}

impl Default for State {
    fn default() -> Self {
        // idle is a neutral state
        Self::Idle
    }
}

pub struct StateChanged(State);

fn enter_state(mut event: EventReader<StateChanged>, mut query: Query<&mut Player>) {
    if event.is_empty() {
        return;
    }

    let state = event
        .iter()
        .collect::<Vec<&StateChanged>>()
        .first()
        .unwrap_or(&&StateChanged(State::default()))
        .0;

    for mut player in query.iter_mut() {
        if state == player.state {
            break;
        }

        info!("state changed: {:?}", state);
        // do something when entering new state
        match state {
            State::Jumping => player.velocity.y = Player::jump_height(),
            _ => (),
        };
    }

    for mut player in query.iter_mut() {
        player.state = state;
    }
}

/// determine what direction the stick is being held in
/// and store it in the player struct
fn set_input_direction(
    mut query: Query<(&ActionState<input::PlayerAction>, &mut Player)>,
    mut state_event: EventWriter<StateChanged>,
) {
    for (actions, mut player) in query.iter_mut() {
        let axis_pair = actions
            .axis_pair(input::PlayerAction::Move)
            .expect("failed to read axis pair for player!");

        match player.state {
            State::Idle => state_event.send(StateChanged(State::Moving)),
            _ => player.input_direction = axis_pair.into(),
        }
    }
}

/// what to do every frame for each state
fn move_around(
    mut query: Query<(&mut Player, &ActionState<input::PlayerAction>)>,
    time: Res<Time>,
    mut state_event: EventWriter<StateChanged>,
) {
    for (mut player, actions) in query.iter_mut() {
        match player.state {
            State::Moving => {
                if actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(StateChanged(State::Jumping));
                    return;
                }
                // if the player is not moving switch to Stopping state
                if player.input_direction.x == 0. {
                    state_event.send(StateChanged(State::Stopping));
                    return;
                }
                player.velocity.x = player.velocity.x.lerp(
                    &(player.input_direction.x * Player::speed()),
                    &(Player::acceleration() * time.delta_seconds()),
                );
            }

            State::Stopping => {
                if actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(StateChanged(State::Jumping));
                    return;
                }
                // start moving again if an input direction is pressed
                if player.input_direction.x != 0. {
                    state_event.send(StateChanged(State::Moving));
                    return;
                }
                if player.velocity.x == 0. {
                    state_event.send(StateChanged(State::Idle));
                    return;
                }
                player.velocity.x = player
                    .velocity
                    .x
                    .lerp(&0., &(Player::stopping_speed() * time.delta_seconds()))
            }

            State::Idle => {
                if actions.just_pressed(input::PlayerAction::Jump) {
                    state_event.send(StateChanged(State::Jumping));
                    return;
                }
                if player.input_direction.x != 0. {
                    state_event.send(StateChanged(State::Moving));
                    return;
                }
            }

            State::Jumping => {
                if player.velocity.y <= 0. {
                    state_event.send(StateChanged(State::Falling));
                }
            }

            _ => (),
        }
    }
}

/// apply gravity every frame
fn apply_gravity(mut players: Query<&mut Player>, time: Res<Time>) {
    for mut player in players.iter_mut() {
        player.velocity.y = player.velocity.y.lerp(
            &Player::gravity(),
            &(time.delta_seconds() * Player::acceleration()),
        );
    }
}

fn flip_sprite(mut query: Query<(&Player, &mut TextureAtlasSprite)>) {
    for (player, mut sprite) in query.iter_mut() {
        match player.state {
            State::Idle => (),
            _ => sprite.flip_x = player.input_direction.x > 0.,
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
        5,
        1,
        Some(Vec2::splat(1.)),
        None,
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn((
            Name::new("player"),
            SpriteAnimation::new(Frames::Constant(2, 0.1), 2, true),
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
