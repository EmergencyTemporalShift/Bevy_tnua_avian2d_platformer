use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::prelude::*;
use crate::util::game_states::GameState;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum SystemAction {
    ToggleEgui,
    Exit,
    TogglePause,
}

/// State and programmatic controls for non-player actions.
#[derive(Resource, Debug, Clone, Copy)]
pub struct OtherControls {
    egui_visible: bool,
}

impl Default for OtherControls {
    fn default() -> Self {
        Self {
            egui_visible: true,
        }
    }
}

impl OtherControls {
    #[cfg(feature = "egui")]
    pub fn is_egui_visible(&self) -> bool {
        self.egui_visible
    }
    #[cfg(not(feature = "egui"))]
    pub fn is_egui_visible(&self) -> bool {
        false
    }

    #[allow(dead_code)]
    pub fn show_egui(&mut self) {
        self.egui_visible = true;
    }
    #[allow(dead_code)]
    pub fn hide_egui(&mut self) {
        self.egui_visible = false;
    }

    pub fn toggle_egui(&mut self) {
        self.egui_visible = !self.egui_visible;
    }

    pub fn exit(exit_events: &mut MessageWriter<AppExit>) {
        exit_events.write(AppExit::Success);
    }
}

pub struct OtherControlsPlugin;

impl Plugin for OtherControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OtherControls>()
            .add_plugins(InputManagerPlugin::<SystemAction>::default())
            .add_systems(Startup, setup_system_controls)
            .add_systems(Update, handle_non_player_controls);
    }
}

// Spawns a dedicated entity to hold our global system inputs
fn setup_system_controls(mut commands: Commands) {
    commands.spawn((
        InputMap::new([
            (SystemAction::ToggleEgui, KeyCode::F1),
            (SystemAction::Exit, KeyCode::Escape),
            (SystemAction::TogglePause, KeyCode::KeyP), // Temporary, we'll get a menu eventually.
        ]),
        ActionState::<SystemAction>::default(),
    ));
}

fn handle_non_player_controls(
    // Query the component instead of requesting a resource
    action_state_q: Query<&ActionState<SystemAction>>,
    mut controls: ResMut<OtherControls>,
    mut exit_events: MessageWriter<AppExit>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    let Ok(action_state) = action_state_q.single() else {
        return;
    };

    if action_state.just_pressed(&SystemAction::ToggleEgui) {
        controls.toggle_egui();
    }

    if action_state.just_pressed(&SystemAction::Exit) {
        OtherControls::exit(&mut exit_events);
    }

    if action_state.just_pressed(&SystemAction::TogglePause) {
        match current_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
                time.pause();
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
                time.unpause();
            }
        }
    }
}