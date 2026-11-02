use bevy::prelude::*;

use crate::global::*;

struct SpriteSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl SpriteSheet {
    pub fn load(asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>, path: &'static str, rows: u32, columns: u32) -> Self {
    }
    pub fn sprite(&self, index: usize) -> Sprite {
    }
}

impl FromWorld for LevelSpriteSheet {
  fn from_world(world: &mut World) -> Self {
    let image = world.resource::<AssetServer>().load(LEVEL_SPRITESHEET_PATH);

    let layout = TextureAtlasLayout::from_grid(
      UVec2::splat(TILE_SIZE), // Size of a tile
      4,                // Columns
      4,                // Rows
      None,             // Padding between frames
      None,             // Offset from the sheet's edge
    );

    let layout = world
      .resource_mut::<Assets<TextureAtlasLayout>>()
      .add(layout);

    Self { image, layout }
  }
}

// #[derive(Resource, Debug)]
// pub struct SpriteSheet {
//     description:str,
//     path: str,
// }
//
// #[derive(Resource, Debug)]
// pub struct TileSprite {
//     spritesheet: &SpriteSheet,
//     topLeft: UVec2, // yet another coordinate system
//     bottomRight: UVec2,
// }
