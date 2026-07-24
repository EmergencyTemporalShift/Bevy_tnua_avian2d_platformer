use avian2d::prelude::*;
use bevy::prelude::*;
use crate::living::{GameLayer::*, Team};
use super::components::*;
use super::events::FireWeapon;

pub fn fire_projectile_weapon(
    mut fire_messages: MessageReader<FireWeapon>,
    mut wielders: Query<(&Weapon, &ProjectileSpawner, &LinearVelocity, Option<&mut FireRate>, Option<&Team>)>,
    projectiles: Query<&Projectile>,
    mut commands: Commands,
) {
    for event in fire_messages.read() {
        let Ok((_, spawner, vel, mut fire_rate, wielder_team)) = wielders.get_mut(event.wielder) else { continue };

        if let Some(ref mut fr) = fire_rate {
            fr.0.reset();
        }

        let active = projectiles.iter().filter(|p| p.fired_by == event.wielder).count();
        if active >= spawner.max_simultaneous_projectiles {
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
            Name(spawner.projectile_name.clone().into()),
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

pub fn process_projectile_ttl(
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

pub fn enable_projectile_wielder_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform, &CollisionLayers)>,
    wielders: Query<(&Transform, Option<&Team>)>,
) {
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
                        [World, EnemyUnit, EnemyProjectile, FriendlyProjectile],
                    ),
                    Some(Team::Enemy) => (
                        EnemyProjectile,
                        [World, FriendlyUnit, FriendlyProjectile, EnemyProjectile],
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