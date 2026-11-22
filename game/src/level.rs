use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Index, IndexMut, Neg, Sub};

use strum::{EnumIter, IntoEnumIterator};

use bevy::prelude::{
    App, AppExtStates, Commands, Component, DespawnOnExit, FromWorld, Name, OnEnter, Plugin, Res,
    Resource, State, SubStates, Transform, Vec2, Visibility, World,
};
use bevy::sprite::Anchor;

use crate::game_state::{GameState, GameStatePlugin};
use crate::global;
use crate::spritesheet::SpriteSheet;
use crate::spritesheet::SpriteSheets;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            app.is_plugin_added::<GameStatePlugin>(),
            "LevelPlugin requires GameStatePlugin"
        );

        app.init_resource::<Levels>()
            .add_sub_state::<Level>();
        for lvl in Level::iter() {
            app.add_systems(OnEnter(lvl), spawn_level);
        };
    }
}

fn spawn_level(
    mut commands: Commands,
    sheets: Res<SpriteSheets>,
    levels: Res<Levels>,
    current_level: Res<State<Level>>,
) {
    let lvl = *current_level.get();
    let grid = levels.get(lvl);
    let sheet = sheets.get(SpriteSheet::Level);
    commands
        .spawn((LevelRoot, DespawnOnExit(lvl)))
        .with_children(|level| {
            for (_, pos, tile) in grid.iter() {
                level.spawn((
                    tile,
                    pos,
                    sheet.sprite(tile.index_in_tile_spritesheet()),
                    Transform::from_translation(pos.to_xy().extend(global::LEVEL_Z)),
                    Anchor::BOTTOM_LEFT,
                    Name::new(format!("Tile({}, {})", pos.row, pos.col)),
                ));
            }
        });
    return;
}

/// The `(row, col)` position of a Tile in the TileGrid.
#[derive(Clone, Copy, Debug, Component, Eq, PartialEq, Default)]
pub struct GridVec2 {
    pub row: i32,
    pub col: i32,
}

impl GridVec2 {
    pub const ZERO: Self = Self { row: 0, col: 0 };
    pub const DOWN: Self = Self { row: -1, col: 0 };
    pub const DOWN_LEFT: Self = Self { row: -1, col: -1 };
    pub const LEFT: Self = Self { row: 0, col: -1 };
    pub const UP_LEFT: Self = Self { row: 1, col: -1 };
    pub const UP: Self = Self { row: 1, col: 0 };
    pub const UP_RIGHT: Self = Self { row: 1, col: 1 };
    pub const RIGHT: Self = Self { row: 0, col: 1 };
    pub const DOWN_RIGHT: Self = Self { row: -1, col: 1 };
    pub const VALID_MOVES: [GridVec2; 8] = [
        GridVec2::DOWN,
        GridVec2::DOWN_RIGHT,
        GridVec2::LEFT,
        GridVec2::UP_LEFT,
        GridVec2::UP,
        GridVec2::UP_RIGHT,
        GridVec2::RIGHT,
        GridVec2::DOWN_RIGHT,
    ];

    pub fn new(row: i32, col: i32) -> Self {
        return GridVec2 { row, col };
    }

    /// Return `(x, y)` (world) coordinates of the cell's anchor
    /// (its bottom-left corner).
    pub fn to_xy(self) -> Vec2 {
        return Vec2::new(
            self.col as f32 * global::TILE_SIZE_PIXELS as f32,
            self.row as f32 * global::TILE_SIZE_PIXELS as f32,
        );
    }

    /// Translate the original GridPos to a new coordinate.
    pub fn translate(self, delta_row: i32, delta_col: i32) -> GridVec2 {
        GridVec2 {
            row: self.row + delta_row,
            col: self.col + delta_col,
        }
    }

    pub fn abs(self) -> GridVec2 {
        let r = if self.row < 0 { -self.row } else { self.row };
        let c = if self.col < 0 { -self.col } else { self.col };
        GridVec2 { row: r, col: c }
    }
}

impl fmt::Display for GridVec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(r{}, c{})", self.row, self.col)
    }
}

impl Add<GridVec2> for GridVec2 {
    type Output = GridVec2;
    fn add(self, d: GridVec2) -> GridVec2 {
        return GridVec2 {
            row: self.row + d.row,
            col: self.col + d.col,
        };
    }
}

impl Neg for GridVec2 {
    type Output = GridVec2;
    fn neg(self) -> GridVec2 {
        return GridVec2 {
            row: -self.row,
            col: -self.col,
        };
    }
}

impl Sub<GridVec2> for GridVec2 {
    type Output = GridVec2;
    fn sub(self, d: GridVec2) -> GridVec2 {
        return GridVec2 {
            row: self.row - d.row,
            col: self.col - d.col,
        };
    }
}

pub type GridPos = GridVec2;

pub type GridDelta = GridVec2;

/// A Tile.
#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub enum Tile {
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
pub struct TileGrid {
    // row/col index of the grid's bounds:
    // bottom <= row <= top, left <= col <= right
    bottom: i32,
    left: i32,
    top: i32,
    right: i32,
    // (row, col) -> tiles[row*cols + col]
    tiles: Vec<Tile>,
    start: GridPos,
    end: GridPos,
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
            start: GridPos {
                // change with set_start()
                row: i32::MIN,
                col: i32::MIN,
            },
            end: GridPos {
                // change with set_end()
                row: i32::MAX,
                col: i32::MAX,
            },
        };

        return Ok(grid);
    }

    fn set_start(mut self, pos: GridPos) -> Self {
        self[pos] = Tile::MazeStart;
        self.start = pos;

        return self;
    }

    fn set_end(mut self, pos: GridPos) -> Self {
        self[pos] = Tile::MazeEnd;
        self.end = pos;

        return self;
    }

    pub fn start(&self) -> GridPos {
        self.start
    }

    pub fn end(&self) -> GridPos {
        self.end
    }

    fn rows(&self) -> usize {
        return (self.top - self.bottom + 1) as usize;
    }

    fn cols(&self) -> usize {
        return (self.right - self.left + 1) as usize;
    }

    pub fn contains(&self, pos: GridVec2) -> bool {
        return (self.bottom..=self.top).contains(&pos.row)
            && (self.left..=self.right).contains(&pos.col);
    }

    /// `GridPos` -> index into `tiles`, or `None` (out of bounds).
    fn flat_index(&self, pos: GridVec2) -> Option<usize> {
        if !self.contains(pos) {
            return None;
        }
        let row = (pos.row - self.bottom) as usize;
        let col = (pos.col - self.left) as usize;
        return Some(row * self.cols() + col);
    }

    /// Returns the tile at a `GridPos`, or `None` (out of bounds).
    pub fn get(&self, pos: GridVec2) -> Option<&Tile> {
        return self.flat_index(pos).map(|i| &self.tiles[i]);
    }

    /// Returns a mutable reference to the tile at a `GridPos`, or `None` (out of bounds).
    pub fn get_mut(&mut self, pos: GridVec2) -> Option<&mut Tile> {
        let i = self.flat_index(pos)?;
        return Some(&mut self.tiles[i]);
    }

    /// Iterator for looping through all tiles.
    fn iter(&self) -> impl Iterator<Item = (usize, GridVec2, Tile)> + '_ {
        return (self.bottom..=self.top).flat_map(move |row| {
            (self.left..=self.right).map(move |col| {
                let pos = GridVec2 { row, col };
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
            Level::_2 => Self::lvl_2(),
            Level::_3 => Self::lvl_3(),
            Level::_4 => Self::lvl_4(),
        }
    }

    fn lvl_0() -> Self {
        let grid = TileGrid::new(-5, -9, 5, 9)
            .unwrap()
            .set_start(GridVec2::new(-2, -2))
            .set_end(GridVec2::new(2, 2));

        return grid;
    }

    fn lvl_1() -> Self {
        let mut grid = TileGrid::new(-1, -5, 1, 5)
            .unwrap()
            .set_start(GridVec2::new(0, -4))
            .set_end(GridVec2::new(0, 4));

        let bottom = grid.bottom;
        let top = grid.top;
        let left = grid.left;
        let right = grid.right;
        for c in grid.left..=grid.right {
            grid[GridVec2::new(bottom, c)] = Tile::Wall;
            grid[GridVec2::new(top, c)] = Tile::Wall;
        }
        for r in grid.bottom..=grid.top {
            grid[GridVec2::new(r, left)] = Tile::Wall;
            grid[GridVec2::new(r, right)] = Tile::Wall;
        }

        return grid;
    }

    fn lvl_2() -> Self {
        let mut grid = TileGrid::new(-5, -9, 5, 9)
            .unwrap()
            .set_start(GridVec2::new(-4, -8))
            .set_end(GridVec2::new(4, 8));

        let bottom = grid.bottom;
        let top = grid.top;
        let left = grid.left;
        let right = grid.right;
        for c in grid.left..=grid.right {
            grid[GridVec2::new(bottom, c)] = Tile::Wall;
            grid[GridVec2::new(top, c)] = Tile::Wall;
        }
        for r in grid.bottom..=grid.top {
            grid[GridVec2::new(r, left)] = Tile::Wall;
            grid[GridVec2::new(r, right)] = Tile::Wall;
        }
        grid[GridPos::new(-4, -3)] = Tile::Wall;
        grid[GridPos::new(-3, -7)] = Tile::Wall;
        grid[GridPos::new(-3, -6)] = Tile::Wall;
        grid[GridPos::new(-3, -5)] = Tile::Wall;
        grid[GridPos::new(-3, -4)] = Tile::Wall;
        grid[GridPos::new(-3, -3)] = Tile::Wall;
        grid[GridPos::new(-3, -1)] = Tile::Wall;
        grid[GridPos::new(-3, 1)] = Tile::Wall;
        grid[GridPos::new(-3, 3)] = Tile::Wall;
        grid[GridPos::new(-3, 4)] = Tile::Wall;
        grid[GridPos::new(-3, 5)] = Tile::Wall;
        grid[GridPos::new(-3, 6)] = Tile::Wall;
        grid[GridPos::new(-3, 7)] = Tile::Wall;
        grid[GridPos::new(-3, 8)] = Tile::Wall;
        grid[GridPos::new(-2, -1)] = Tile::Wall;
        grid[GridPos::new(-2, 1)] = Tile::Wall;
        grid[GridPos::new(-2, 5)] = Tile::Wall;
        grid[GridPos::new(-1, -7)] = Tile::Wall;
        grid[GridPos::new(-1, -5)] = Tile::Wall;
        grid[GridPos::new(-1, -3)] = Tile::Wall;
        grid[GridPos::new(-1, -2)] = Tile::Wall;
        grid[GridPos::new(-1, -1)] = Tile::Wall;
        grid[GridPos::new(-1, 0)] = Tile::Wall;
        grid[GridPos::new(-1, 1)] = Tile::Wall;
        grid[GridPos::new(-1, 2)] = Tile::Wall;
        grid[GridPos::new(-1, 3)] = Tile::Wall;
        grid[GridPos::new(-1, 5)] = Tile::Wall;
        grid[GridPos::new(-1, 7)] = Tile::Wall;
        grid[GridPos::new(-1, 8)] = Tile::Wall;
        grid[GridPos::new(0, -7)] = Tile::Wall;
        grid[GridPos::new(0, -5)] = Tile::Wall;
        grid[GridPos::new(0, 3)] = Tile::Wall;
        grid[GridPos::new(1, -7)] = Tile::Wall;
        grid[GridPos::new(1, -5)] = Tile::Wall;
        grid[GridPos::new(1, -3)] = Tile::Wall;
        grid[GridPos::new(1, -2)] = Tile::Wall;
        grid[GridPos::new(1, -1)] = Tile::Wall;
        grid[GridPos::new(1, 1)] = Tile::Wall;
        grid[GridPos::new(1, 2)] = Tile::Wall;
        grid[GridPos::new(1, 3)] = Tile::Wall;
        grid[GridPos::new(1, 4)] = Tile::Wall;
        grid[GridPos::new(1, 5)] = Tile::Wall;
        grid[GridPos::new(1, 7)] = Tile::Wall;
        grid[GridPos::new(1, 8)] = Tile::Wall;
        grid[GridPos::new(2, -7)] = Tile::Wall;
        grid[GridPos::new(2, -5)] = Tile::Wall;
        grid[GridPos::new(2, -3)] = Tile::Wall;
        grid[GridPos::new(2, 5)] = Tile::Wall;
        grid[GridPos::new(2, 7)] = Tile::Wall;
        grid[GridPos::new(3, -7)] = Tile::Wall;
        grid[GridPos::new(3, -6)] = Tile::Wall;
        grid[GridPos::new(3, -5)] = Tile::Wall;
        grid[GridPos::new(3, -3)] = Tile::Wall;
        grid[GridPos::new(3, -2)] = Tile::Wall;
        grid[GridPos::new(3, -1)] = Tile::Wall;
        grid[GridPos::new(3, 1)] = Tile::Wall;
        grid[GridPos::new(3, 3)] = Tile::Wall;
        grid[GridPos::new(3, 4)] = Tile::Wall;
        grid[GridPos::new(3, 5)] = Tile::Wall;
        grid[GridPos::new(3, 7)] = Tile::Wall;
        grid[GridPos::new(4, -5)] = Tile::Wall;
        grid[GridPos::new(4, -1)] = Tile::Wall;
        grid[GridPos::new(4, 1)] = Tile::Wall;
        grid[GridPos::new(4, 5)] = Tile::Wall;

        return grid;
    }

    fn lvl_3() -> Self {
        let mut grid = TileGrid::new(-5, -9, 5, 9)
            .unwrap()
            .set_start(GridVec2::new(-4, -8))
            .set_end(GridVec2::new(4, 8));

        let bottom = grid.bottom;
        let top = grid.top;
        let left = grid.left;
        let right = grid.right;
        for c in grid.left..=grid.right {
            grid[GridVec2::new(bottom, c)] = Tile::Wall;
            grid[GridVec2::new(top, c)] = Tile::Wall;
        }
        for r in grid.bottom..=grid.top {
            grid[GridVec2::new(r, left)] = Tile::Wall;
            grid[GridVec2::new(r, right)] = Tile::Wall;
        }
        grid[GridPos::new(-4, -1)] = Tile::Wall;
        grid[GridPos::new(-4, 5)] = Tile::Wall;
        grid[GridPos::new(-3, -8)] = Tile::Wall;
        grid[GridPos::new(-3, -7)] = Tile::Wall;
        grid[GridPos::new(-3, -5)] = Tile::Wall;
        grid[GridPos::new(-3, -4)] = Tile::Wall;
        grid[GridPos::new(-3, -3)] = Tile::Wall;
        grid[GridPos::new(-3, -2)] = Tile::Wall;
        grid[GridPos::new(-3, -1)] = Tile::Wall;
        grid[GridPos::new(-3, 0)] = Tile::Wall;
        grid[GridPos::new(-3, 1)] = Tile::Wall;
        grid[GridPos::new(-3, 3)] = Tile::Wall;
        grid[GridPos::new(-3, 4)] = Tile::Wall;
        grid[GridPos::new(-3, 5)] = Tile::Wall;
        grid[GridPos::new(-3, 7)] = Tile::Wall;
        grid[GridPos::new(-3, 8)] = Tile::Wall;
        grid[GridPos::new(-2, -5)] = Tile::Wall;
        grid[GridPos::new(-2, -1)] = Tile::Wall;
        grid[GridPos::new(-1, -8)] = Tile::Wall;
        grid[GridPos::new(-1, -7)] = Tile::Wall;
        grid[GridPos::new(-1, -5)] = Tile::Wall;
        grid[GridPos::new(-1, -3)] = Tile::Wall;
        grid[GridPos::new(-1, -1)] = Tile::Wall;
        grid[GridPos::new(-1, 1)] = Tile::Wall;
        grid[GridPos::new(-1, 2)] = Tile::Wall;
        grid[GridPos::new(-1, 3)] = Tile::Wall;
        grid[GridPos::new(-1, 4)] = Tile::Wall;
        grid[GridPos::new(-1, 5)] = Tile::Wall;
        grid[GridPos::new(-1, 6)] = Tile::Wall;
        grid[GridPos::new(-1, 7)] = Tile::Wall;
        grid[GridPos::new(0, -3)] = Tile::Wall;
        grid[GridPos::new(0, -1)] = Tile::Wall;
        grid[GridPos::new(0, 5)] = Tile::Wall;
        grid[GridPos::new(1, -7)] = Tile::Wall;
        grid[GridPos::new(1, -6)] = Tile::Wall;
        grid[GridPos::new(1, -5)] = Tile::Wall;
        grid[GridPos::new(1, -4)] = Tile::Wall;
        grid[GridPos::new(1, -3)] = Tile::Wall;
        grid[GridPos::new(1, -1)] = Tile::Wall;
        grid[GridPos::new(1, 0)] = Tile::Wall;
        grid[GridPos::new(1, 1)] = Tile::Wall;
        grid[GridPos::new(1, 3)] = Tile::Wall;
        grid[GridPos::new(1, 5)] = Tile::Wall;
        grid[GridPos::new(1, 6)] = Tile::Wall;
        grid[GridPos::new(1, 7)] = Tile::Wall;
        grid[GridPos::new(2, -5)] = Tile::Wall;
        grid[GridPos::new(2, 3)] = Tile::Wall;
        grid[GridPos::new(2, 5)] = Tile::Wall;
        grid[GridPos::new(3, -7)] = Tile::Wall;
        grid[GridPos::new(3, -5)] = Tile::Wall;
        grid[GridPos::new(3, -4)] = Tile::Wall;
        grid[GridPos::new(3, -3)] = Tile::Wall;
        grid[GridPos::new(3, -1)] = Tile::Wall;
        grid[GridPos::new(3, 0)] = Tile::Wall;
        grid[GridPos::new(3, 1)] = Tile::Wall;
        grid[GridPos::new(3, 3)] = Tile::Wall;
        grid[GridPos::new(3, 4)] = Tile::Wall;
        grid[GridPos::new(3, 5)] = Tile::Wall;
        grid[GridPos::new(3, 6)] = Tile::Wall;
        grid[GridPos::new(3, 7)] = Tile::Wall;
        grid[GridPos::new(4, -7)] = Tile::Wall;
        grid[GridPos::new(4, -3)] = Tile::Wall;
        grid[GridPos::new(4, -1)] = Tile::Wall;
        grid[GridPos::new(4, 7)] = Tile::Wall;

        return grid;
    }

    fn lvl_4() -> Self {
        let mut grid = TileGrid::new(-5, -9, 5, 9)
            .unwrap()
            .set_start(GridVec2::new(-4, -8))
            .set_end(GridVec2::new(4, 8));

        let bottom = grid.bottom;
        let top = grid.top;
        let left = grid.left;
        let right = grid.right;
        for c in grid.left..=grid.right {
            grid[GridVec2::new(bottom, c)] = Tile::Wall;
            grid[GridVec2::new(top, c)] = Tile::Wall;
        }
        for r in grid.bottom..=grid.top {
            grid[GridVec2::new(r, left)] = Tile::Wall;
            grid[GridVec2::new(r, right)] = Tile::Wall;
        }
        grid[GridPos::new(-4, -3)] = Tile::Wall;
        grid[GridPos::new(-4, 5)] = Tile::Wall;
        grid[GridPos::new(-3, -8)] = Tile::Wall;
        grid[GridPos::new(-3, -7)] = Tile::Wall;
        grid[GridPos::new(-3, -6)] = Tile::Wall;
        grid[GridPos::new(-3, -5)] = Tile::Wall;
        grid[GridPos::new(-3, -3)] = Tile::Wall;
        grid[GridPos::new(-3, -1)] = Tile::Wall;
        grid[GridPos::new(-3, 0)] = Tile::Wall;
        grid[GridPos::new(-3, 1)] = Tile::Wall;
        grid[GridPos::new(-3, 2)] = Tile::Wall;
        grid[GridPos::new(-3, 3)] = Tile::Wall;
        grid[GridPos::new(-3, 5)] = Tile::Wall;
        grid[GridPos::new(-3, 6)] = Tile::Wall;
        grid[GridPos::new(-3, 7)] = Tile::Wall;
        grid[GridPos::new(-2, -3)] = Tile::Wall;
        grid[GridPos::new(-2, -1)] = Tile::Wall;
        grid[GridPos::new(-2, 3)] = Tile::Wall;
        grid[GridPos::new(-2, 5)] = Tile::Wall;
        grid[GridPos::new(-1, -7)] = Tile::Wall;
        grid[GridPos::new(-1, -6)] = Tile::Wall;
        grid[GridPos::new(-1, -5)] = Tile::Wall;
        grid[GridPos::new(-1, -4)] = Tile::Wall;
        grid[GridPos::new(-1, -3)] = Tile::Wall;
        grid[GridPos::new(-1, -1)] = Tile::Wall;
        grid[GridPos::new(-1, 0)] = Tile::Wall;
        grid[GridPos::new(-1, 1)] = Tile::Wall;
        grid[GridPos::new(-1, 3)] = Tile::Wall;
        grid[GridPos::new(-1, 5)] = Tile::Wall;
        grid[GridPos::new(-1, 7)] = Tile::Wall;
        grid[GridPos::new(0, -7)] = Tile::Wall;
        grid[GridPos::new(0, -1)] = Tile::Wall;
        grid[GridPos::new(0, 3)] = Tile::Wall;
        grid[GridPos::new(0, 7)] = Tile::Wall;
        grid[GridPos::new(1, -7)] = Tile::Wall;
        grid[GridPos::new(1, -5)] = Tile::Wall;
        grid[GridPos::new(1, -4)] = Tile::Wall;
        grid[GridPos::new(1, -3)] = Tile::Wall;
        grid[GridPos::new(1, -2)] = Tile::Wall;
        grid[GridPos::new(1, -1)] = Tile::Wall;
        grid[GridPos::new(1, 1)] = Tile::Wall;
        grid[GridPos::new(1, 2)] = Tile::Wall;
        grid[GridPos::new(1, 3)] = Tile::Wall;
        grid[GridPos::new(1, 4)] = Tile::Wall;
        grid[GridPos::new(1, 5)] = Tile::Wall;
        grid[GridPos::new(1, 6)] = Tile::Wall;
        grid[GridPos::new(1, 7)] = Tile::Wall;
        grid[GridPos::new(2, -7)] = Tile::Wall;
        grid[GridPos::new(2, -5)] = Tile::Wall;
        grid[GridPos::new(2, -1)] = Tile::Wall;
        grid[GridPos::new(3, -7)] = Tile::Wall;
        grid[GridPos::new(3, -5)] = Tile::Wall;
        grid[GridPos::new(3, -3)] = Tile::Wall;
        grid[GridPos::new(3, -1)] = Tile::Wall;
        grid[GridPos::new(3, 1)] = Tile::Wall;
        grid[GridPos::new(3, 2)] = Tile::Wall;
        grid[GridPos::new(3, 3)] = Tile::Wall;
        grid[GridPos::new(3, 4)] = Tile::Wall;
        grid[GridPos::new(3, 5)] = Tile::Wall;
        grid[GridPos::new(3, 6)] = Tile::Wall;
        grid[GridPos::new(3, 7)] = Tile::Wall;
        grid[GridPos::new(3, 8)] = Tile::Wall;
        grid[GridPos::new(4, -5)] = Tile::Wall;
        grid[GridPos::new(4, -3)] = Tile::Wall;

        return grid;
    }
}

impl Index<GridVec2> for TileGrid {
    type Output = Tile;

    fn index(&self, pos: GridVec2) -> &Tile {
        return self
            .get(pos)
            .unwrap_or_else(|| panic!("{pos:?} out of grid bounds"));
    }
}

impl IndexMut<GridVec2> for TileGrid {
    fn index_mut(&mut self, pos: GridVec2) -> &mut Tile {
        return self
            .get_mut(pos)
            .unwrap_or_else(|| panic!("{pos:?} out of grid bounds"));
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct LevelRoot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SubStates, Default, EnumIter)]
#[source(GameState = GameState::Intro | GameState::Playing)]
pub enum Level {
    _0,
    #[default]
    _1,
    _2,
    _3,
    _4,
}

impl Level {
    pub const PLAYABLE: [Level; 3] = [Level::_2, Level::_3, Level::_4];
}

// fn on_maze_end_reached(mut next: ResMut<NextState<Level>>) {
//     next.set(Level::_0);
// }

/// All Level(s) used in-game.
#[derive(Resource)]
pub struct Levels(HashMap<Level, TileGrid>);

impl Levels {
    pub fn get(&self, lvl: Level) -> &TileGrid {
        return &self.0[&lvl];
    }
}

impl FromWorld for Levels {
    fn from_world(_: &mut World) -> Self {
        let mut levels = HashMap::new();
        for lvl in Level::iter() {
            levels.insert(lvl, TileGrid::for_level(lvl));
        }

        return Levels(levels);
    }
}
