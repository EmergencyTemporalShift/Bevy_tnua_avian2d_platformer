use avian2d::prelude::*;
use bevy::prelude::*;

use super::{flip_sprite_for_direction, spawn_living, CharacterVisualConfig, CharacterPhysicsConfig, CharacterSprite, Health, Team, GameLayer};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IsEnemy;

/// Patrol behavior - unique to enemies
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Patrol {
    pub speed: f32,
    pub direction: f32, // 1.0 = right, -1.0 = left
    pub edge_timer: Timer,
}

impl Default for Patrol {
    fn default() -> Self {
        Self {
            speed: 2.0,
            direction: -1.0,
            edge_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsEnemy>()
            .register_type::<Patrol>();
        app.add_systems(Startup, setup_enemy)
            .add_systems(Update, (
                enemy_patrol,
                check_enemy_death,
            ));
    }
}

pub fn setup_enemy(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let _enemy_entity = spawn_living(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        "Enemy",
        CharacterVisualConfig {
            texture_path: "enemies/Slime_pack/Tiny_Slime Green.png",
            tile_size: UVec2::new(24, 24),
            columns: 10,
            rows: 6,
            initial_frame_index: 11,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
        },
        CharacterPhysicsConfig {
            collider: Collider::capsule(1.0, 1.0),
            lock_rotation: true,
            spawn_position: Vec3::new(5.0, 2.0, 0.0),
        },
        |cmd| {
            cmd.insert((IsEnemy, Team::Enemy));

            // Collision layers specific to enemies
            cmd.insert(CollisionLayers::new(
                GameLayer::EnemyUnit,
                [GameLayer::World, GameLayer::FriendlyProjectile, GameLayer::FriendlyUnit],
            ));
            
            // Gameplay state
            cmd.insert(Health(50.0));

            // Patrol behavior
            cmd.insert(Patrol {
                speed: 2.0,
                direction: 1.0,
                edge_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            });
        },
    );
}

/// Patrol movement system - flips sprite based on patrol direction
pub fn enemy_patrol(
    time: Res<Time>,
    mut enemies: Query<(&mut LinearVelocity, &mut Patrol, &Children), With<IsEnemy>>,
    mut sprites: Query<&mut Sprite, With<CharacterSprite>>,
) {
    let delta = time.delta();

    for (mut velocity, mut patrol, children) in enemies.iter_mut() {
        // Reverse direction on timer tick
        if patrol.edge_timer.tick(delta).just_finished() || time.elapsed_secs() < 0.1 { // Needs to start facing left
            patrol.direction *= -1.0;
            for &child in children {
                if let Ok(mut sprite) = sprites.get_mut(child) {
                    flip_sprite_for_direction(&mut sprite, patrol.direction);
                }
            }
        }

        // Apply horizontal movement (preserve vertical for gravity)
        velocity.x = patrol.speed * patrol.direction;
    }
}

/// Death check - despawns enemies when health reaches zero
pub fn check_enemy_death(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<IsEnemy>>,
) {
    for (entity, health) in query.iter() {
        if health.0 <= 0.0 {
            commands.entity(entity).despawn();
            info!("Enemy defeated!");
        }
    }
}