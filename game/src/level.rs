use std::collections::HashMap;

use bevy::prelude::{
    App, AssetServer, Assets, Commands, Component, FromWorld, Handle, Image, Plugin, Res, Resource,
    Sprite, Startup, TextureAtlas, TextureAtlasLayout, Transform, UVec2, Vec2, Visibility, World,
};

use crate::global::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>()
            .add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands, sheets: Res<SpriteSheets>) {
    let sheet = sheets.get(SpriteSheetKind::Level);

    const bottom_left: GridPos = GridPos { row: -1, col: -4 };
    const top_right: GridPos = GridPos { row: 1, col: 4 };

    fn is_border(pos: GridPos) -> bool {
        return pos.row == bottom_left.row
            || pos.row == top_right.row
            || pos.col == bottom_left.col
            || pos.col == top_right.col;
    }

    commands.spawn(Level).with_children(|level| {
        level.spawn(TileGrid).with_children(|grid| {
            for row in bottom_left.row..=top_right.row {
                for col in bottom_left.col..=top_right.col {
                    let pos = GridPos::new(row, col);
                    let kind = if is_border(pos) {
                        TileKind::Wall
                    } else {
                        TileKind::Floor
                    };
                    grid.spawn((
                        Tile { kind },
                        pos,
                        sheet.sprite(kind.index_in_spritesheet()),
                        Transform::from_translation(pos.to_xy().extend(0.0)),
                    ));
                }
            }
        });
    });
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

impl TileKind {
    fn index_in_spritesheet(self) -> usize {
        return match self {
            TileKind::Floor => 0,
            TileKind::Wall => 1,
        };
    }
}

/// A Tile within the TileGrid.
#[derive(Component, Debug)]
#[require(GridPos, Transform, Visibility, Sprite)]
struct Tile {
    kind: TileKind,
}

/// The `(row, col)` of a cell inside a grid.
///
/// **IMPORTANT!**: Pay attention to the order of `(row, col)` vs `(x, y)`.
/// - For a grid, we tend to go vertical (row) first, then horizontal (col).
/// - But in a xy grid, we tend to go horizontal (x) first, then horizontal (y).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct GridPos {
    /// Vertical index, map to Y-coordinate.
    pub row: i32,
    /// Horizontal index, map to X-coordinate.
    pub col: i32,
}

impl Default for GridPos {
    fn default() -> Self {
        return GridPos::new(0, 0);
    }
}

impl GridPos {
    pub fn new(row: i32, col: i32) -> Self {
        return GridPos { row, col };
    }

    /// Return `(x, y)` (world) coordinates of the cell's anchor
    /// (its bottom-left corner).
    pub fn to_xy(self) -> Vec2 {
        return Vec2::new(
            self.col as f32 * TILE_SIZE as f32,
            self.row as f32 * TILE_SIZE as f32,
        );
    }
}

/// What SpriteSheet we are using.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpriteSheetKind {
    Level,
    Player,
}

/// An image with multiple objects.
pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

impl SpriteSheet {
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
                // image: asset_server..load_builder().with_settings(|s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest()).load(LEVEL_SPRITESHEET_PATH),
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
