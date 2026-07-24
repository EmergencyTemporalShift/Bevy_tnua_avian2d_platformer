use avian2d::prelude::*;
use bevy::prelude::*;

pub mod components;
pub mod config;
pub mod spawner;
pub mod enemy;
pub mod player;
pub mod weapon_shooting;

// Re-export common items so external code isn't impacted by this refactor
pub use components::*;
pub use config::*;
pub use spawner::*;

#[allow(dead_code)]
#[derive(PhysicsLayer, Default)]
pub(crate) enum GameLayer {
    #[default]
    World,
    FriendlyUnit,
    FriendlyProjectile,
    EnemyUnit,
    EnemyProjectile,
    NeutralUnit,
    NeutralProjectile,
}

pub struct LivingPlugin;

impl Plugin for LivingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Team>()
            .register_type::<CharacterSprite>()
            .register_type::<CharacterVisualConfig>()
            .register_type::<CharacterPhysicsConfig>();
    }
}