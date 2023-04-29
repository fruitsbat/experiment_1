use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Jump,
    Move,
}

pub fn player_input() -> InputManagerBundle<PlayerAction> {
    InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map: input_map(),
    }
}

fn input_map() -> InputMap<PlayerAction> {
    InputMap::default()
        // move using gamepad
        .insert(DualAxis::left_stick(), PlayerAction::Move)
        // move on keyboard
        .insert(
            VirtualDPad {
                up: KeyCode::W.into(),
                down: KeyCode::S.into(),
                left: KeyCode::A.into(),
                right: KeyCode::D.into(),
            },
            PlayerAction::Move,
        )
        .insert(KeyCode::Space, PlayerAction::Jump)
        .insert(GamepadButtonType::South, PlayerAction::Jump)
        .build()
}
