use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::living::player::IsPlayer;
use crate::levels_setup::LevelObject;

#[derive(Component)]
pub struct Cannon {
    pub timer: Timer,
    pub cmd: Box<dyn Send + Sync + Fn(&mut EntityCommands)>,
}

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (shoot, handle_collision));
    }
}

fn shoot(
    time: Res<Time>,
    mut query: Query<(&mut Cannon, &GlobalTransform, Option<&Name>)>,
    mut commands: Commands,
) {
    for (mut cannon, cannon_transform, cannon_name) in query.iter_mut() {
        if cannon.timer.tick(time.delta()).just_finished() {
            let mut cmd = commands.spawn(LevelObject);
            if let Some(cannon_name) = cannon_name.as_ref() {
                cmd.insert(Name::new(format!("{cannon_name} projectile")));
            }

            (cannon.cmd)(&mut cmd);
            cmd.insert(Transform::from_translation(cannon_transform.translation()));
        }
    }
}

#[derive(Component)]
#[allow(clippy::type_complexity)]
pub struct CannonBullet {
    effect: Box<dyn Send + Sync + Fn(&mut EntityCommands)>,
}

impl CannonBullet {
    pub fn new_with_effect(effect: impl 'static + Send + Sync + Fn(&mut EntityCommands)) -> Self {
        Self {
            effect: Box::new(effect),
        }
    }
}

fn handle_collision(
    bullets_query: Query<&CannonBullet>,
    player_query: Query<(), With<IsPlayer>>,
    mut commands: Commands,
) {
    let events = std::iter::empty::<(Entity, Entity)>();

    let events = events.flat_map(|(e1, e2)| [(e1, e2), (e2, e1)]);

    for (bullet_entity, player_entity) in events {
        let Ok(bullet) = bullets_query.get(bullet_entity) else {
            continue;
        };
        if !player_query.contains(player_entity) {
            continue;
        }
        (bullet.effect)(&mut commands.entity(player_entity));
        commands.entity(bullet_entity).despawn();
    }
}
