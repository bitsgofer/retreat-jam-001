use std::collections::HashMap;
use std::fmt;
use std::ops::RangeInclusive;

use bevy::prelude::{
    App, AssetServer, Assets, Commands, Component, FromWorld, Handle, Image, Plugin, Res, Resource,
    Sprite, Startup, TextureAtlas, TextureAtlasLayout, Transform, UVec2, Vec2, Visibility, World,
};

use crate::global::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>()
            .init_resource::<Levels>()
            .add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands, sheets: Res<SpriteSheets>, levels: Res<Levels>) {
    let sheet = sheets.get(SpriteSheetKind::Level);
    // let kind = LevelKind::StraightHorizontal;
    let kind = LevelKind::EmptyFloor;
    let level = levels.get(kind);

    // `Levels` keeps the data; the entities only show it.
    commands.spawn(kind).with_children(|tiles| {
        for (pos, tile) in level.grid.iter() {
            tiles.spawn((
                tile,
                pos,
                sheet.sprite(tile.kind.index_in_spritesheet()),
                Transform::from_translation(pos.to_xy().extend(0.0)),
            ));
        }
    });
}

/// Level identifier.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[require(Transform, Visibility)] // TODO: why?
enum LevelKind {
    EmptyFloor,
    StraightHorizontal,
}

// All levels
#[derive(Resource)]
pub struct Levels(HashMap<LevelKind, Level>);

impl Levels {
    pub fn get(&self, kind: LevelKind) -> &Level {
        return &self.0[&kind];
    }
}

impl FromWorld for Levels {
    fn from_world(world: &mut World) -> Self {
        let mut levels = HashMap::new();

        levels.insert(
            LevelKind::EmptyFloor,
            Level {
                grid: TileGrid::new(-5, -10, 5, 10).unwrap(),
            },
        );
        levels.insert(
            LevelKind::StraightHorizontal,
            Level {
                grid: TileGrid::straight_horizontal(),
            },
        );

        return Levels(levels);
    }
}

/// A single Level.
#[derive(Component, Debug)]
#[require(Transform, Visibility)]
struct Level {
    grid: TileGrid,
}

/// A 2D grid of tiles, representing the layout of the a level.
#[derive(Component, Debug)]
#[require(Transform, Visibility)]
struct TileGrid {
    // row/col index (inclusive) of where the grid is
    pub bottom: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    // size
    pub rows: usize,
    pub cols: usize,
    //
    pub tiles: Vec<Tile>,
}

impl TileGrid {
    fn new(bottom: i32, left: i32, top: i32, right: i32) -> Result<Self, GridError> {
        if bottom > top {
            return Err(GridError::InvertedRows { bottom, top });
        }
        if left > right {
            return Err(GridError::InvertedCols { left, right });
        }

        let rows = (top - bottom + 1) as usize;
        let cols = (right - left + 1) as usize;
        let mut grid = TileGrid {
            bottom,
            left,
            top,
            right,
            rows,
            cols,
            tiles: vec![
                Tile {
                    kind: TileKind::Floor
                };
                rows * cols
            ],
        };

        return Ok(grid);
    }

    fn contains(&self, pos: GridPos) -> bool {
        let row_range: RangeInclusive<i32> = self.bottom..=self.top;
        let col_range: RangeInclusive<i32> = self.left..=self.right;
        return row_range.contains(&pos.row) && col_range.contains(&pos.col);
    }

    fn index(&self, pos: GridPos) -> Option<usize> {
        if !self.contains(pos) {
            return None;
        }
        let row = (pos.row - self.bottom) as usize;
        let col = (pos.col - self.left) as usize;
        return Some(row * self.cols + col);
    }

    /// Every `(GridPos, Tile)` in the grid, bottom row first.
    fn iter(&self) -> impl Iterator<Item = (GridPos, Tile)> + '_ {
        return (self.bottom..=self.top).flat_map(move |row| {
            (self.left..=self.right).map(move |col| {
                let pos = GridPos::new(row, col);
                (pos, self.tiles[self.index(pos).unwrap()])
            })
        });
    }

    fn straight_horizontal() -> Self {
        const bottom_left: GridPos = GridPos { row: -1, col: -9 };
        const top_right: GridPos = GridPos { row: 1, col: 9 };

        fn is_border(pos: GridPos) -> bool {
            return pos.row == bottom_left.row
                || pos.row == top_right.row
                || pos.col == bottom_left.col
                || pos.col == top_right.col;
        }

        let rows = (top_right.row - bottom_left.row + 1) as usize;
        let cols = (top_right.col - bottom_left.col + 1) as usize;
        let mut grid = TileGrid {
            bottom: bottom_left.row,
            left: bottom_left.col,
            top: top_right.row,
            right: top_right.col,
            rows: rows,
            cols: cols,
            // tiles: Vec::with_capacity(3 as usize * 9 as usize),
            tiles: vec![
                Tile {
                    kind: TileKind::Floor
                };
                rows * cols
            ],
        };

        for row in grid.bottom..=grid.top {
            for col in grid.left..=grid.right {
                let pos = GridPos::new(row, col);
                let kind = if is_border(pos) {
                    TileKind::Wall
                } else {
                    TileKind::Floor
                };
                match grid.index(pos) {
                    None => panic!("bad"),
                    Some(idx) => grid.tiles[idx].kind = kind,
                };
            }
        }

        return grid;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    /// `bottom` > `top`.
    InvertedRows { bottom: i32, top: i32 },
    /// `left` > `right`.
    InvertedCols { left: i32, right: i32 },
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedRows { bottom, top } => write!(f, "bottom({bottom}) > top({top})"),
            Self::InvertedCols { left, right } => write!(f, "left({left}) > right({right})"),
        }
    }
}

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
#[derive(Component, Clone, Copy, Debug)]
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
