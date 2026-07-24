use std::marker::PhantomData;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{Component, IntoScheduleConfigs, SystemSet};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_tnua::TnuaScheme;
use bevy_tnua_physics_integration_layer::math::{AsF32, Float, Vector3};
use crate::ui::components::{DemoUiPhysicsBackendSettings, EmptyTunable};
use crate::ui::plotting::{make_update_plot_data_system, plot_source_rolling_update};
use crate::ui::tuning::UiTunable;
use crate::ui::framerate;
use crate::ui::systems::{apply_selectors, setup_gizmos, ui_system, update_physics_active_from_ui};

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub struct DemoInfoUpdateSystems;


pub struct DemoUi<
    S: TnuaScheme,
    C: Component<Mutability = bevy::ecs::component::Mutable> + UiTunable = EmptyTunable,
> {
    _phantom: PhantomData<(S, C)>,
}

#[cfg(feature = "egui")]
impl<S: TnuaScheme, C: Component<Mutability = bevy::ecs::component::Mutable> + UiTunable> Default
for DemoUi<S, C>
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

pub const GRAVITY_MAGNITUDE: Float = 9.81;

#[cfg(not(feature = "egui"))]
impl<S: TnuaScheme, C: Component<Mutability = bevy::ecs::component::Mutable> + UiTunable> Plugin
for DemoUi<S, C>
where
    S::Config: UiTunable,
{
    fn build(&self, app: &mut App) {
        // Keep the physics settings resource alive so systems don't panic
        app.insert_resource(DemoUiPhysicsBackendSettings {
            active: true,
            gravity: Vector3::NEG_Y * GRAVITY_MAGNITUDE,
        });

        // Keep the physics management system running
        app.add_systems(Update, update_physics_active_from_ui);
    }
}

#[cfg(feature = "egui")]
impl<S: TnuaScheme, C: Component<Mutability = bevy::ecs::component::Mutable> + UiTunable> Plugin

for DemoUi<S, C>
where
    S::Config: UiTunable,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(DemoUiPhysicsBackendSettings {
            active: true,
            gravity: Vector3::NEG_Y * GRAVITY_MAGNITUDE,
        });

        //#[cfg(feature = "egui")]
        {
            app.add_plugins(EguiPlugin::default());
            // Registers the reflection-based type data the embedded inspector uses.
            app.add_plugins(DefaultInspectorConfigPlugin);
            app.configure_sets(
                Update,
                DemoInfoUpdateSystems.after(bevy_tnua::TnuaUserControlsSystems),
            );
            app.add_systems(Update, apply_selectors);
            app.add_systems(EguiPrimaryContextPass, ui_system::<S, C>);
            app.add_systems(Update, plot_source_rolling_update);
            app.add_plugins(framerate::DemoFrameratePlugin);
            app.add_systems(
                Update,
                make_update_plot_data_system(
                    |velocity: &avian2d::dynamics::rigid_body::LinearVelocity| {
                        (**velocity).f32().extend(0.0)
                    },
                ),
            );
            app.add_systems(Startup, setup_gizmos);
        }

        app.add_systems(Update, update_physics_active_from_ui);
    }
}


