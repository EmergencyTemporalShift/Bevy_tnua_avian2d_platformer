//! Gameplay shared between the standalone binary (`cargo run`) and the
//! editor binary (`cargo editor`). Migrated from `main.rs` by `jackdaw`.
//!
//! Component, resource, and system definitions live here so the editor
//! can discover their reflected types; `main.rs` keeps only the window
//! and ambient-plugin setup.

use bevy::prelude::*;
#[cfg(feature = "pie")]
use jackdaw_runtime::prelude::*;
use avian2d::prelude::*;
use bevy_tnua::control_helpers::TnuaAirActionsPlugin;
//use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};
#[cfg(feature = "egui")]
use {
    bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext},
    bevy_inspector_egui,
};
#[allow(unused_imports)]
use bevy_tnua::math::{AsF32, Vector3, float_consts};
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::prelude::*;
use levels_setup::helper::helper2d;
use ui::DemoUi;

use bevy_hanabi::prelude::*;

// --- Internal modules ---
pub mod app_setup_options;
pub mod character_animating_systems;
pub mod character_control_systems;
pub mod level_mechanics;
pub mod levels_setup;
pub mod living;
pub mod ui;
pub mod util;

// --- Crate-level imports ---
use crate::app_setup_options::{AppSetupConfiguration, ScheduleToUse};
use crate::character_control_systems::info_dumping_systems::character_control_radar_visualization_system;
use crate::character_control_systems::platformer_control_scheme::{
    DemoControlScheme, DemoControlSchemeAirActions,
};
use crate::character_control_systems::platformer_control_systems::{JustPressedCachePlugin,
                                                                   apply_platformer_controls,
};
use crate::level_mechanics::LevelMechanicsPlugin;
use crate::levels_setup::level_switching::LevelSwitchingPlugin;
use crate::levels_setup::levels_for_2d;
use crate::living::LivingPlugin;
use crate::living::enemy::EnemyPlugin;
use crate::living::player::PlayerPlugin;
use crate::living::weapon_shooting::FireWeapon;
use crate::util::controls_other::OtherControlsPlugin;

#[cfg(feature = "egui")]
use crate::character_control_systems::info_dumping_systems::character_control_info_dumping_system;
#[cfg(feature = "egui")]
use crate::ui::DemoInfoUpdateSystems;
#[cfg(feature = "egui")]
use crate::ui::plotting::plot_source_rolling_update;

/// Your game's Bevy plugin. The editor links this so your components
/// show up in the inspector; the standalone binary adds it too. Gameplay
/// systems gated by [`play_gate::is_playing`] run only during Play.
#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let app_setup_configuration = AppSetupConfiguration::from_environment();
        app.insert_resource(app_setup_configuration.clone());

        let schedule = app_setup_configuration.schedule_to_use;

        // --- Physics & Tnua backend (schedule-dependent) ---
        app.add_plugins(PhysicsDebugPlugin);
        match schedule {
            ScheduleToUse::Update => {
                app.add_plugins(PhysicsPlugins::new(PostUpdate));
                app.add_plugins(TnuaAvian2dPlugin::new(Update));
            }
            ScheduleToUse::FixedUpdate => {
                app.add_plugins(PhysicsPlugins::new(FixedPostUpdate));
                app.add_plugins(TnuaAvian2dPlugin::new(FixedUpdate));
            }
        }

        // --- Tnua controller & air actions (schedule-dependent) ---
        match schedule {
            ScheduleToUse::Update => {
                app.add_plugins(TnuaControllerPlugin::<DemoControlScheme>::new(Update));
                app.add_plugins(TnuaAirActionsPlugin::<DemoControlSchemeAirActions>::new(Update));
            }
            ScheduleToUse::FixedUpdate => {
                app.add_plugins(TnuaControllerPlugin::<DemoControlScheme>::new(FixedUpdate));
                app.add_plugins(TnuaAirActionsPlugin::<DemoControlSchemeAirActions>::new(FixedUpdate));
            }
        }

        // --- Living and Weapon systems ---
        app.add_plugins(character_control_systems::WeaponPlugin);
        app.add_message::<FireWeapon>();
        app.add_plugins(character_control_systems::player_input::PlayerInputPlugin);
        app.add_plugins((EnemyPlugin, LivingPlugin));
        app.add_plugins(OtherControlsPlugin);

        // --- Debug / info dumping (egui-gated) ---
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            character_control_info_dumping_system
                .in_set(DemoInfoUpdateSystems)
                .run_if(play_gate::is_playing),
        );

        app.add_systems(
            Update,
            character_control_radar_visualization_system.run_if(play_gate::is_playing),
        );

        // --- UI & rendering ---
        #[cfg(feature = "egui")]
        app.add_systems(Update, plot_source_rolling_update.run_if(play_gate::is_playing));

        app.add_plugins(DemoUi::<DemoControlScheme>::default());

        #[cfg(feature = "egui")]
        app.add_systems(EguiPrimaryContextPass, inspector_ui);

        app.add_systems(Startup, helper2d::setup_camera);

        // --- Levels & player ---
        app.add_plugins(
            LevelSwitchingPlugin::new(app_setup_configuration.level_to_load.as_ref())
                .with_levels(levels_for_2d),
        );
        app.add_plugins(PlayerPlugin);
        app.add_systems(
            Update,
            apply_platformer_controls
                .in_set(TnuaUserControlsSystems)
                .run_if(play_gate::is_playing),
        );

        // --- Game mechanics ---
        app.add_plugins((LevelMechanicsPlugin, JustPressedCachePlugin));
        // --- Particles ---
        app.add_plugins(HanabiPlugin);
    }
}

/// Bridges the editor's `PlayState` to gameplay without forcing a `jackdaw` dep
/// in standalone builds. Always `true` without the `editor` feature; gates on
/// `PlayState::Playing` when the editor is compiled in.
pub mod play_gate {
    #[cfg(feature = "editor")]
    pub fn is_playing(
        state: bevy::prelude::Res<bevy::state::State<jackdaw::prelude::PlayState>>,
    ) -> bool {
        matches!(*state.get(), jackdaw::prelude::PlayState::Playing)
    }

    #[cfg(not(feature = "editor"))]
    pub fn is_playing() -> bool {
        true
    }
}

#[cfg(feature = "egui")]
pub fn inspector_ui(world: &mut World) {
    let mut context_query =
        world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();

    let Ok(egui_context) = context_query.single_mut(world) else {
        return;
    };

    // Clone the component handle so the query no longer borrows the world
    // while the inspector itself accesses it.
    let mut egui_context = egui_context.clone();

    egui::Window::new("Inspector").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                    world, ui,
                );
            });
        });
    });
}


