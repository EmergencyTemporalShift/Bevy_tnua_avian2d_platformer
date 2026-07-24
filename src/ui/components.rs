use bevy::prelude::{Component, Entity, Resource};
use bevy_tnua_physics_integration_layer::math::Vector3;
use crate::ui::tuning::UiTunable;

/// Custom Hierarchy component for UI - displays entity list/dropdown
#[derive(Component, Default)]
pub struct Hierarchy {
    pub entities: Vec<Entity>,
}

impl Hierarchy {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    /// Shows the hierarchy dropdown in egui
    #[cfg(feature = "egui")]
    pub fn show_in_ui(
        &mut self,
        ui: &mut egui::Ui,
        selected_entity: Option<Entity>,
    ) -> Option<Entity> {
        let mut new_selected = selected_entity;
        egui::ComboBox::from_label("Entity Hierarchy")
            .selected_text(match selected_entity {
                Some(ent) => format!("Entity {:?}", ent),
                None => "Select an entity".to_string(),
            })
            .show_ui(ui, |ui| {
                for &ent in &self.entities {
                    let label = format!("Entity {:?}", ent);
                    if ui
                        .selectable_label(selected_entity == Some(ent), &label)
                        .clicked()
                    {
                        new_selected = Some(ent);
                    }
                }
            });
        new_selected
    }
}

// Before we moved it into the `config_ext`, `CharacterMotionConfigForPlatformerDemo` was passed to
// the `DemoUi` plugin to be tuned. This replaces it so that the mechanism for passing a tunable
// component will remain, in case its ever needed again.
#[derive(Component)]
pub struct EmptyTunable;

#[cfg(feature = "egui")]
impl UiTunable for EmptyTunable {
    fn tune(&mut self, _ui: &mut egui::Ui) {}
}

#[cfg(not(feature = "egui"))]
impl UiTunable for EmptyTunable {}

#[derive(Component)]
pub struct TrackedEntity(pub String);

// NOTE: The demos are responsible for updating the physics backend
#[derive(Resource)]
pub struct DemoUiPhysicsBackendSettings {
    pub active: bool,
    pub gravity: Vector3,
}