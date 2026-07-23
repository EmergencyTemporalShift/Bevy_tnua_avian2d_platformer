use bevy::{app::AppExit, prelude::*};

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
            .add_systems(Update, handle_non_player_controls);
    }
}

fn handle_non_player_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controls: ResMut<OtherControls>,
    mut exit_events: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        controls.toggle_egui();
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        OtherControls::exit(&mut exit_events);
    }
}