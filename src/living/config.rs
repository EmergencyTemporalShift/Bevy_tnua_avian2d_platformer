use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Clone, Reflect)]
pub struct CharacterVisualConfig {
    pub texture_path: &'static str,
    pub tile_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub initial_frame_index: usize,
    pub sprite_scale: Vec3,
}

impl Default for CharacterVisualConfig {
    fn default() -> Self {
        Self {
            texture_path: "",
            tile_size: UVec2::new(24, 24),
            columns: 10,
            rows: 1,
            initial_frame_index: 0,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
        }
    }
}

#[derive(Clone, Reflect)]
pub struct CharacterPhysicsConfig {
    #[reflect(ignore)]
    pub collider: Collider,
    pub lock_rotation: bool,
    pub spawn_position: Vec3,
}

impl Default for CharacterPhysicsConfig {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(0.5, 1.0),
            lock_rotation: true,
            spawn_position: Vec3::ZERO,
        }
    }
}