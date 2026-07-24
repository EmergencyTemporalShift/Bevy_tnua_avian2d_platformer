use std::cmp::PartialEq;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::*;
#[cfg(feature = "egui")]
use bevy_egui::EguiContexts;
use glamour::Vector2;
use crate::living::player::IsPlayer;
use crate::living::{CharacterSprite, flip_sprite_for_direction};
use crate::living::weapon_shooting::{FireWeapon, Weapon, WeaponInventory, FireRate, WeaponKind, rotate_active_weapon, WeaponOffset, WeaponIntent};
use crate::util::game_states::GameState;
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext, WindowSpace};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Fire,
    CycleNext,
    CyclePrev,
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.add_systems(Startup, setup_ui_text);
        app.add_systems(Update, (
            player_weapon_face_mouse,
            player_fire_input,
            player_cycle_weapon,
            flip_sprite_to_mouse,
        ).run_if(in_state(GameState::Running)));

        // --- Debug Systems ---
        app.add_systems(Update, (
            update_weapon_selection_text,
            debug_player_components,
        ));
    }
}

pub fn player_fire_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    players: Query<(Entity, &Transform, Option<&FireRate>, &ActionState<PlayerAction>), (With<IsPlayer>, With<Weapon>)>,
    mut fire_messages: MessageWriter<FireWeapon>,
    #[cfg(feature = "egui")]
    mut egui_contexts: EguiContexts,
) {
    #[cfg(not(feature = "egui"))]
    let egui_wants_pointer_input = false;
    #[cfg(feature = "egui")]
    let egui_wants_pointer_input = egui_contexts
        .ctx_mut()
        .map(|ctx| ctx.egui_wants_pointer_input())
        .unwrap_or(false);

    if egui_wants_pointer_input {
        return;
    }

    let (cam, cam_tf) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(cam_tf, cursor) else {
        return;
    };
    let world_pos = ray.origin.truncate();

    for (entity, transform, fire_rate, action_state) in &players {
        // 1. Determine the intent from state transitions
        let intent = if action_state.just_pressed(&PlayerAction::Fire) {
            WeaponIntent::BeginHold
        } else if action_state.just_released(&PlayerAction::Fire) {
            WeaponIntent::ReleaseHold
        } else if action_state.pressed(&PlayerAction::Fire) {
            WeaponIntent::ContinueHold
        } else {
            continue; // Action is inactive this tick
        };

        // 2. Bypass fire rate check on release so release events aren't dropped
        if intent != WeaponIntent::ReleaseHold {
            if let Some(fr) = fire_rate {
                if !fr.0.is_finished() {
                    continue;
                }
            }
        }

        let dir = (world_pos - transform.translation.truncate()).normalize_or_zero();
        if dir != Vec2::ZERO {
            fire_messages.write(FireWeapon {
                wielder: entity,
                origin: transform.translation,
                direction: Dir2::new(dir).unwrap(),
                intent,
            });
        }
    }
}

pub fn player_weapon_face_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_q: Query<
        &GlobalTransform,
        With<IsPlayer>,
    >,
    mut weapon_q: Query<
        (&mut Transform, &WeaponOffset),
    >,
) {
    let (cam, cam_tf) = *camera;

    let player_pos = player_q
        .single()
        .expect("Incorrect number of players")
        .translation()
        .truncate()
        .to_space();

    // Guard clause for weapon transforms
    let (mut wep_tf, wep_vis) = match weapon_q.single_mut() {
        Ok(components) => components,
        Err(err) => {
            match err {
                QuerySingleError::NoEntities(_) => {
                    println!("Failed to get weapon: 0 entities matched the query.");
                }
                QuerySingleError::MultipleEntities(_) => {
                    let total_count = weapon_q.iter().count();
                    println!("Failed to get weapon: expected 1, but found {} entities.", total_count);
                }
            }
            return;
        }
    };
    // --- Guard clause, ensure mouse is on screen.
    let Some(pre_cpos) = window.cursor_position() else {
        return;
    };

    let cpos = pre_cpos.to_space::<WindowSpace>();
    let Ok(raw_world_cpos) = cam.viewport_to_world_2d(cam_tf, cpos.to_bevy()) else {
        return;
    };
    let world_cpos: Vector2<CartesianSpace> = raw_world_cpos.to_space();

    let aim_pos = world_cpos - player_pos;
    let aim_angle =
        aim_pos.y.atan2(aim_pos.x) + wep_vis.offset;

    rotate_active_weapon(&mut wep_tf, aim_angle);
}

pub fn player_cycle_weapon(
    mut inventory_q: Query<(&mut WeaponInventory, &ActionState<PlayerAction>), (With<IsPlayer>, With<Weapon>)>,
    #[cfg(feature = "egui")]
    mut egui_contexts: EguiContexts,
) {
    let Ok((mut inv, action_state)) = inventory_q.single_mut() else { return };

    #[cfg(not(feature = "egui"))]
    let egui_wants_pointer_input = false;
    #[cfg(feature = "egui")]
    let egui_wants_pointer_input = egui_contexts
        .ctx_mut()
        .map(|ctx| ctx.egui_wants_pointer_input())
        .unwrap_or(false);

    if egui_wants_pointer_input {
        return;
    }

    if action_state.just_pressed(&PlayerAction::CycleNext) {
        inv.cycle(true);
    }
    if action_state.just_pressed(&PlayerAction::CyclePrev) {
        inv.cycle(false);
    }
}

pub fn flip_sprite_to_mouse(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut player_sprite_query: Query<(&ChildOf, &mut Sprite), With<CharacterSprite>>,
    player_query: Query<&GlobalTransform, With<IsPlayer>>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (parent, mut sprite) in player_sprite_query.iter_mut() {
        if let Ok(player_transform) = player_query.get(parent.0) {
            let player_x = player_transform.translation().x;
            flip_sprite_for_direction(&mut sprite, world_pos.x - player_x);
        }
    }
}

// ─── Debug Text ─────────────────────────────────────────────

#[derive(Component)]
pub struct DebugTextTag;

pub fn setup_ui_text(mut commands: Commands) {
    commands.spawn((
        Text::new("Weapon Status: Unknown"),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        DebugTextTag,
    ));
}

pub fn update_weapon_selection_text(
    player_query: Query<Option<&WeaponKind>, With<IsPlayer>>,
    mut text_query: Query<&mut Text, With<DebugTextTag>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };

    let Ok(weapon_kind_opt) = player_query.single() else { return };

    match weapon_kind_opt {
        Some(kind) => text.0 = format!("Weapon Equipped: {:?}", kind),
        None => text.0 = "Weapon Equipped: None".to_string(),
    }
}

pub fn debug_player_components(
    player_query: Query<Entity, With<IsPlayer>>,
) {
    let count = player_query.iter().count();
    if count > 1 {
        warn!("CRITICAL: Found {} entities with IsPlayer! The shooting system might be grabbing the wrong one.", count);
    }
}