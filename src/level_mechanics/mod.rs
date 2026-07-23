mod cannon;
mod center_of_gravity;
mod moving_platform;
mod push_effect;
mod time_to_despawn;

use bevy::prelude::*;

#[allow(unused_imports)]
pub use cannon::{Cannon, CannonBullet};
pub use center_of_gravity::IsCenterOfGravity;
pub use moving_platform::MovingPlatform;
#[allow(unused_imports)]
pub use push_effect::PushEffect;
#[allow(unused_imports)]
pub use time_to_despawn::TimeToDespawn;

pub struct LevelMechanicsPlugin;

impl Plugin for LevelMechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(moving_platform::MovingPlatformPlugin);
        app.add_plugins(cannon::CannonPlugin);
        app.add_plugins(push_effect::PushEffectPlugin);
        app.add_plugins(time_to_despawn::TimeToDespawnPlugin);
        app.add_plugins(center_of_gravity::CenterOfGravityPlugin);
    }
}

#[derive(Component)]
pub struct Climbable;
