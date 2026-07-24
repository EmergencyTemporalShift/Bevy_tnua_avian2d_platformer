#![allow(unexpected_cfgs)]

use bevy::{color::palettes::css, prelude::*};

use avian2d::prelude::*;
use bevy_tnua::TnuaGhostPlatform;
use bevy_tnua::math::{Vector2, Vector3};

use crate::level_mechanics::MovingPlatform;

use super::{
    PositionPlayer,
    helper::{LevelSetupHelper2d, LevelSetupHelper2dEntityCommandsExtension},
};

#[derive(PhysicsLayer, Default)]
pub enum LayerNames {
    #[default]
    Default,
    Player,
    FallThrough,
    PhaseThrough,
}

pub fn setup_level(mut helper: LevelSetupHelper2d) {
    helper.spawn(PositionPlayer::from(Vec3::new(0.0, 2.0, 0.0)));

    helper.spawn_floor(css::GRAY);

    helper.spawn_rectangle(
        "Moderate Slope",
        css::GRAY,
        Transform::from_xyz(7.0, 7.0, 0.0).with_rotation(Quat::from_rotation_z(0.6)),
        Vector2::new(10.0, 0.1),
    );
    helper.spawn_rectangle(
        "Steep Slope",
        css::GRAY,
        Transform::from_xyz(14.0, 14.0, 0.0).with_rotation(Quat::from_rotation_z(1.0)),
        Vector2::new(10.0, 0.1),
    );
    helper.spawn_rectangle(
        "Box to Step on",
        css::GRAY,
        Transform::from_xyz(-4.0, 1.0, 0.0),
        Vector2::new(4.0, 2.0),
    );
    helper.spawn_rectangle(
        "Floating Box",
        css::GRAY,
        Transform::from_xyz(-10.0, 4.0, 0.0),
        Vector2::new(6.0, 1.0),
    );
    helper.spawn_rectangle(
        "Box to Crawl Under",
        css::GRAY,
        Transform::from_xyz(-20.0, 2.6, 0.0),
        Vector2::new(6.0, 1.0),
    );

    // Fall-through platforms
    for (i, y) in [5.0, 7.5].into_iter().enumerate() {
        helper
            .spawn_rectangle(
                format!("Fall Through #{}", i + 1),
                css::PINK,
                Transform::from_xyz(-20.0, y, -1.0),
                Vector2::new(6.0, 0.5),
            )
            .insert((
                CollisionLayers::new([LayerNames::FallThrough], [LayerNames::FallThrough]),
                TnuaGhostPlatform,
            ));
    }

    helper
        .spawn_text_circle(
            "Collision Groups",
            "collision\ngroups",
            0.01,
            Transform::from_xyz(10.0, 2.0, 0.0),
            1.0,
        )
        .insert((
            CollisionLayers::new([LayerNames::PhaseThrough], [LayerNames::PhaseThrough]),
        ));

    helper
        .spawn_text_circle(
            "Sensor",
            "sensor",
            0.01,
            Transform::from_xyz(20.0, 2.0, 0.0),
            1.0,
        )
        .insert((
            Sensor,
        ));

    // spawn moving platform
    helper
        .spawn_rectangle(
            "Moving Platform",
            css::BLUE,
            Transform::from_xyz(-4.0, 6.0, 0.0),
            Vector2::new(4.0, 1.0),
        )
        .make_kinematic()
        .insert(MovingPlatform::new(
            4.0,
            &[
                Vector3::new(-4.0, 6.0, 0.0),
                Vector3::new(-8.0, 6.0, 0.0),
                Vector3::new(-8.0, 10.0, 0.0),
                Vector3::new(-4.0, 10.0, 0.0),
            ],
        ));
}
