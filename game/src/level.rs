use std::collections::HashMap;

use bevy::prelude::{App, Handle, Image, Plugin, Resource, TextureAtlasLayout};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, _: &mut App) {
        return;
    }
}

/// The SpriteSheet we are using.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SpriteSheet {
    Level,
}

impl SpriteSheet {
    fn as_str(&self) -> &'static str {
        match self {
            SpriteSheet::Level => "spritesheets/level.png",
        }
    }
}

/// An image with multiple objects.
struct SpriteSheetHandle {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

/// All SpriteSheet(s) used in-game.
#[derive(Resource)]
struct SpriteSheets(HashMap<SpriteSheet, SpriteSheetHandle>);
