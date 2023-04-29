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
                    x_movement,
                    jump,
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
}

/// event that happens when the player state changes
pub struct StateChange(State);

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum State {
    Idle,
    Moving,
    Jumping,
    Falling,
    Landing,
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

    for player in query.iter() {
        if state == player.state {
            break;
        }

        info!("state changed: {:?}", state);
        // do something when entering new state
        match state {
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

fn x_movement(mut players: Query<&mut Player>, time: Res<Time>) {
    for mut player in players.iter_mut() {
        player.velocity.x = player.velocity.x.lerp(
            &(player.input_direction.x * Player::speed()),
            &(Player::acceleration() * time.delta_seconds()),
        );
    }
}

fn jump(mut query: Query<(&ActionState<input::PlayerAction>, &mut Player)>) {
    for (actions, mut player) in query.iter_mut() {
        if actions.just_pressed(input::PlayerAction::Jump) {
            player.velocity.y += 24.;
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
