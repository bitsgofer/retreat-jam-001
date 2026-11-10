use std::collections::HashMap;

use bevy::prelude::{
    App, AssetServer, Assets, FromWorld, Handle, Image, Plugin, Resource, Sprite, TextureAtlas,
    TextureAtlasLayout, UVec2, World,
};

use crate::global;

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>();
    }
}

/// The SpriteSheet we are using.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpriteSheet {
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
pub struct SpriteSheetHandle {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl SpriteSheetHandle {
    /// The Sprite at `index`.
    pub fn sprite(&self, index: usize) -> Sprite {
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
pub struct SpriteSheets(HashMap<SpriteSheet, SpriteSheetHandle>);

impl SpriteSheets {
    pub fn get(&self, kind: SpriteSheet) -> &SpriteSheetHandle {
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
