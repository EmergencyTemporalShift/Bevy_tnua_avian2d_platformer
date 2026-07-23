use avian2d::prelude::*;
use bevy::prelude::*;

pub mod player;
pub mod enemy;
pub mod weapon_shooting;
// ─── Physics Layers ─────────────────────────────────────────

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

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Reflect, Default)]
#[reflect(Component)]
pub enum Team {
    #[default]
    Neutral,
    Player,
    Enemy,
}

/// Simple health component.
/// Not required if the entity doesn't take damage.
#[derive(Component)]
pub struct Health(pub f32);

pub struct LivingPlugin;

impl Plugin for LivingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Team>()
            .register_type::<CharacterSprite>()
            .register_type::<CharacterVisualConfig>()
            .register_type::<CharacterPhysicsConfig>();
    }
}

/// Common configuration for spawning any character entity's visual representation
#[derive(Clone, Reflect)]
pub struct CharacterVisualConfig {
    /// Path to the texture asset
    pub texture_path: &'static str,
    /// Grid cell size for atlas layout
    pub tile_size: UVec2,
    /// Number of columns in the atlas
    pub columns: u32,
    /// Number of rows in the atlas
    pub rows: u32,
    /// Initial sprite frame index
    pub initial_frame_index: usize,
    /// Scale factor applied to the sprite (not the physics body)
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

/// Common physics configuration for character entities
#[derive(Clone, Reflect)]
pub struct CharacterPhysicsConfig {
    /// Collider to use (capsule, circle, etc.)
    #[reflect(ignore)]
    pub collider: Collider,
    /// Whether to lock rotation
    pub lock_rotation: bool,
    /// Starting position
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

/// Marker for the sprite child of a character
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CharacterSprite;

/// Spawns a character entity with a physics body and a child sprite.
///
/// This is the shared spawn logic for all character-like entities.
/// Callers pass their own marker component(s) via the `extra` closure
/// to specialize the entity (e.g., `IsPlayer`, `IsEnemy`, health, etc.).
///
/// Usage:
/// ```ignore
/// spawn_character(
///     &mut commands,
///     &asset_server,
///     &mut texture_atlas_layouts,
///     "Player",
///     CharacterVisualConfig { /* ... */ },
///     CharacterPhysicsConfig { /* ... */ },
///     |cmd| {
///         cmd.insert(IsPlayer);
///         cmd.insert(TnuaController::default());
///         // ... player-specific components
///     },
/// )
/// ```
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
    let layout =
        TextureAtlasLayout::from_grid(visual.tile_size, visual.columns, visual.rows, None, None);
    let atlas_layout = texture_atlas_layouts.add(layout);

    let mut cmd = commands.spawn((
        Name::new(name.to_string()),
        // Physics
        RigidBody::Dynamic,
        physics.collider,
        LinearVelocity::default(),
        Transform::from_translation(physics.spawn_position),
        Visibility::default(),
    ));

    if physics.lock_rotation {
        cmd.insert(LockedAxes::new().lock_rotation());
    }

    // Child sprite entity — local transform offsets are relative to the physics parent
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
            Visibility::default(),
        ));
    });

    // Apply caller-specific components
    extra(&mut cmd);

    cmd.id()
}

/// Flips a sprite horizontally based on a direction sign.
/// Positive = face right, negative = face left.
pub fn flip_sprite_for_direction(sprite: &mut Sprite, direction_x: f32) {
    sprite.flip_x = direction_x < 0.0;
    // Handle facing forward
}