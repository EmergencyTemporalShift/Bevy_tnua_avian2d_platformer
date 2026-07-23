use avian2d::parry::glamx::Vec2;
use glamour::{Unit, Vector2};
#[allow(unused)]
pub enum ScreenSpace {}

impl Unit for ScreenSpace {
    type Scalar = f32;
}

#[allow(unused)]
pub enum CartesianSpace {}

impl Unit for CartesianSpace {
    type Scalar = f32;
}

pub enum WindowSpace {}

impl Unit for WindowSpace {
    type Scalar = f32;
}

#[allow(unused)]
pub enum NormalizedWindowSpace {}

impl Unit for NormalizedWindowSpace {
    type Scalar = f32;
}

/// Conversions for the vector representations used by this module.
pub trait Vector2Ext<U: Unit<Scalar = f32>> {
    /// Changes the compile-time coordinate-space tag without changing values.
    #[allow(unused)]
    fn retag<V: Unit<Scalar = f32>>(self) -> Vector2<V>;

    /// Converts a Glamour vector into Bevy's vector representation.
    fn to_bevy(self) -> Vec2;
}

impl<U: Unit<Scalar = f32>> Vector2Ext<U> for Vector2<U> {
    #[inline]
    fn retag<V: Unit<Scalar = f32>>(self) -> Vector2<V> {
        Vector2::new(self.x, self.y)
    }

    #[inline]
    fn to_bevy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

pub trait BevyVec2Ext {
    fn to_space<U: Unit<Scalar = f32>>(self) -> Vector2<U>;
}

impl BevyVec2Ext for Vec2 {
    #[inline]
    fn to_space<U: Unit<Scalar = f32>>(self) -> Vector2<U> {
        Vector2::new(self.x, self.y)
    }
}