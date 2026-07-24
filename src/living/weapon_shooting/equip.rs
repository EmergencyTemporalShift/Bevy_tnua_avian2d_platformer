use std::f32::consts::{FRAC_PI_3, FRAC_PI_4};
use bevy::prelude::*;
use super::components::*;

#[derive(Component, Clone, Debug, Reflect)]
pub struct WeaponVisualConfig {
    pub texture_path: &'static str,
    pub tile_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub initial_frame_index: usize,
    pub sprite_scale: Vec3,
    pub sprite_angle_offset: f32,
}

impl Default for WeaponVisualConfig {
    fn default() -> Self {
        Self {
            texture_path: "",
            tile_size: UVec2::new(24, 24),
            columns: 10,
            rows: 1,
            initial_frame_index: 0,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
            sprite_angle_offset: 0.0,
        }
    }
}

pub(crate) fn weapon_bundle(kind: WeaponKind) -> (
    Weapon,
    WeaponKind,
    WeaponVisualConfig,
    Option<ProjectileSpawner>,
    Option<Melee>,
    Option<FireRate>,
    Option<Beam>,
) {
    match kind {
        WeaponKind::Bow => (
            Weapon { damage: 1.5 },
            kind,
            WeaponVisualConfig {
                texture_path: "weapons/bow/Bow Pack.png",
                tile_size: UVec2::new(24, 24),
                columns: 6,
                rows: 6,
                initial_frame_index: 25,
                sprite_scale: Vec3::splat(0.25),
                sprite_angle_offset: FRAC_PI_4,
            },
            Some(ProjectileSpawner {
                projectile_name: "Arrow".to_string(),
                speed: 40.0,
                max_simultaneous_projectiles: 12,
                lifetime: 5.0,
                collider_width: 1.0,
                collider_height: 0.2,
            }),
            None,
            Some(FireRate(Timer::from_seconds(0.3, TimerMode::Once))),
            None,
        ),
        WeaponKind::Sword => (
            Weapon { damage: 10.0 },
            kind,
            WeaponVisualConfig {
                texture_path: "weapons/Swords/sword_icons.png",
                tile_size: UVec2::new(16, 16),
                columns: 6,
                rows: 4,
                initial_frame_index: 12,
                sprite_scale: Vec3::splat(0.25),
                sprite_angle_offset: -FRAC_PI_4,
            },
            None,
            Some(Melee {
                arc: FRAC_PI_3,
                reach: 40.0,
                damage: 25.0,
                cooldown: Timer::from_seconds(0.4, TimerMode::Once),
            }),
            None,
            None,
        ),
        WeaponKind::HeavyBow => (
            Weapon { damage: 3.0 },
            kind,
            WeaponVisualConfig {
                texture_path: "weapons/bow/Bow Pack.png",
                tile_size: UVec2::new(24, 24),
                columns: 6,
                rows: 6,
                initial_frame_index: 31,
                sprite_scale: Vec3::splat(0.25),
                sprite_angle_offset: FRAC_PI_4,
            },
            Some(ProjectileSpawner {
                projectile_name: "Heavy Arrow".to_string(),
                speed: 40.0,
                max_simultaneous_projectiles: 3,
                lifetime: 5.0,
                collider_width: 1.2,
                collider_height: 0.35,
            }),
            None,
            Some(FireRate(Timer::from_seconds(0.6, TimerMode::Once))),
            None,
        ),
        WeaponKind::Beam => (
            Weapon { damage: 0.01 },
            kind,
            WeaponVisualConfig {
                texture_path: "weapons/beam_circle_2.png",
                tile_size: UVec2::new(24, 24),
                columns: 1,
                rows: 1,
                initial_frame_index: 0,
                sprite_scale: Vec3::splat(0.25),
                sprite_angle_offset: 0.0,
            },
            None,
            None,
            None,
            Some(Beam { range: 300.0, damage_per_second: 15.0 }),
        ),
    }
}

pub fn apply_active_weapon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    inventory_query: Query<(&WeaponInventory, &WeaponKind, Entity, &Name, Option<&Children>)>,
    weapon_sprites: Query<(), With<WeaponSprite>>,
) {
    for (inv, current_kind, entity, name, children) in inventory_query.iter() {
        let Some(new_kind) = inv.current() else { continue };

        let has_weapon_sprite = children
            .map(|c| c.iter().any(|child| weapon_sprites.contains(child)))
            .unwrap_or(false);

        if *current_kind == new_kind && has_weapon_sprite {
            continue;
        }

        if let Some(children) = children {
            for &child in children {
                if weapon_sprites.contains(child) {
                    commands.entity(child).despawn();
                }
            }
        }

        // Clean batch removal
        commands.entity(entity).remove::<(
            Weapon,
            WeaponKind,
            ProjectileSpawner,
            Hitscan,
            Melee,
            FireRate,
            Beam,
        )>();

        let (weapon, kind, weapon_visual, spawner, melee, fire_rate, beam) = weapon_bundle(new_kind);

        let texture = asset_server.load(weapon_visual.texture_path);
        let layout = TextureAtlasLayout::from_grid(
            weapon_visual.tile_size,
            weapon_visual.columns,
            weapon_visual.rows,
            None,
            None,
        );
        let atlas_layout = texture_atlas_layouts.add(layout);

        let mut ent_cmd = commands.entity(entity);
        ent_cmd.insert((weapon, kind));

        if let Some(s) = spawner { ent_cmd.insert(s); }
        if let Some(m) = melee { ent_cmd.insert(m); }
        if let Some(f) = fire_rate { ent_cmd.insert(f); }
        if let Some(b) = beam { ent_cmd.insert(b); }

        ent_cmd.with_children(|parent| {
            parent.spawn((
                WeaponSprite,
                Name::new(format!("{} Weapon", name.as_str())),
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: atlas_layout,
                        index: weapon_visual.initial_frame_index,
                    },
                ),
                WeaponOffset { offset: weapon_visual.sprite_angle_offset },
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                    .with_scale(weapon_visual.sprite_scale),
            ));
        });

        if name.to_string().ne("Player") {
            info!("Entity {:?} ({:?}) switched to {:?}", name.as_str(), entity, new_kind);
        }
    }
}

pub(crate) fn rotate_active_weapon(transform: &mut Transform, rotation: f32) {
    transform.rotation = Quat::from_rotation_z(rotation);
}