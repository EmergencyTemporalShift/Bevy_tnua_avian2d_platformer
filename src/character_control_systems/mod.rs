use serde::{Deserialize, Serialize};

pub mod info_dumping_systems;
pub mod platformer_control_scheme;
pub mod platformer_control_systems;
pub mod player_input;
mod querying_helpers;
mod spatial_ext_facade;

pub use crate::living::weapon_shooting::WeaponPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimensionality {
    Dim2,
    Dim3,
}
