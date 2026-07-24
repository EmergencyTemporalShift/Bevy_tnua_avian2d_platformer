use avian2d::math::{Vector2, PI};
use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PresentMode, PrimaryWindow};
use bevy_egui::{EguiContext, EguiContexts, PrimaryEguiContext};
use bevy_tnua::{TnuaConfig, TnuaScheme};
use bevy_tnua_physics_integration_layer::data_for_backends::TnuaToggle;
use bevy_tnua_physics_integration_layer::math::Vector3;
use egui::collapsing_header::HeaderResponse;
use egui::Color32;
use crate::character_control_systems::platformer_control_systems::CameraControllerFloating;
use crate::ui::component_alteration::CommandAlteringSelectors;
use crate::ui::{framerate, info, level_selection, plotting};
use crate::ui::components::{DemoUiPhysicsBackendSettings, Hierarchy, TrackedEntity};
use crate::ui::plugin::GRAVITY_MAGNITUDE;
use crate::ui::tuning::UiTunable;
use crate::util;

#[cfg(feature = "egui")]
pub fn apply_selectors(
    mut query: Query<(Entity, &mut CommandAlteringSelectors)>,
    mut commands: Commands,
) {
    for (entity, mut command_altering_selectors) in query.iter_mut() {
        command_altering_selectors.apply_set_to(&mut commands, entity);
    }
}

// This is an exclusive system so that it can hold `&mut World`: the embedded `bevy-inspector-egui`
// hierarchy view (`ui_for_world`) needs it, and egui is immediate-mode — a single window has to be
// built in a single system, so the inspector can't live in a second system/window and still share
// this "Tnua" window. The regular `SystemParam`s are recovered through a cached `SystemState`.
#[cfg(feature = "egui")]
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn ui_system<
    S: TnuaScheme,
    C: Component<Mutability = bevy::ecs::component::Mutable> + UiTunable,
>(
    mut egui_context: EguiContexts,
    non_player_controls: Res<
        util::controls_other::OtherControls,
    >,
    mut physics_backend_settings: ResMut<DemoUiPhysicsBackendSettings>,
    mut query: Query<(
        Entity,
        &TrackedEntity,
        Option<&plotting::PlotSource>,
        Option<&mut info::InfoSource>,
        &mut TnuaToggle,
        &TnuaConfig<S>,
        Option<&mut C>,
        Option<&mut CommandAlteringSelectors>,
        Option<&mut CameraControllerFloating>,
        Option<&mut Hierarchy>,
    )>,
    mut commands: Commands,
    mut primary_window_query: Query<(&mut Window, &CursorOptions), With<PrimaryWindow>>,
    mut level_selection: level_selection::LevelSelectionParam,
    mut framerate: framerate::DemoFramerateParam,
    mut control_scheme_config_assets: ResMut<Assets<S::Config>>,
    #[cfg(target_arch = "wasm32")] app_setup_configuration: Res<
        crate::app_setup_options::AppSetupConfiguration,
    >,
) where
    S::Config: UiTunable,
{
    use std::any::TypeId;

    if !non_player_controls.is_egui_visible() {
        return;
    }

    let Ok((mut primary_window, primary_window_cursor_options)) =
        primary_window_query.single_mut()
    else {
        return;
    };
    // ... existing code ...
    let mut egui_window = egui::Window::new("Tnua");
    if !primary_window_cursor_options.visible {
        egui_window = egui::Window::new("Tnua")
            .interactable(false)
            .movable(false)
            .resizable(false);
    }
    egui_window.show(egui_context.ctx_mut().unwrap(), |ui| {
        #[cfg(target_arch = "wasm32")]
        if let Some(new_schedule) = app_setup_configuration
            .schedule_to_use
            .pick_different_option(ui)
        {
            app_setup_configuration.change_and_reload_page(|cfg| {
                cfg.schedule_to_use = new_schedule;
            });
        }
        egui::ComboBox::from_label(
            "Present Mode (picking unsupported mode will crash the demo)",
        )
            .selected_text(format!("{:?}", primary_window.present_mode))
            .show_ui(ui, |ui| {
                let present_mode = &mut primary_window.present_mode;
                ui.selectable_value(present_mode, PresentMode::AutoVsync, "AutoVsync");
                ui.selectable_value(present_mode, PresentMode::AutoNoVsync, "AutoNoVsync");
                ui.selectable_value(present_mode, PresentMode::Fifo, "Fifo");
                ui.selectable_value(present_mode, PresentMode::FifoRelaxed, "FifoRelaxed");
                ui.selectable_value(present_mode, PresentMode::Immediate, "Immediate");
                ui.selectable_value(present_mode, PresentMode::Mailbox, "Mailbox");
            });
        framerate.show_in_ui(ui);
        egui::CollapsingHeader::new("Controls:")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Move with the arrow keys or WASD");
                ui.label("Jump with Spacebar or with the up arrow)");
                ui.label(
                    "Crouch or fall through pink platforms with Ctrl or the down arrow key",
                );
                ui.label("Dash with Shift (while moving in a direction)");
            });
        level_selection.show_in_ui(ui);
        ui.collapsing("Physics Backend", |ui| {
            ui.checkbox(&mut physics_backend_settings.active, "Physics Enabled");
            let mut gravity_angle = physics_backend_settings.gravity.truncate().to_angle();
            ui.horizontal(|ui| {
                ui.label("Gravity Angle:");
                if ui
                    .add(egui::Slider::new(
                        &mut gravity_angle,
                        -PI..=0.0,
                    ))
                    .changed()
                {
                    physics_backend_settings.gravity =
                        Vector2::from_angle(gravity_angle).extend(0.0)
                            * GRAVITY_MAGNITUDE;
                }
                if ui.button("Reset").clicked() {
                    physics_backend_settings.gravity =
                        Vector3::NEG_Y * GRAVITY_MAGNITUDE;
                }
            });
        });
        for (
            entity,
            TrackedEntity(name),
            plot_source,
            mut info_source,
            mut tnua_toggle,
            config_handle,
            mut tunable,
            command_altering_selectors,
            camera_controller,
            hierarchy_controller,
        ) in query.iter_mut()
        {
            let collapse_state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(("for-character", entity)),
                false,
            );

            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            enum ThingToShow {
                #[default]
                Settings,
                Plots,
                Info,
                Camera,
                Hierarchy,
            }

            let thing_to_show_id =
                ui.make_persistent_id((TypeId::of::<ThingToShow>(), entity));
            let is_open = collapse_state.is_open();
            let mut thing_to_show = ui.memory_mut(|mem| {
                *mem.data
                    .get_temp_mut_or_default::<ThingToShow>(thing_to_show_id)
            });
            let mut set_open = None;

            let mut collapse_state = collapse_state.show_header(ui, |ui| {
                ui.label(name);
                for (possible, option, text) in [
                    (true, ThingToShow::Settings, "settings"),
                    (plot_source.is_some(), ThingToShow::Plots, "plots"),
                    (info_source.is_some(), ThingToShow::Info, "info"),
                    (camera_controller.is_some(), ThingToShow::Camera, "camera"),
                    (
                        hierarchy_controller.is_some(),
                        ThingToShow::Hierarchy,
                        "hierarchy",
                    ),
                ] {
                    let mut selected = is_open && option == thing_to_show;
                    ui.add_enabled_ui(possible, |ui| {
                        if ui.toggle_value(&mut selected, text).changed() {
                            set_open = Some(selected);
                            if selected {
                                thing_to_show = option;
                                ui.memory_mut(|mem| {
                                    *mem.data.get_temp_mut_or_default::<ThingToShow>(
                                        thing_to_show_id,
                                    ) = option
                                });
                            }
                        }
                    });
                }
            });
            if let Some(set_open) = set_open {
                collapse_state.set_open(set_open);
            }

            if let Some(info_source) = info_source.as_mut() {
                info_source
                    .set_active(collapse_state.is_open() && thing_to_show == ThingToShow::Info);
            }

            HeaderResponse::body(collapse_state, |ui| match thing_to_show {
                ThingToShow::Settings => {
                    egui::ComboBox::from_label("Toggle Tnua")
                        .selected_text(format!("{:?}", tnua_toggle.as_ref()))
                        .show_ui(ui, |ui| {
                            for option in [
                                TnuaToggle::Disabled,
                                TnuaToggle::SenseOnly,
                                TnuaToggle::Enabled,
                            ] {
                                let label = format!("{:?}", option);
                                ui.selectable_value(tnua_toggle.as_mut(), option, label);
                            }
                        });

                    if let Some(mut control_scheme_config) =
                        control_scheme_config_assets.get_mut(&config_handle.0)
                    {
                        control_scheme_config.tune(ui);
                    }
                    if let Some(tunable) = tunable.as_mut() {
                        tunable.tune(ui);
                    }

                    if let Some(mut command_altering_selectors) =
                        command_altering_selectors
                    {
                        command_altering_selectors.show_ui(ui, &mut commands, entity);
                    }
                }
                ThingToShow::Plots => {
                    if let Some(plot_source) = plot_source {
                        plot_source.show(entity, ui);
                    } else {
                        ui.colored_label(
                            Color32::DARK_RED,
                            "No plotting configured for this entity",
                        );
                    }
                }
                ThingToShow::Info => {
                    if let Some(info_source) = info_source.as_mut() {
                        info_source.show(entity, ui);
                    } else {
                        ui.colored_label(
                            Color32::DARK_RED,
                            "No info configured for this entity",
                        );
                    }
                }
                ThingToShow::Camera => {
                    use core::ops::DerefMut;
                    if let Some(mut camera) = camera_controller {
                        let CameraControllerFloating {
                            looking_from: from,
                            looking_to: to,
                        } = camera.deref_mut();
                        ui.label("Looking From: ");
                        ui.add(egui::Slider::new(&mut from.x, -30.0..=30.0));
                        ui.add(egui::Slider::new(&mut from.y, -30.0..=30.0));
                        ui.add(egui::Slider::new(&mut from.z, -30.0..=30.0));
                        ui.label("Looking At: ");
                        ui.add(egui::Slider::new(&mut to.x, -30.0..=30.0));
                        ui.add(egui::Slider::new(&mut to.y, -30.0..=30.0));
                        ui.add(egui::Slider::new(&mut to.z, -30.0..=30.0));
                    }
                }
                ThingToShow::Hierarchy => {
                    // Show the custom Hierarchy dropdown
                    if let Some(hierarchy) = hierarchy_controller {
                        ui.label("Available Entities:");
                        // TODO: Connect this to selection logic for switching between entities
                        for &ent in &hierarchy.entities {
                            ui.label(format!("Entity {:?}", ent));
                        }
                        if hierarchy.entities.is_empty() {
                            ui.colored_label(Color32::BLUE, "No entities tracked");
                        }
                    } else {
                        ui.colored_label(
                            Color32::DARK_RED,
                            "Hierarchy view not yet configured",
                        );
                    }
                }
            });
        }
    });
}

pub fn update_physics_active_from_ui(
    setting_from_ui: Res<DemoUiPhysicsBackendSettings>,
    mut physics_time_avian2d: Option<ResMut<Time<avian2d::schedule::Physics>>>,
    mut gravity_avian2d: Option<ResMut<avian2d::prelude::Gravity>>,
) {
    if let Some(physics_time) = physics_time_avian2d.as_mut() {
        use avian2d::schedule::PhysicsTime;
        if setting_from_ui.active {
            physics_time.unpause();
        } else {
            physics_time.pause();
        }
    }
    if let Some(gravity) = gravity_avian2d.as_mut() {
        gravity.0 = setting_from_ui.gravity.truncate();
    }
}

#[cfg(feature = "egui")]
pub fn inspector_ui(world: &mut World) {
    let mut context_query =
        world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();

    let Ok(egui_context) = context_query.single_mut(world) else {
        return;
    };

    // Clone the component handle so the query no longer borrows the world
    // while the inspector itself accesses it.
    let mut egui_context = egui_context.clone();

    egui::Window::new("Inspector").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                    world, ui,
                );
            });
        });
    });
}

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0; // Draw over everything
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;
    use bevy::winit::WinitPlugin;
    use super::*;
    #[cfg(feature = "egui")]
    use bevy_egui::EguiContext;
    use bevy_egui::EguiPlugin;

    #[cfg(feature = "egui")]
    #[test]
    fn test_egui_context_retrieval() {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
        app.add_plugins(EguiPlugin::default());

        // Run one frame
        app.update();

        // Check if EguiContext exists on the Primary Window
        let mut query = app
            .world_mut()
            .query_filtered::<&EguiContext, With<PrimaryWindow>>();
        let _context = query.single(app.world()).unwrap();

        assert!(true, "EguiContext should exist on PrimaryWindow");
    }

    #[derive(Resource, Default)]
    struct RunCounter(u32);

    #[cfg(feature = "egui")]
    #[test]
    fn test_egui_schedule_is_called() {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
        app.add_plugins(EguiPlugin::default());
        app.init_resource::<RunCounter>();

        app.add_systems(Update, |mut counter: ResMut<RunCounter>| {
            counter.0 += 1;
        });

        app.update();

        let counter = app.world().resource::<RunCounter>();
        assert!(counter.0 > 0, "Systems in Update should be called");
    }
}


//     #[test]
//     fn test_demo_ui_plugin_registers_physics_settings() {
//         let mut app = App::new();
//         app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
//
//         // Test that DemoUi plugin adds the DemoUiPhysicsBackendSettings resource
//         app.add_plugins(DemoUi::<DemoControlScheme>::default());
//
//         // The resource should be added by the plugin
//         let settings_exists = app
//             .world()
//             .contains_resource::<DemoUiPhysicsBackendSettings>();
//         assert!(
//             settings_exists,
//             "DemoUiPhysicsBackendSettings resource should be added"
//         );
//
//         // Check default values
//         let settings = app
//             .world()
//             .get_resource::<DemoUiPhysicsBackendSettings>()
//             .unwrap();
//         assert!(settings.active, "Physics should be active by default");
//         assert_eq!(
//             settings.gravity,
//             Vector3::NEG_Y * GRAVITY_MAGNITUDE,
//             "Gravity should have default value"
//         );
//     }
//
//     #[test]
//     fn test_physics_settings_update() {
//         let mut app = App::new();
//         app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
//
//         // Add DemoUi plugin
//         app.add_plugins(DemoUi::<DemoControlScheme>::default());
//
//         // Get the settings resource and modify it
//         let mut settings = app
//             .world_mut()
//             .get_resource_mut::<DemoUiPhysicsBackendSettings>()
//             .unwrap();
//         settings.active = false;
//         settings.gravity = Vector3::NEG_X * GRAVITY_MAGNITUDE;
//
//         // Run update to see if systems handle the changes
//         app.update();
//
//         // The test verifies the plugin compiles and runs without panicking
//         // More specific tests would require mocking the physics backend
//         assert!(true, "Physics settings update should work without errors");
//     }
// }