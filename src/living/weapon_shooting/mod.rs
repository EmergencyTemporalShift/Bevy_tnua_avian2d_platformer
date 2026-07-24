use bevy::prelude::*;

pub mod beam;
pub mod components;
pub mod equip;
pub mod events;
pub mod melee;
pub mod projectiles;

pub use components::*;
pub use equip::*;
pub use events::*;

use beam::*;
use melee::*;
use projectiles::*;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FireWeapon>()
            .add_systems(
                Update,
                (
                    fire_projectile_weapon,
                    fire_melee_weapon,
                    handle_fire_beam_messages,
                    draw_active_beams,
                    process_projectile_ttl,
                    enable_projectile_wielder_collisions,
                    tick_fire_rate,
                    apply_active_weapon,
                ).chain(),
            );
    }
}

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