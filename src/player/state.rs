use crate::{
    animation::{Frames, SpriteAnimation},
    player::Player,
};
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Resource, Eq)]
pub enum State {
    Idle,
    Moving,
    Jumping,
    Falling,
    Landing,
    Stopping,
}

impl State {
    pub fn sprite_animation(&self) -> SpriteAnimation {
        match self {
            Self::Idle => SpriteAnimation::new(Frames::Constant(2, 0.5), 7, true),
            Self::Moving => SpriteAnimation::new(Frames::Constant(5, 0.1), 2, true),
            Self::Jumping => SpriteAnimation::new(Frames::Constant(2, 0.24), 0, true),
            Self::Falling => SpriteAnimation::new(Frames::Constant(2, 0.24), 9, true),
            Self::Landing => SpriteAnimation::new(Frames::Constant(1, 0.5), 14, false),
            Self::Stopping => SpriteAnimation::new(Frames::Constant(1, 0.5), 15, false),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        // idle is a neutral state
        Self::Idle
    }
}

pub fn set_state(mut event: EventReader<State>, mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        player.last_state = player.state;
    }

    // no need to do anything else if
    // there is no change in state
    if event.is_empty() {
        return;
    }

    let new_state = event
        .iter()
        .map(|s| s.clone())
        .collect_vec()
        .first()
        .unwrap_or(&&State::default())
        .clone();
    for mut player in query.iter_mut() {
        player.state = new_state;
    }
}

/// what to do when exiting a certain state
pub fn exit(query: Query<&Player>) {
    for player in query.iter() {
        // no point in doing something if the state hasn't changed
        if !player.state_changed() {
            break;
        }
        match player.last_state {
            _ => (),
        }
    }
}

/// what to do when entering a certain state
pub fn enter(mut query: Query<(&mut Player, &mut SpriteAnimation)>) {
    for (mut player, mut animation) in query.iter_mut() {
        // the state has not changed, don't do anything
        if !player.state_changed() {
            break;
        }

        *animation = player.state.sprite_animation();

        info!("state changed: {:?}", player.state);
        // do something when entering new state
        match player.state {
            State::Jumping => player.velocity.y = Player::jump_height(),
            State::Falling => player.velocity.y = Player::jump_downforce(),
            _ => (),
        };
    }
}
