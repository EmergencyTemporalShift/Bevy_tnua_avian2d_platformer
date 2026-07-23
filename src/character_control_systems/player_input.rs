use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::window::PrimaryWindow;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
#[cfg(feature = "egui")]
use bevy_egui::EguiContexts;
use glamour::Vector2;
use crate::living::player::IsPlayer;
use crate::living::{CharacterSprite, flip_sprite_for_direction};
use crate::living::weapon_shooting::{FireWeapon, Weapon, WeaponInventory, FireRate, WeaponKind, rotate_active_weapon, WeaponOffset};
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext, WindowSpace};

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_text);
        app.add_systems(Update, (
            player_weapon_face_mouse,
            player_fire_input,
            mouse_wheel_switch,
            flip_sprite_to_mouse,
            update_debug_text,
            debug_player_components,
        ));
    }
}

pub fn player_fire_input(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    players: Query<(Entity, &Transform, Option<&FireRate>), (With<IsPlayer>, With<Weapon>)>,
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

    if !mouse.pressed(MouseButton::Left) || egui_wants_pointer_input {
        // Do not fire when the button is released or egui is consuming pointer input.
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

    for (entity, transform, fire_rate) in &players {
        // Respect fire rate cooldown if the weapon has one.
        if let Some(fr) = fire_rate {
            if !fr.0.is_finished() {
                continue;
            }
        }

        let dir = (world_pos - transform.translation.truncate()).normalize_or_zero();
        if dir != Vec2::ZERO {
            fire_messages.write(FireWeapon {
                wielder: entity,
                origin: transform.translation,
                direction: Dir2::new(dir).unwrap(),
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
                    // If multiple entities exist, count them accurately using .iter()
                    let total_count = weapon_q.iter().count();
                    println!("Failed to get weapon: expected 1, but found {} entities.", total_count);
                }
            }
            return;
        }
    };

    // Safely get the cursor position. This is optional when the cursor
    // is outside the window.
    let Some(pre_cpos) = window.cursor_position() else {
        return;
    };

    let cpos = Vector2::<WindowSpace>::new(pre_cpos.x, pre_cpos.y);
    //info!("cpos: {:?}", cpos.to_bevy());
    let Ok(raw_world_cpos) = cam.viewport_to_world_2d(cam_tf, cpos.to_bevy()) else {
        return;
    };
    let world_cpos: Vector2<CartesianSpace> = raw_world_cpos.to_space();

    let aim_pos = world_cpos - player_pos;
    let aim_angle =
        aim_pos.y.atan2(aim_pos.x) + wep_vis.offset;

    rotate_active_weapon(&mut wep_tf, aim_angle);
}

pub fn mouse_wheel_switch(
    mut mouse_wheel_messages: MessageReader<MouseWheel>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut inventory: Query<&mut WeaponInventory, (With<IsPlayer>, With<Weapon>)>,
    #[cfg(feature = "egui")]
    mut egui_contexts: EguiContexts,
) {
    let Ok(mut inv) = inventory.single_mut() else { return };
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
    // --- Mouse wheel ---
    for event in mouse_wheel_messages.read() {
        if event.unit == MouseScrollUnit::Line {
            if event.y > 0.0 {
                inv.cycle(true);
            } else if event.y < 0.0 {
                inv.cycle(false);
            }
        }
    }

    // --- Keyboard ---
    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::BracketLeft  => inv.cycle(false),
            KeyCode::BracketRight => inv.cycle( true),
            _ => {}
        }
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

    // Convert screen-space cursor to world-space
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (parent, mut sprite) in player_sprite_query.iter_mut() {
        if let Ok(player_transform) = player_query.get(parent.0) {
            let player_x = player_transform.translation().x;
            // Flip sprite: true = face left, false = face right
            flip_sprite_for_direction(&mut sprite, world_pos.x - player_x);
        }
    }
}

// ─── Debug Text ─────────────────────────────────────────────

#[derive(Component)]
pub struct DebugTextTag;

pub fn setup_debug_text(mut commands: Commands) {
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

pub fn update_debug_text(
    player_query: Query<Option<&WeaponKind>, With<IsPlayer>>,
    mut text_query: Query<&mut Text, With<DebugTextTag>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };

    // get_single() safely returns an error if the player hasn't spawned yet
    // or if there are multiple players, avoiding panics.
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