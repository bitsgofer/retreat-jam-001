use std::collections::HashMap;

use bevy::prelude::{
    App, AssetServer, Assets, FromWorld, Handle, Image, Plugin, Resource, TextureAtlasLayout,
    UVec2, World,
};

use crate::global;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>();
    }
}

/// The `(row, col)` position of a Tile in the TileGrid.
#[derive(Clone, Copy, Debug)]
struct GridPos {
    row: i32,
    col: i32,
}

impl Default for GridPos {
    fn default() -> Self {
        return GridPos::new(0, 0);
    }
}

impl GridPos {
    fn new(row: i32, col: i32) -> Self {
        return GridPos { row, col };
    }
}

/// A Tile.
#[derive(Clone, Copy, Debug)]
enum Tile {
    Floor,
    Wall,
}

/// The SpriteSheet we are using.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SpriteSheet {
    Level,
}

impl SpriteSheet {
    fn as_str(&self) -> &'static str {
        match self {
            SpriteSheet::Level => global::LEVEL_SPRITESHEET_PATH,
        }
    }
}

/// An image with multiple objects.
struct SpriteSheetHandle {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl SpriteSheetHandle {
    /// The Sprite at `index`.
    fn sprite(&self, index: usize) -> Sprite {
        return Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index,
            },
        );
    }
}

/// All SpriteSheet(s) used in-game.
#[derive(Resource)]
struct SpriteSheets(HashMap<SpriteSheet, SpriteSheetHandle>);

impl SpriteSheets {
    fn get(&self, kind: SpriteSheet) -> &SpriteSheetHandle {
        return &self.0[&kind];
    }
}

impl FromWorld for SpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let mut sheets = HashMap::new();

        sheets.insert(
            SpriteSheet::Level,
            SpriteSheetHandle {
                image: asset_server.load(SpriteSheet::Level.as_str()),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(global::TILE_SIZE_PIXELS),
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
