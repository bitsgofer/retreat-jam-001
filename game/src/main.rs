use bevy::camera::ScalingMode;
use bevy::log::{Level, LogPlugin, info};
use bevy::prelude::{
    App, Camera2d, Commands, DefaultPlugins, ImagePlugin, OrthographicProjection, PluginGroup,
    Projection, Startup, Window, WindowPlugin,
};
use bevy::utils::default;

mod global;
mod level;
mod spritesheet;

use crate::level::LevelPlugin;
use crate::spritesheet::SpriteSheetPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::INFO,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "gamejam-001".into(),
                        fit_canvas_to_parent: true, // only affect WASM build
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup_camera)
        .add_plugins(SpriteSheetPlugin)
        .add_plugins(LevelPlugin)
        .run();
}

fn print_copyright() {
    info!("Copyright © 2026 BitsGofer Studio.");
}
fn setup_camera(mut commands: Commands) {
    print_copyright();
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 320.0,
                height: 180.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
