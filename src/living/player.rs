use crate::living::weapon_shooting::WeaponKind::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::prelude::*;
use bevy_tnua::{
    TnuaGhostOverwrites, TnuaObstacleRadar, TnuaToggle,
};
use bevy_tnua::control_helpers::{TnuaBlipReuseAvoidance, TnuaSimpleFallThroughPlatformsHelper};
use leafwing_input_manager::prelude::*;
use crate::character_control_systems::Dimensionality;
use crate::character_control_systems::platformer_control_scheme::{
    DemoControlScheme, DemoControlSchemeConfig,
};
use crate::character_control_systems::platformer_control_systems::CharacterMotionConfigForPlatformerDemo;
use crate::character_control_systems::player_input::PlayerAction;
use crate::levels_setup::for_2d_platformer::LayerNames;
use crate::ui::component_alteration::CommandAlteringSelectors;
use crate::living::{spawn_living, weapon_shooting, CharacterPhysicsConfig, CharacterVisualConfig, Team};

#[cfg(feature = "egui")]
use crate::ui::info::InfoSource;
#[cfg(feature = "egui")]
use crate::ui::plotting::PlotSource;
use crate::ui::components::TrackedEntity;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IsPlayer;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsPlayer>();
        app.add_systems(Startup, setup_player);
    }
}

pub fn setup_player(
    mut commands: Commands,
    mut control_scheme_config_assets: ResMut<Assets<DemoControlSchemeConfig>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let _player_entity = spawn_living(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        "Ambrosia",
        CharacterVisualConfig {
            texture_path: "Witchcraft_Sprites/Witchcraft_spr_1.png",
            tile_size: UVec2::new(24, 24),
            columns: 21,
            rows: 1,
            initial_frame_index: 0,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
        },
        CharacterPhysicsConfig {
            collider: Collider::capsule(0.5, 1.0),
            lock_rotation: true,
            ..default()
        },
        |cmd| {
            cmd.insert((IsPlayer, Team::Player));

            // Replace the old command block with this:
            cmd.insert((
                InputMap::default()
                    .with(PlayerAction::Fire, MouseButton::Left)
                    .with(PlayerAction::CyclePrev, KeyCode::BracketLeft)
                    .with(PlayerAction::CycleNext, KeyCode::BracketRight)
                    // Add your mouse scroll bindings here if using them:
                    .with(PlayerAction::CycleNext, MouseScrollDirection::UP)
                    .with(PlayerAction::CyclePrev, MouseScrollDirection::DOWN),
                ActionState::<PlayerAction>::default(),
            ));

            // `TnuaController` is Tnua's main interface with the user code. Read
            // examples/src/character_control_systems/platformer_control_systems.rs to see how
            // `TnuaController` is used in this example.
            // `TnuaConfig` holds the configuration for the Tnua controller. It can be loaded from a
            // file as an asset, but in this case we are creating it by code and injecting it to the
            // assets resource.
            cmd.insert((
                TnuaController::<DemoControlScheme>::default(),
                TnuaConfig::<DemoControlScheme>(control_scheme_config_assets.add(
                    DemoControlSchemeConfig {
                        ext: CharacterMotionConfigForPlatformerDemo {
                            dimensionality: Dimensionality::Dim2,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )),
            ));

            // The obstacle radar is used to detect obstacles around the player that the player can use
            // for environment actions (e.g., climbing). The physics backend integration plugin is
            // responsible for generating the collider in a child object. The collider is a cylinder around
            // the player character (it needs to be a little bigger than the character's collider),
            // configured so that it'll generate collision data without generating forces for the actual
            // physics simulation.
            cmd.insert(TnuaObstacleRadar::new(1.0, 3.0));

            // We use the blip reuse avoidance helper to avoid initiating actions on obstacles we've just
            // finished an action with.
            cmd.insert(TnuaBlipReuseAvoidance::<DemoControlScheme>::default());

            // An entity's Tnua behavior can be toggled individually with this component, if inserted.
            cmd.insert(TnuaToggle::default());

            cmd.insert({
                let command_altering_selectors = CommandAlteringSelectors::default()
                    // By default, Tnua uses a raycast, but this could be a problem if the character stands
                    // just past the edge while part of its body is above the platform. To solve this, we
                    // need to cast a shape - which is physics-engine specific. We set the shape using a
                    // component.
                    .with_combo(
                        "Sensor Shape",
                        1,
                        &[
                            ("Point", |mut cmd| {
                                cmd.remove::<TnuaAvian2dSensorShape>();
                            }),
                            ("Flat (underfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(
                                    0.99, 0.0,
                                )));
                            }),
                            ("Flat (exact)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(1.0, 0.0)));
                            }),
                            ("flat (overfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(
                                    1.01, 0.0,
                                )));
                            }),
                            ("Ball (underfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::circle(0.49)));
                            }),
                            ("Ball (exact)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::circle(0.5)));
                            }),
                        ],
                    )
                    .with_checkbox("Lock Tilt", false, |mut cmd, lock_tilt| {
                        // Tnua will automatically apply angular impulses/forces to fix the tilt and make
                        // the character stand upright, but it is also possible to just let the physics
                        // engine prevent rotation (other than around the Y axis, for turning)
                        if lock_tilt {
                            cmd.insert(LockedAxes::new().lock_rotation());
                        } else {
                            cmd.insert(LockedAxes::new());
                        }
                    })
                    .with_checkbox(
                        "Phase Through Collision Groups",
                        true,
                        |mut cmd, use_collision_groups| {
                            {
                                let player_layers: LayerMask = if use_collision_groups {
                                    [LayerNames::Default, LayerNames::Player].into()
                                } else {
                                    [
                                        LayerNames::Default,
                                        LayerNames::Player,
                                        LayerNames::PhaseThrough,
                                    ]
                                        .into()
                                };
                                cmd.insert(CollisionLayers::new(player_layers, player_layers));
                            }
                        },
                    );
                command_altering_selectors
            });

            // The ghost sensor is used for detecting ghost platforms - platforms configured in the physics
            // backend to not contact with the character (or detect the contact but not apply physical
            // forces based on it) and marked with the `TnuaGhostPlatform` component. These can then be
            // used as one-way platforms.
            cmd.insert(TnuaGhostOverwrites::<DemoControlScheme>::default());

            // This helper is used to operate the ghost sensor and ghost platforms and implement
            // fall-through behavior where the player can intentionally fall through a one-way platform.
            cmd.insert(TnuaSimpleFallThroughPlatformsHelper::default());

            // 1. Unpack the default weapon (Bow) from your single source of truth
            // The weapon's visual child is spawned by `apply_active_weapon`, so the
            // visual config is not needed here.
            let (weapon, kind, _weapon_visual, spawner, melee, fire_rate, _beam) =
                weapon_shooting::weapon_bundle(Bow);

            // 2. Insert the inventory and the guaranteed components
            cmd.insert((
                weapon_shooting::WeaponInventory {
                    slots: vec![
                        Bow,
                        Sword,
                        HeavyBow,
                        Beam,
                    ],
                    active: 3,
                },
                weapon,
                kind,
            ));

            // 3. Conditionally insert the Option components, just like apply_active_weapon does
            if let Some(s) = spawner { cmd.insert(s); }
            if let Some(m) = melee { cmd.insert(m); }
            if let Some(f) = fire_rate { cmd.insert(f); }

            #[cfg(feature = "egui")]
                cmd.insert((TrackedEntity("Player".to_owned()),
                PlotSource::default(),
                InfoSource::default(),
            ));
        }
    );
}