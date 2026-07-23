use bevy::{
    color::palettes::css,
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use avian2d::prelude as avian;

use bevy_tnua::math::Vector2;
#[allow(unused_imports)]
use bevy_tnua::math::{AsF32, Float};

use crate::levels_setup::LevelObject;

#[derive(SystemParam, Deref, DerefMut)]
pub struct LevelSetupHelper2d<'w, 's> {
    #[deref]
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
}

impl LevelSetupHelper2d<'_, '_> {
    pub fn spawn_named(&'_ mut self, name: impl ToString) -> EntityCommands<'_> {
        self.commands
            .spawn((LevelObject, Name::new(name.to_string())))
    }

    pub fn spawn_floor(&'_ mut self, color: impl Into<Color>) -> EntityCommands<'_> {
        let mut cmd = self.spawn_named("Floor");
        cmd.insert(Sprite {
            custom_size: Some(Vec2::new(128.0, 0.5)),
            color: color.into(),
            ..Default::default()
        });

        {
            cmd.insert(avian::RigidBody::Static);
            cmd.insert(avian::Collider::half_space(Vector2::Y));
        }

        cmd
    }

    pub fn spawn_rectangle(
        &'_ mut self,
        name: impl ToString,
        color: impl Into<Color>,
        transform: Transform,
        size: Vector2,
    ) -> EntityCommands<'_> {
        let mut cmd = self.spawn_named(name);

        cmd.insert((
            Sprite {
                custom_size: Some(size.f32()),
                color: color.into(),
                ..Default::default()
            },
            transform,
        ));

        {
            cmd.insert(avian::RigidBody::Static);
            cmd.insert(avian::Collider::rectangle(size.x, size.y));
        }

        cmd
    }

    pub fn spawn_compound_rectangles(
        &'_ mut self,
        name: impl ToString,
        color: impl Into<Color>,
        transform: Transform,
        parts: &[(Vector2, Float, Vector2)],
    ) -> EntityCommands<'_> {
        let mut cmd = self.spawn_named(name);

        cmd.insert((
            //Sprite {
            //custom_size: Some(size.f32()),
            //color: color.into(),
            //..Default::default()
            //},
            transform,
        ));
        let color = color.into();

        cmd.with_children(|commands| {
            for (pos, rot, size) in parts.iter().copied() {
                commands.spawn((
                    Sprite {
                        custom_size: Some(size.f32()),
                        color,
                        ..Default::default()
                    },
                    Transform {
                        translation: pos.extend(0.0).f32(),
                        rotation: Quat::from_rotation_z(rot.f32()),
                        scale: Vec3::ONE,
                    },
                ));
            }
        });

        {
            cmd.insert(avian::RigidBody::Static);
            cmd.insert(avian::Collider::compound(
                parts
                    .iter()
                    .map(|&(pos, rot, size)| (pos, rot, avian::Collider::rectangle(size.x, size.y)))
                    .collect(),
            ));
        }

        cmd
    }

    pub fn spawn_text_circle(
        &'_ mut self,
        name: impl ToString,
        text: impl ToString,
        text_scale: Float,
        transform: Transform,
        #[allow(unused)] radius: Float,
    ) -> EntityCommands<'_> {
        let font = self.asset_server.load("FiraSans-Bold.ttf").into();
        let child = self
            .spawn((
                LevelObject,
                Text::new(text.to_string()),
                TextLayout::justify(Justify::Center),
                TextFont {
                    font,
                    font_size: FontSize::Px(72.0),
                    ..default()
                },
                TextColor(css::WHITE.into()),
                Transform::from_xyz(0.0, 0.0, 1.0).with_scale(text_scale.f32() * Vec3::ONE),
            ))
            .id();
        let mut cmd = self.spawn_named(name);
        cmd.add_child(child);
        cmd.insert((
            transform,
            #[cfg(feature = "avian2d")]
            (avian::RigidBody::Static, avian::Collider::circle(radius)),
        ));
        cmd
    }

    pub fn spawn_dynamic_rectangle(
        &'_ mut self,
        name: impl ToString,
        color: impl Into<Color>,
        transform: Transform,
        size: Vector2,
    ) -> EntityCommands<'_> {
        let mut cmd = self.spawn_named(name);

        cmd.insert((
            Sprite {
                custom_size: Some(size.f32()),
                color: color.into(),
                ..Default::default()
            },
            transform,
        ));
        {
            cmd.insert(avian::RigidBody::Dynamic);
            cmd.insert(avian::Collider::rectangle(size.x, size.y));
        }

        cmd
    }

    pub fn spawn_circle(
        &'_ mut self,
        name: impl ToString,
        //color: impl Into<Color>,
        transform: Transform,
        #[allow(unused)] radius: Float,
    ) -> EntityCommands<'_> {
        let mut cmd = self.spawn_named(name);

        cmd.insert((
            // Sprite {
            // custom_size: Some(size.f32()),
            // color: color.into(),
            // ..Default::default()
            // },
            transform,
        ));

        cmd.insert((
            (avian::RigidBody::Static, avian::Collider::circle(radius)),
        ));

        cmd
    }
}

pub trait LevelSetupHelper2dEntityCommandsExtension {
    fn make_kinematic(&mut self) -> &mut Self;
    fn make_sensor(&mut self) -> &mut Self;
    // fn give_weapon(&mut self, name: String, speed: f32, max_projectiles: usize) -> &mut Self;
}

impl LevelSetupHelper2dEntityCommandsExtension for EntityCommands<'_> {
    fn make_kinematic(&mut self) -> &mut Self {
        self.insert((
            avian::RigidBody::Kinematic,
        ))
    }

    fn make_sensor(&mut self) -> &mut Self {
        self.insert((
            avian::Sensor,
        ))
    }

//     fn give_weapon(&mut self, name: String, speed: f32, max_projectiles: usize) -> &mut Self {
//         self.insert(crate::character_control_systems::weapon_shooting::Weapon {
//             name            : "Bow".to_owned(),
//             projectile_speed: speed,
//             max_projectiles,
//         })
//     }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 14.0, 30.0)
            .with_scale((0.05 * Vec2::ONE).extend(1.0))
            .looking_at(Vec3::new(0.0, 14.0, 0.0), Vec3::Y),
    ));
}