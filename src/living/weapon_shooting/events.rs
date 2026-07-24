use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponIntent {
    BeginHold,
    ContinueHold,
    ReleaseHold,
}

#[derive(Message)]
pub struct FireWeapon {
    pub wielder: Entity,
    pub origin: Vec3,
    pub direction: Dir2,
    pub intent: WeaponIntent,
}