use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}

pub fn toggle_pause(
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match current_state.get() {
        GameState::Running => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Running),
    }
}

pub fn pause_virtual_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

pub fn unpause_virtual_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}