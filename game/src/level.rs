use std::collections::HashMap;

use bevy::prelude::{
    App, Component, Handle, Image, Plugin, Resource, Sprite, TextureAtlasLayout, Transform,
    Visibility,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        return;
    }
}

/// A single Level.
#[derive(Component, Debug)]
#[require(Transform, Visibility)]
struct Level;

/// A 2D grid of tiles, representing the layout of the a level.
#[derive(Component, Debug)]
#[require(Transform, Visibility)]
struct TileGrid;

/// What a Tile represent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TileKind {
    Floor,
    Wall,
}

/// A Tile within the TileGrid.
#[derive(Component, Debug)]
#[require(GridPos, Transform, Visibility, Sprite)]
struct Tile {
    kind: TileKind,
}

/// The `(row, col)` position of a Tile in the TileGrid.
#[derive(Component, Default, Debug)]
struct GridPos;

/// What SpriteSheet we are using.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SpriteSheetKind {
    Level,
    Player,
}

/// An image with multiple objects.
struct SpriteSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

/// All SpriteSheet(s) used in-game.
#[derive(Resource)]
struct SpriteSheets(HashMap<SpriteSheetKind, SpriteSheet>);
