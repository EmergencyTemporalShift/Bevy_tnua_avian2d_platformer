//! Gameplay shared between the standalone binary (`cargo run`) and the
//! editor binary (`cargo editor`). Migrated from `main.rs` by `jackdaw`.
//!
//! Component, resource, and system definitions live here so the editor
//! can discover their reflected types; `main.rs` keeps only the window
//! and ambient-plugin setup.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_tnua::control_helpers::TnuaAirActionsPlugin;
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::prelude::*;

#[allow(unused_imports)]
use bevy_tnua::math::{float_consts, AsF32, Vector3};

#[cfg(feature = "pie")]
use jackdaw_runtime::prelude::*;

#[cfg(feature = "egui")]
use bevy_egui::EguiPrimaryContextPass;

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
use crate::character_control_systems::platformer_control_systems::{
    apply_platformer_controls, JustPressedCachePlugin,
};
use crate::level_mechanics::LevelMechanicsPlugin;
use crate::levels_setup::level_switching::LevelSwitchingPlugin;
use crate::levels_setup::levels_for_2d;
use crate::levels_setup::helper::helper2d;
use crate::living::enemy::EnemyPlugin;
use crate::living::player::PlayerPlugin;
use crate::living::weapon_shooting::FireWeapon;
use crate::living::LivingPlugin;
use crate::ui::DemoUi;
use crate::util::controls_other::OtherControlsPlugin;
use crate::util::game_states::{toggle_pause, GameState};
use crate::util::particles::{pause_particle_spawners, unpause_particle_spawners, ParticlePlugin};

#[cfg(feature = "egui")]
use crate::character_control_systems::info_dumping_systems::character_control_info_dumping_system;
#[cfg(feature = "egui")]
use crate::ui::plotting::plot_source_rolling_update;
#[cfg(feature = "egui")]
use crate::ui::plugin::DemoInfoUpdateSystems;
use crate::ui::systems::inspector_ui;

/// Systems that should only run when the editor's play gate is active (or always in standalone).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EditorPlaySet;

/// Systems that require both the play gate to be active AND the game to be unpaused.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActiveGameplaySet;

/// Your game's Bevy plugin. The editor links this so your components
/// show up in the inspector; the standalone binary adds it too. Gameplay
/// systems gated by [`play_gate::is_playing`] run only during Play.
#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {

    fn build(&self, app: &mut App) {
        let app_setup_configuration = AppSetupConfiguration::from_environment();

        // 1. Clone and insert the resource FIRST while the struct is whole
        app.insert_resource(app_setup_configuration.clone());

        // 2. THEN extract the schedule (partially moving it is fine now since we don't need the struct again)
        let schedule = app_setup_configuration.schedule_to_use;

        app.init_state::<GameState>();

        // --- System Sets Configuration ---
        app.configure_sets(Update, EditorPlaySet.run_if(play_gate::is_playing));
        app.configure_sets(
            Update,
            ActiveGameplaySet
                .in_set(EditorPlaySet)
                .run_if(in_state(GameState::Running)),
        );

        // --- Pause / Unpause Handling ---
        // Add your toggle system (runs in Update regardless of pause state)
        app.add_systems(Update, toggle_pause);

        // --- Physics & Tnua backend ---
        app.add_plugins(PhysicsDebugPlugin);
        match schedule {
            ScheduleToUse::Update => {
                app.add_plugins((
                    PhysicsPlugins::new(PostUpdate),
                    TnuaAvian2dPlugin::new(Update),
                    TnuaControllerPlugin::<DemoControlScheme>::new(Update),
                    TnuaAirActionsPlugin::<DemoControlSchemeAirActions>::new(Update),
                ));
            }
            ScheduleToUse::FixedUpdate => {
                app.add_plugins((
                    PhysicsPlugins::new(FixedPostUpdate),
                    TnuaAvian2dPlugin::new(FixedUpdate),
                    TnuaControllerPlugin::<DemoControlScheme>::new(FixedUpdate),
                    TnuaAirActionsPlugin::<DemoControlSchemeAirActions>::new(FixedUpdate),
                ));
            }
        }

        // --- Living and Weapon systems ---
        app.add_plugins((
            character_control_systems::WeaponPlugin,
            character_control_systems::player_input::PlayerInputPlugin,
            EnemyPlugin,
            LivingPlugin,
            OtherControlsPlugin,
        ));

        // Preserving custom extension trait if applicable
        app.add_message::<FireWeapon>();

        // --- Core Gameplay Systems ---
        app.add_systems(
            Update,
            (
                character_control_radar_visualization_system,
                apply_platformer_controls.in_set(TnuaUserControlsSystems),
            )
                .in_set(ActiveGameplaySet),
        );

        // --- UI, Rendering & Debug ---
        app.add_plugins(DemoUi::<DemoControlScheme>::default());
        app.add_systems(Startup, helper2d::setup_camera);

        #[cfg(feature = "egui")]
        {
            app.add_systems(EguiPrimaryContextPass, inspector_ui);
            app.add_systems(Update, plot_source_rolling_update.in_set(EditorPlaySet));
            app.add_systems(
                Update,
                character_control_info_dumping_system
                    .in_set(DemoInfoUpdateSystems)
                    .in_set(ActiveGameplaySet),
            );
        }

        // --- Levels & player ---
        app.add_plugins(
            LevelSwitchingPlugin::new(app_setup_configuration.level_to_load.as_ref())
                .with_levels(levels_for_2d),
        );
        app.add_plugins(PlayerPlugin);

        // --- Game mechanics ---
        app.add_plugins((LevelMechanicsPlugin, JustPressedCachePlugin));

        // --- Particles ---
        app.add_plugins((HanabiPlugin, ParticlePlugin));
        app.add_systems(OnEnter(GameState::Paused), pause_particle_spawners);
        app.add_systems(OnEnter(GameState::Running), unpause_particle_spawners);
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
