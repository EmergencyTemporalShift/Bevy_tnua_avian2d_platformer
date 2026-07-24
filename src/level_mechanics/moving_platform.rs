use avian2d::prelude::LinearVelocity;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy_tnua::math::{AdjustPrecision, Float, Vector3};

pub struct MovingPlatformPlugin;

impl Plugin for MovingPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            MovingPlatform::make_system(
                |velocity: &mut LinearVelocity, linvel: Vector3| {
                    velocity.0 = linvel.truncate();
                },
            ),
        );
    }
}

#[derive(Component)]
pub struct MovingPlatform {
    pub current_leg: usize,
    pub speed: Float,
    pub locations: Vec<Vector3>,
}

impl MovingPlatform {
    pub fn new(speed: Float, locations: &[Vector3]) -> Self {
        Self {
            current_leg: 0,
            speed,
            locations: locations.to_owned(),
        }
    }

    fn make_system<V: Component<Mutability = bevy::ecs::component::Mutable>>(
        mut updater: impl 'static + Send + Sync + FnMut(&mut V, Vector3),
    ) -> ScheduleConfigs<ScheduleSystem> {
        (move |time: Res<Time>,
               mut query: Query<(&mut MovingPlatform, &GlobalTransform, &mut V)>| {
            for (mut moving_platform, transform, mut velocity) in query.iter_mut() {
                let current = transform.translation().adjust_precision();
                let target = moving_platform.locations[moving_platform.current_leg];
                let vec_to = target - current;
                updater(
                    velocity.as_mut(),
                    vec_to.normalize_or_zero() * moving_platform.speed,
                );
                if vec_to.length() <= time.delta_secs().adjust_precision() * moving_platform.speed {
                    moving_platform.current_leg =
                        (moving_platform.current_leg + 1) % moving_platform.locations.len();
                }
            }
        })
        .into_configs()
    }
}
