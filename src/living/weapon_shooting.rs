use bevy::prelude::*;
use bevy::color::palettes::css;
use avian2d::prelude::*;

use std::f32::consts::{FRAC_PI_3, FRAC_PI_4};
use crate::living::{GameLayer::*, Health, Team};



// ─── Components ─────────────────────────────────────────────

/// Common to every weapon — identity + constraints
#[derive(Component)]
pub struct Weapon {
    pub max_projectiles: usize,
    pub damage: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Debug, Reflect)]
pub enum WeaponKind {
    Bow,
    Sword,
    HeavyBow,
    Beam,
    // Add more here later, e.g.: Gun, Staff, …
}

#[derive(Component, Reflect)]
pub struct WeaponInventory {
    pub slots: Vec<WeaponKind>,
    pub active: usize,
}

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

#[derive(Component)]
pub struct WeaponOffset {
    pub(crate) offset: f32,
}

impl WeaponInventory {
    /// Convenience accessor for the currently selected weapon kind
    pub fn current(&self) -> Option<WeaponKind> {
        self.slots.get(self.active).copied()
    }

    /// Scrolls forward or backward, wrapping around
    pub fn cycle(&mut self, forward: bool) {
        if self.slots.is_empty() {
            return;
        }
        if forward {
            self.active = (self.active + 1) % self.slots.len();
        } else {
            // + len avoids underflow when active == 0
            self.active = (self.active + self.slots.len() - 1) % self.slots.len();
        }
    }
}

/// Marks the child sprite entity that renders the wielder's active weapon.
#[derive(Component)]
pub struct WeaponSprite;

/// Marks that this weapon fires projectiles
#[derive(Component)]
pub struct ProjectileSpawner {
    pub speed: f32,
    pub lifetime: f32,
    pub collider_width: f32,
    pub collider_height: f32,
}

/// Marks a hitscan weapon (raycast on fire, no travel)
#[derive(Component)]
pub struct Hitscan {
    pub range: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct Beam {
    pub range: f32,
    pub damage_per_second: f32,
}

/// Marks a melee weapon
#[derive(Component)]
pub struct Melee {
    pub arc: f32,       // radians
    pub reach: f32,
    pub damage: f32,
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct Tackle {
    pub damage: f32,
    pub cooldown: Timer,
}

/// Fire-rate limiter shared by many weapons
#[derive(Component, Default)]
pub struct FireRate(pub Timer);

// Replaces the `Arrow` component
#[derive(Component)]
pub struct Projectile {
    pub fired_by: Entity,
}

#[derive(Component)]
pub struct LifetimeTimer {
    pub remaining: f32,
}

// ─── Messages ──────────────────────────────────────────────

#[derive(Message)]
pub struct FireWeapon {
    pub wielder: Entity,
    pub origin: Vec3,
    pub direction: Dir2,
}

// ─── Plugin ─────────────────────────────────────────────────

pub struct WeaponPlugin;
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FireWeapon>()
            .add_systems(Update, (
                fire_projectile_weapon,
                fire_melee_weapon,
                fire_beam_weapon,
                process_projectile_ttl,
                enable_projectile_wielder_collisions,
                tick_fire_rate,
                apply_active_weapon,
            ).chain());
    }
}

pub fn fire_projectile_weapon(
    mut fire_messages: MessageReader<FireWeapon>,
    mut wielders: Query<(&Weapon, &ProjectileSpawner, &LinearVelocity, Option<&mut FireRate>, Option<&Team>)>,
    projectiles: Query<&Projectile>,
    mut commands: Commands,
) {
    for event in fire_messages.read() {
        let Ok((weapon, spawner, vel, mut fire_rate, wielder_team)) = wielders.get_mut(event.wielder) else { continue };

        if let Some(ref mut fr) = fire_rate {
            fr.0.reset();
        }

        let active = projectiles.iter()
            .filter(|p| p.fired_by == event.wielder)
            .count();
        if active >= weapon.max_projectiles {
            continue;
        }

        let shot_vel = event.direction.as_vec2() * spawner.speed + vel.0;
        let angle = event.direction.as_vec2().y.atan2(event.direction.as_vec2().x);

        let (proj_layer, target_layers) = match wielder_team {
            Some(Team::Player) | Some(Team::Neutral) | None => (
                FriendlyProjectile,
                [World, EnemyUnit, EnemyProjectile],
            ),
            Some(Team::Enemy) => (
                EnemyProjectile,
                [World, FriendlyUnit, FriendlyProjectile],
            ),
        };

        commands.spawn((
            Projectile { fired_by: event.wielder },
            LifetimeTimer { remaining: spawner.lifetime },
            Transform::from_translation(event.origin)
                .with_rotation(Quat::from_rotation_z(angle)),
            LinearVelocity(shot_vel),
            Collider::rectangle(spawner.collider_width, spawner.collider_height),
            RigidBody::Dynamic,
            CollisionLayers::new(proj_layer, target_layers),
        ));
    }
}

fn process_projectile_ttl(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut LifetimeTimer)>,
) {
    let delta = time.delta_secs();

    for (entity, mut timer) in projectiles.iter_mut() {
        timer.remaining -= delta;

        if timer.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn enable_projectile_wielder_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform, &CollisionLayers)>,
    wielders: Query<(&Transform, Option<&Team>)>,
    //time: Res<bevy::time::Time>,
) {
    // if time.elapsed_secs() >= 20.0 {
    //     let trigger_breakpoint = true;
    // }
    for (proj_entity, projectile, proj_transform, current_layers) in projectiles.iter() {
        if let Ok((wielder_transform, w_team)) = wielders.get(projectile.fired_by) {
            let distance = proj_transform
                .translation
                .truncate()
                .distance(wielder_transform.translation.truncate());

            if distance > 2.0 {
                let (proj_layer, target_layers) = match w_team {
                    Some(Team::Player) | Some(Team::Neutral) | None => (
                        FriendlyProjectile,
                        [
                            World,
                            EnemyUnit,
                            EnemyProjectile,
                            FriendlyProjectile,
                        ],
                    ),
                    Some(Team::Enemy) => (
                        EnemyProjectile,
                        [
                            World,
                            FriendlyUnit,
                            FriendlyProjectile,
                            EnemyProjectile,
                        ],
                    ),
                };

                let new_layers = CollisionLayers::new(proj_layer, target_layers);

                if *current_layers != new_layers {
                    commands.entity(proj_entity).insert(new_layers);
                }
            }
        }
    }
}

// ─── Systems: Melee Weapons ─────────────────────────────────

pub fn fire_melee_weapon(
    mut fire_messages: MessageReader<FireWeapon>,
    mut wielders: Query<(&Weapon, &mut Melee, Option<&mut FireRate>, Option<&Team>)>,
    targets: Query<(Entity, &Transform, Option<&Team>)>,
    mut commands: Commands,
) {
    for event in fire_messages.read() {
        let Ok((_, mut melee, mut fire_rate, wielder_team)) = wielders.get_mut(event.wielder) else { continue };

        if !melee.cooldown.is_finished() {
            continue;
        }

        melee.cooldown.reset();
        if let Some(ref mut fr) = fire_rate {
            fr.0.reset();
        }

        for (target, target_tf, target_team) in &targets {
            if target == event.wielder {
                continue;
            }

            // Simple friendly fire check if teams are present
            if let (Some(w_team), Some(t_team)) = (wielder_team, target_team) {
                if w_team == t_team {
                    continue;
                }
            }

            let to_target = (target_tf.translation.truncate() - event.origin.truncate())
                .normalize_or_zero();
            let dot = to_target.dot(event.direction.as_vec2());
            let dist = target_tf.translation.truncate().distance(event.origin.truncate());

            if dot > (melee.arc / 2.0).cos() && dist <= melee.reach {
                commands.entity(target).insert(Health(-melee.damage));
            }
        }
    }
}

// ─── Systems: Beam Weapons ──────────────────────────────────

pub fn fire_beam_weapon(
    mut fire_messages: MessageReader<FireWeapon>,
    wielders: Query<&Beam>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    for event in fire_messages.read() {
        let Ok(beam) = wielders.get(event.wielder) else { continue };

        // Ensure the wielder is excluded
        let filter = SpatialQueryFilter::default().with_excluded_entities([event.wielder]);

        if let Some(hit) = spatial_query.cast_ray(
            event.origin.truncate(),
            event.direction,
            beam.range,
            true,
            &filter,
        ) {
            // warn!("Beam from {:?} hit something!", event.wielder);

            // Hit! Draw RED line.
            let end_point = event.origin.truncate() + event.direction.as_vec2() * hit.distance;

            // Use .line() instead of .line_2d() to preserve Z-depth
            gizmos.line_2d(event.origin.truncate(), end_point, css::RED);

        } else {
            // warn!("Beam from {:?} missed!", event.wielder);

            // Miss! Draw LIME line to max range.
            let end_point = event.origin.truncate() + event.direction.as_vec2() * beam.range;
            gizmos.line_2d(event.origin.truncate(), end_point, css::LIME);
        }
    }
}

// ─── Systems: Timers ────────────────────────────────────────

/// Ticks all fire-rate timers and melee cooldowns each frame
pub fn tick_fire_rate(
    mut fire_rates: Query<&mut FireRate>,
    mut melee_cooldowns: Query<&mut Melee>,
    time: Res<Time>,
) {
    let delta = time.delta();

    for mut fr in &mut fire_rates {
        fr.0.tick(delta);
    }

    for mut melee in &mut melee_cooldowns {
        melee.cooldown.tick(delta);
    }
}

// ─── Equip Helpers ──────────────────────────────────────────
// Call these from whatever system triggers weapon swaps (key press, menu, etc.)

/// A bundle representing one fully-equipped weapon on the player.
/// Not a `#[derive(Bundle)]` because we need runtime logic to pick fields.
pub(crate) fn weapon_bundle(kind: WeaponKind) -> (
    Weapon,
    WeaponKind,
    WeaponVisualConfig,
    Option<ProjectileSpawner>,
    Option<Melee>,
    Option<FireRate>,
    Option<Beam>
) {
    match kind {
        WeaponKind::Bow => (
            Weapon { max_projectiles: 12, damage: 1.5 },
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
                speed: 40.0,
                lifetime: 5.0,
                collider_width: 1.0,
                collider_height: 0.2,
            }),
            None,
            Some(FireRate(Timer::from_seconds(0.3, TimerMode::Once))),
            None,
        ),
        WeaponKind::Sword => (
            Weapon { max_projectiles: 0, damage: 10.0 },
            kind,
            WeaponVisualConfig {
                texture_path: "weapons/Swords/sword_icons.png",
                tile_size: UVec2::new(16, 16),
                columns: 6, // Single sprite configuration
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
            Weapon { max_projectiles: 2, damage: 3.0 },
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
                speed: 40.0,
                lifetime: 5.0,
                collider_width: 1.2,
                collider_height: 0.35,
            }),
            None,
            Some(FireRate(Timer::from_seconds(0.6, TimerMode::Once))),
            None
        ),
        WeaponKind::Beam => (
            Weapon { max_projectiles: 1, damage: 0.01 },
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

        // Does the wielder already carry a visual for its active weapon?
        let has_weapon_sprite = children
            .map(|c| c.iter().any(|child| weapon_sprites.contains(child)))
            .unwrap_or(false);

        // Nothing to do if the equipped kind matches and its sprite already exists.
        // (The sprite check also covers the very first frame, where the weapon was
        // inserted at spawn time, but no visual child has been created yet.)
        if *current_kind == new_kind && has_weapon_sprite {
            continue;
        }

        // Remove the previous weapon's visual child before re-equipping.
        if let Some(children) = children {
            for &child in children {
                if weapon_sprites.contains(child) {
                    commands.entity(child).despawn();
                }
            }
        }

        // Strip every weapon-related component, then re-insert fresh ones
        commands.entity(entity)
            .remove::<Weapon>()
            .remove::<WeaponKind>()
            .remove::<ProjectileSpawner>()
            .remove::<Hitscan>()
            .remove::<Melee>()
            .remove::<FireRate>()
            .remove::<Beam>();

        let (weapon, kind, weapon_visual, spawner, melee, fire_rate, beam) = weapon_bundle(new_kind);

        // Build the weapon sprite's texture atlas from its visual config.
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

        // Spawn the weapon's visual as a child so it renders attached to the wielder.
        // The child's transform is local to the wielder; a small +Z keeps the weapon
        // drawn in front of the character sprite.
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
                Visibility::default(),
            ));
        });

        // Player switching works fine, don't need to log it atm.
        if name.to_string().ne("Player") {
            info!("Entity {:?} ({:?}) switched to {:?}", name.as_str(), entity, new_kind);
        }
    }
}

pub(crate) fn rotate_active_weapon(transform: &mut Transform, rotation: f32) {
    transform.rotation = Quat::from_rotation_z(rotation);
}