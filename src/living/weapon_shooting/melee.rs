use bevy::prelude::*;
use crate::living::{Health, Team};
use super::components::*;
use super::events::FireWeapon;

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

            if let (Some(w_team), Some(t_team)) = (wielder_team, target_team) {
                if w_team == t_team {
                    continue;
                }
            }

            let to_target = (target_tf.translation.truncate() - event.origin.truncate()).normalize_or_zero();
            let dot = to_target.dot(event.direction.as_vec2());
            let dist = target_tf.translation.truncate().distance(event.origin.truncate());

            if dot > (melee.arc / 2.0).cos() && dist <= melee.reach {
                commands.entity(target).insert(Health(-melee.damage));
            }
        }
    }
}