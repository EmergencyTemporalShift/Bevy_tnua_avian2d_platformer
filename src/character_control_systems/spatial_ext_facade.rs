use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use bevy_tnua::math::Vector3;
use bevy_tnua::spatial_ext::{TnuaPointProjectionResult, TnuaSpatialExt};
use bevy_tnua_avian2d::TnuaSpatialExtAvian2d;


#[derive(SystemParam)]
pub struct SpatialExtFacade<'w, 's> {
    for_avian2d: TnuaSpatialExtAvian2d<'w, 's>,
    _phantom: PhantomData<(&'w (), &'s ())>,
}

#[allow(unreachable_code)]
impl<'w, 's> TnuaSpatialExt for SpatialExtFacade<'w, 's> {
    type ColliderData<'a>
        = ColliderDataFacade<'a, 'w, 's>
    where
        Self: 'a;

    fn fetch_collider_data(&self, entity: Entity) -> Option<Self::ColliderData<'_>> {
        Some(ColliderDataFacade {
            for_avian2d: self.for_avian2d.fetch_collider_data(entity)?,
            _phantom: PhantomData,
        })
    }

    fn project_point(
        &'_ self,
        point: Vector3,
        solid: bool,
        collider_data: &Self::ColliderData<'_>,
    ) -> TnuaPointProjectionResult {
        return self
            .for_avian2d
            .project_point(point, solid, &collider_data.for_avian2d);

        panic!("Running without any physics backend configured");
    }

    fn cast_ray(
        &'_ self,
        origin: Vector3,
        direction: Vector3,
        max_time_of_impact: bevy_tnua::math::Float,
        collider_data: &Self::ColliderData<'_>,
    ) -> Option<(bevy_tnua::math::Float, Vector3)> {
        #[cfg(feature = "avian2d")]
        return self.for_avian2d.cast_ray(
            origin,
            direction,
            max_time_of_impact,
            &collider_data.for_avian2d,
        );

        panic!("Running without any physics backend configured");
    }

    fn can_interact(&self, entity1: Entity, entity2: Entity) -> bool {
        #[cfg(feature = "avian2d")]
        return self.for_avian2d.can_interact(entity1, entity2);

        panic!("Running without any physics backend configured");
    }
}

pub struct ColliderDataFacade<'a, 'w, 's>
where
    Self: 'a,
{
    for_avian2d: <TnuaSpatialExtAvian2d<'w, 's> as TnuaSpatialExt>::ColliderData<'a>,
    _phantom: PhantomData<(&'a (), &'w (), &'s ())>,
}
