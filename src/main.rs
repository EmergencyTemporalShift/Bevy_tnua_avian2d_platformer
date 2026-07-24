use bevy::prelude::*;

fn main() {
    let default_plugins = DefaultPlugins.set(ImagePlugin::default_nearest());

    #[cfg(feature = "pie")]
    let default_plugins = jackdaw_runtime::maybe_windowless(default_plugins);

    App::new().add_plugins(default_plugins)
    // All gameplay lives in the library crate so the editor can link it too.
    .add_plugins(platformer::GamePlugin)
    .run();
}