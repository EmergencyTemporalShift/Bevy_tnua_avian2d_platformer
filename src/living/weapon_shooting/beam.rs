use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::prelude::*;
use crate::util::particles::{spawn_particle_entity, ParticleEffectHandle};
use super::components::*;
use super::events::{FireWeapon, WeaponIntent};

pub fn handle_fire_beam_messages(
    mut commands: Commands,
    mut fire_messages: MessageReader<FireWeapon>,
    wielders: Query<&Beam>,
    spatial_query: SpatialQuery,
    effect_handle: Res<ParticleEffectHandle>,
) {
    for event in fire_messages.read() {
        let Ok(beam) = wielders.get(event.wielder) else { continue };

        let filter = SpatialQueryFilter::default().with_excluded_entities([event.wielder]);
        let direction_2d = event.direction.as_vec2().normalize_or_zero();
        let origin_2d = event.origin.truncate();

        let end_point = if let Some(hit) = spatial_query.cast_ray(
            origin_2d,
            direction_2d.try_into().unwrap(),
            beam.range,
            true,
            &filter,
        ) {
            (origin_2d + direction_2d * hit.distance).extend(0.0)
        } else {
            (origin_2d + direction_2d * beam.range).extend(0.0)
        };

        match event.intent {
            WeaponIntent::BeginHold | WeaponIntent::ContinueHold => {
                commands.entity(event.wielder).insert(BeamActive { end_point });
                spawn_particle_entity(&mut commands, &effect_handle, end_point);
            }
            WeaponIntent::ReleaseHold => {
                commands.entity(event.wielder).remove::<BeamActive>();
            }
        }
    }
}

pub fn draw_active_beams(
    query: Query<(&GlobalTransform, &BeamActive)>,
    mut gizmos: Gizmos,
) {
    for (transform, active_beam) in &query {
        let start = transform.translation().truncate();
        gizmos.line_2d(start, active_beam.end_point.truncate(), css::RED);
    }
}