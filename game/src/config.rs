use serde::Deserialize;
use std::fmt;

use bevy::app::{App, Plugin, PreStartup, Update};
use bevy::asset::io::Reader;
use bevy::asset::{
    Asset, AssetApp, AssetEvent, AssetLoader, AssetServer, Assets, Handle, LoadContext,
};
use bevy::ecs::message::MessageReader;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, Res};
use bevy::log::info;
use bevy::reflect::TypePath;

use crate::global::*;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Config>()
            .init_asset_loader::<ConfigLoader>()
            // ALWAYS load config before the game actually start
            .add_systems(PreStartup, load_config_from_file)
            // print config on reload to debug -> Can we only build on dev build?
            .add_systems(Update, print_config);
    }
}

/// Return the current config from file.
/// However, when the config is still loading, return the default.
pub fn current_config(handle: &ConfigHandle, assets: &Assets<Config>) -> Config {
    assets.get(&handle.0).copied().unwrap_or_default()
}

fn load_config_from_file(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ConfigHandle(asset_server.load(CONFIG_PATH)));
}

fn print_config(
    mut events: MessageReader<AssetEvent<Config>>,
    handle: Res<ConfigHandle>,
    configs: Res<Assets<Config>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } if *id == handle.0.id() => {
                let config = current_config(&handle, &configs);
                info!("config = {config:#?}");
            }
            _ => {}
        }
    }
}

/// Values that need to be tweaked often in dev build.
/// Bevy will hot-reload values after the config file is saved.
#[derive(Asset, TypePath, Deserialize, Clone, Copy, Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

#[derive(Resource)]
pub struct ConfigHandle(pub Handle<Config>);

#[derive(Default, TypePath)]
pub struct ConfigLoader;

#[derive(Debug)]
pub enum ConfigLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
}

impl fmt::Display for ConfigLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigLoaderError::Io(e) => write!(f, "could not read settings file: {e}"),
            ConfigLoaderError::Ron(e) => write!(f, "could not parse settings RON: {e}"),
        }
    }
}

impl std::error::Error for ConfigLoaderError {}

impl From<std::io::Error> for ConfigLoaderError {
    fn from(e: std::io::Error) -> Self {
        ConfigLoaderError::Io(e)
    }
}

impl From<ron::error::SpannedError> for ConfigLoaderError {
    fn from(e: ron::error::SpannedError) -> Self {
        ConfigLoaderError::Ron(e)
    }
}

impl AssetLoader for ConfigLoader {
    type Asset = Config;
    type Settings = ();
    type Error = ConfigLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes::<Config>(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
