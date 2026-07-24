use avian2d::prelude::*;
use bevy::prelude::*;
use super::components::CharacterSprite;
use super::config::{CharacterPhysicsConfig, CharacterVisualConfig};

pub fn spawn_living(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    name: &str,
    visual: CharacterVisualConfig,
    physics: CharacterPhysicsConfig,
    extra: impl FnOnce(&mut EntityCommands),
) -> Entity {
    let texture = asset_server.load(visual.texture_path);
    let layout = TextureAtlasLayout::from_grid(visual.tile_size, visual.columns, visual.rows, None, None);
    let atlas_layout = texture_atlas_layouts.add(layout);

    let mut cmd = commands.spawn((
        Name::new(name.to_string()),
        RigidBody::Dynamic,
        physics.collider,
        LinearVelocity::default(),
        Transform::from_translation(physics.spawn_position),
        Visibility::default(),
    ));

    if physics.lock_rotation {
        cmd.insert(LockedAxes::new().lock_rotation());
    }

    cmd.with_children(|parent| {
        parent.spawn((
            CharacterSprite,
            Name::new(format!("{name} Sprite")),
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: atlas_layout,
                    index: visual.initial_frame_index,
                },
            ),
            Transform::from_scale(visual.sprite_scale),
        ));
    });

    extra(&mut cmd);
    cmd.id()
}

pub fn flip_sprite_for_direction(sprite: &mut Sprite, direction_x: f32) {
    sprite.flip_x = direction_x < 0.0;
}