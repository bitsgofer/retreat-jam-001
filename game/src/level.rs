use std::collections::HashMap;

use bevy::prelude::{
    App, Component, Handle, Image, Plugin, Resource, Sprite, TextureAtlasLayout, Transform,
    Visibility, FromWorld, AssetServer, UVec2, World, Assets,
};

use crate::global::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>();
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
pub struct SpriteSheets(HashMap<SpriteSheetKind, SpriteSheet>);

impl SpriteSheets {
    pub fn get(&self, kind: SpriteSheetKind) -> &SpriteSheet {
        return &self.0[&kind];
    }
}

impl FromWorld for SpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let mut sheets = HashMap::new();

        sheets.insert(
            SpriteSheetKind::Level,
            SpriteSheet {
                image: asset_server.load(LEVEL_SPRITESHEET_PATH),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(TILE_SIZE),
                    4,
                    4,
                    None,
                    None,
                )),
            },
        );

        return SpriteSheets(sheets);
    }
}
