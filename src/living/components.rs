use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Reflect, Default)]
#[reflect(Component)]
pub enum Team {
    #[default]
    Neutral,
    Player,
    Enemy,
}

/// Simple health component.
#[derive(Component)]
pub struct Health(pub f32);

/// Marker for the sprite child of a character
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform, Visibility)] // Automatically attached if missing!
pub struct CharacterSprite;