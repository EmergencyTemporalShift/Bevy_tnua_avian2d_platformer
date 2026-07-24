use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// Plugin to register and manage particle effects
pub struct ParticlePlugin;

#[derive(Resource, Default)]
pub struct ParticleEffectHandle(Handle<EffectAsset>);

#[derive(Component)]
pub struct Lifetime(pub Timer);

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        // Register your custom resource type
        app.init_resource::<ParticleEffectHandle>();

        // Setup runs first, spawns can happen in Update
        app.add_systems(Startup, setup_particle_effects);
        app.add_systems(Update, tick_despawn_timers);
        //app.add_systems(Update, spawn_particle_entity);
    }
}

fn setup_particle_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut gradient = bevy_hanabi::Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::ZERO);

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(2.),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles will stay
    // alive forever, and new particles can't be spawned instead.
    let lifetime = module.lit(0.5); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        1024,
        // Spawn at a rate of 5 particles per second
        SpawnerSettings::once(30.0.into()),
        // Move the expression module into the asset
        module,
    )
        .with_name("MyEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        // Render the particles with a color gradient over their
        // lifetime. This maps the gradient key 0 to the particle spawn
        // time, and the gradient key 1 to the particle death (10s).
        .render(ColorOverLifetimeModifier {
            gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        });

    let effect_asset = effects.add(effect);
    commands.insert_resource(ParticleEffectHandle(effect_asset));
}

pub fn spawn_particle_entity(
    commands: &mut Commands,
    effect_handle: &Res<ParticleEffectHandle>,
    position: Vec3,
) {
    commands.spawn((
                       Name("Particle Thing".into()),
                       ParticleEffect::new(effect_handle.0.clone()),
                       Transform::from_translation(position),
                       Lifetime(Timer::from_seconds(0.5, TimerMode::Once)),
                   ),
    );
}

pub fn tick_despawn_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn pause_particle_spawners(
    mut query: Query<&mut EffectSpawner>,
) {
    for mut spawner in &mut query {
        spawner.active = false; // Stops generating new particles
    }
}

pub fn unpause_particle_spawners(
    mut query: Query<&mut EffectSpawner>,
) {
    for mut spawner in &mut query {
        spawner.active = true;
    }
}