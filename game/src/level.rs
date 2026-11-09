use std::collections::HashMap;
use std::fmt;
use std::ops::{Index, IndexMut};

use bevy::prelude::{
    App, AssetServer, Assets, Commands, Component, FromWorld, Handle, Image, Name, Plugin, Res,
    Resource, Sprite, Startup, TextureAtlas, TextureAtlasLayout, Transform, UVec2, Vec2,
    Visibility, World,
};

use crate::global;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheets>()
            .init_resource::<Levels>()
            .add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands, sheets: Res<SpriteSheets>, levels: Res<Levels>) {
    let lvl = Level::_1;
    let grid = levels.get(lvl);
    let sheet = sheets.get(SpriteSheet::Level);
    commands.spawn(lvl).with_children(|level| {
        for (_, pos, tile) in grid.iter() {
            level.spawn((
                tile,
                pos,
                sheet.sprite(tile.index_in_tile_spritesheet()),
                Transform::from_translation(pos.to_xy().extend(0.0)),
                bevy::sprite::Anchor::BOTTOM_LEFT,
                Name::new(format!("Tile({}, {})", pos.row, pos.col)),
            ));
        }
    });
    return;
}

/// The `(row, col)` position of a Tile in the TileGrid.
#[derive(Clone, Copy, Debug, Component)]
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

    /// Return `(x, y)` (world) coordinates of the cell's anchor
    /// (its bottom-left corner).
    fn to_xy(self) -> Vec2 {
        return Vec2::new(
            self.col as f32 * global::TILE_SIZE_PIXELS as f32,
            self.row as f32 * global::TILE_SIZE_PIXELS as f32,
        );
    }
}

/// A Tile.
#[derive(Clone, Copy, Debug, Component)]
enum Tile {
    Floor,
    Wall,
    MazeStart,
    MazeEnd,
}

impl Tile {
    fn index_in_tile_spritesheet(self) -> usize {
        return match self {
            Tile::Floor => 0,
            Tile::Wall => 1,
            Tile::MazeStart => 2,
            Tile::MazeEnd => 3,
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GridError {
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

/// A 2D grid of tiles, representing the layout of the level.
#[derive(Debug)]
struct TileGrid {
    // row/col index of the grid's bounds:
    // bottom <= row <= top, left <= col <= right
    bottom: i32,
    left: i32,
    top: i32,
    right: i32,
    // (row, col) -> tiles[row*cols + col]
    tiles: Vec<Tile>,
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
        let grid = TileGrid {
            bottom,
            left,
            top,
            right,
            tiles: vec![Tile::Floor; rows * cols],
        };

        return Ok(grid);
    }

    fn rows(&self) -> usize {
        return (self.top - self.bottom + 1) as usize;
    }

    fn cols(&self) -> usize {
        return (self.right - self.left + 1) as usize;
    }

    fn contains(&self, pos: GridPos) -> bool {
        return (self.bottom..=self.top).contains(&pos.row)
            && (self.left..=self.right).contains(&pos.col);
    }

    /// `GridPos` -> index into `tiles`, or `None` (out of bounds).
    fn flat_index(&self, pos: GridPos) -> Option<usize> {
        if !self.contains(pos) {
            return None;
        }
        let row = (pos.row - self.bottom) as usize;
        let col = (pos.col - self.left) as usize;
        return Some(row * self.cols() + col);
    }

    /// Returns the tile at a `GridPos`, or `None` (out of bounds).
    fn get(&self, pos: GridPos) -> Option<&Tile> {
        return self.flat_index(pos).map(|i| &self.tiles[i]);
    }

    /// Returns a mutable reference to the tile at a `GridPos`, or `None` (out of bounds).
    fn get_mut(&mut self, pos: GridPos) -> Option<&mut Tile> {
        let i = self.flat_index(pos)?;
        return Some(&mut self.tiles[i]);
    }

    /// Iterator for looping through all tiles.
    fn iter(&self) -> impl Iterator<Item = (usize, GridPos, Tile)> + '_ {
        return (self.bottom..=self.top).flat_map(move |row| {
            (self.left..=self.right).map(move |col| {
                let pos = GridPos::new(row, col);
                let index = self.flat_index(pos).unwrap();
                (index, pos, self.tiles[index])
            })
        });
    }
}

impl TileGrid {
    fn for_level(lvl: Level) -> Self {
        match lvl {
            Level::_0 => Self::lvl_0(),
            Level::_1 => Self::lvl_1(),
        }
    }

    fn lvl_0() -> Self {
        let grid = TileGrid::new(-5, -9, 5, 9).unwrap();

        return grid;
    }

    fn lvl_1() -> Self {
        let mut grid = TileGrid::new(-1, -5, 1, 5).unwrap();
        let bottom = grid.bottom;
        let top = grid.top;
        let left = grid.left;
        let right = grid.right;
        for c in grid.left..=grid.right {
            grid[GridPos::new(bottom, c)] = Tile::Wall;
            grid[GridPos::new(top, c)] = Tile::Wall;
        }
        for r in grid.bottom..=grid.top {
            grid[GridPos::new(r, left)] = Tile::Wall;
            grid[GridPos::new(r, right)] = Tile::Wall;
        }
        grid[GridPos::new(0, -4)] = Tile::MazeStart;
        grid[GridPos::new(0, 4)] = Tile::MazeEnd;

        return grid;
    }
}

impl Index<GridPos> for TileGrid {
    type Output = Tile;

    fn index(&self, pos: GridPos) -> &Tile {
        return self
            .get(pos)
            .unwrap_or_else(|| panic!("{pos:?} out of grid bounds"));
    }
}

impl IndexMut<GridPos> for TileGrid {
    fn index_mut(&mut self, pos: GridPos) -> &mut Tile {
        return self
            .get_mut(pos)
            .unwrap_or_else(|| panic!("{pos:?} out of grid bounds"));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
#[require(Transform, Visibility)]
enum Level {
    _0,
    _1,
}

/// All Level(s) used in-game.
#[derive(Resource)]
struct Levels(HashMap<Level, TileGrid>);

impl Levels {
    fn get(&self, lvl: Level) -> &TileGrid {
        return &self.0[&lvl];
    }
}

impl FromWorld for Levels {
    fn from_world(_: &mut World) -> Self {
        let mut levels = HashMap::new();
        let to_init = vec![Level::_0, Level::_1];
        for lvl in to_init {
            levels.insert(lvl, TileGrid::for_level(lvl));
        }

        return Levels(levels);
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
