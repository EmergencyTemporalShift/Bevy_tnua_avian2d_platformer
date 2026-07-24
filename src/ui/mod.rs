pub mod component_alteration;
pub mod info;
mod level_selection;
pub mod tuning;

#[cfg(feature = "egui")]
mod framerate;
#[cfg(feature = "egui")]
pub mod plotting;

pub mod components;
pub mod plugin;
pub mod systems;

// Re-export DemoUi so it can be accessed directly as `ui::DemoUi`
pub use plugin::DemoUi;