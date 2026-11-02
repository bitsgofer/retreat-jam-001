use std::fmt;
use std::ops::RangeInclusive;

use bevy::prelude::*;

use crate::global::*;

/// The (row, col) location of a cell inside a grid.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos {
    /// Vertical index, map to Y-coordinate.
    pub row: i32,
    /// Horizontal index, map to X-coordinate.
    pub col: i32,
}

impl GridPos {
    pub fn new(row: i32, col: i32) -> Self {
        GridPos { row, col }
    }

    /// Return (x, y) world coordinates of the cell's anchor (bottom-left corner).
    pub fn anchor_coordinates(self) -> Vec2 {
        // IMPORTANT! row & col are swapped when making coordinates.
        // We tend to think of cells via (row, col): vertical then horizontal.
        // But we tend to think of coordinates as (x, y): horizontal then vertical.
        Vec2::new(self.col as f32 * TILE_SIZE as f32, self.row as f32 * TILE_SIZE as f32)
    }
}

/// Kind of Tile in the maze:
/// - Wall is not walkable.
/// - Floor is walkable.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileKind {
    Floor,
    Wall,
}

/// A Tile, with its TileKind and visual information
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub kind: TileKind,
    // TODO: visual information from spritesheet
}

/// Boundaries of a grid, validated at creation. Guarantees all of these:
/// - `bottom <= top`.
/// - `left <= right`.
/// - At least one row and one col.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridBounds {
    bottom: i32,
    left: i32,
    top: i32,
    right: i32,
}

impl GridBounds {
    fn new(bottom: i32, left: i32, top: i32, right: i32) -> Result<Self, GridError> {
        if bottom > top {
            return Err(GridError::InvertedRows{bottom, top});
        }
        if left > right {
            return Err(GridError::InvertedCols{left, right});
        }

        // let row_count = top as i64 - bottom as i64 + 1;
        // if i32::try_from(row_count).is_err() {
        //     return Err(GridError::TooManyRows { row_count});
        // }

        // let col_count = right as i64 - left as i64 + 1;
        // if i32::try_from(col_count).is_err() {
        //     return Err(GridError::TooManyCols { col_count });
        // }

        return Ok(GridBounds { bottom, left, top, right });
    }

    // /// Bounds from the bottom-left cell and the number of rows/cols.
    // fn from_origin(row_offset: i32, col_offset: i32, row_count: i32, col_count: i32) -> Result<Self, GridError> {
    //     if !(row_count >= 1) {
    //         return Err(GridError::RowCountNotPositive{ row_count });
    //     }
    //     if !(col_count >= 1) {
    //         return Err(GridError::ColCountNotPositive{ col_count });
    //     }
    //     let Some(top) = last_index(row_offset, row_count) else {
    //         return Err(GridError::RowsOverflow { row_offset, row_count });
    //     };
    //     let Some(right) = last_index(col_offset, col_count) else {
    //         return Err(GridError::ColsOverflow { col_offset, col_count });
    //     };

    //     return Ok(GridBounds { bottom: row_offset, left: col_offset, top, right });
    // }

    fn rows(&self) -> i32 {
        // TODO: deal with int overflows
        return self.top - self.bottom + 1;
    }

    fn columns(&self) -> i32 {
        // TODO: deal with int overflows
        return self.right - self.left + 1;
    }

    fn bottom(&self) -> i32 {
        return self.bottom;
    }

    fn left(&self) -> i32 {
        return self.left;
    }

    fn top(&self) -> i32 {
        return self.top;
    }

    fn right(&self) -> i32 {
        return self.right;
    }

    fn row_range(&self) -> RangeInclusive<i32> {
        self.bottom..=self.top
    }

    fn col_range(&self) -> RangeInclusive<i32> {
        self.left..=self.right
    }

    fn contains(&self, pos: GridPos) -> bool {
        return self.row_range().contains(&pos.row) && self.col_range().contains(&pos.col);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    /// `bottom` > `top`.
    InvertedRows { bottom: i32, top: i32 },
    /// `left` > `right`.
    InvertedCols { left: i32, right: i32 },
    /// Given `GridPos` is outside the grid.
    OutOfBounds { pos: GridPos, bounds: GridBounds },
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedRows { bottom, top } => write!(f, "bottom({bottom}) > top({top})"),
            Self::InvertedCols { left, right } => write!(f, "left({left}) > right({right})"),
            Self::OutOfBounds { pos, bounds} => write!(f, "({}, {}) is out of bounds (bottom={}, left={}, top={}, right={})", pos.row, pos.col, bounds.bottom(), bounds.left(), bounds.top(), bounds.right()),
        }
    }
}

impl std::error::Error for GridError {}

/// A 2D grid mapping `(row, col)` -> `Tile`.
/// The indexes `(row, col)` can be negative. It represents the cell's position
/// w.r.t the world map's root (at `(0, 0)`).
///
/// IMPORTANT: Bevy uses right-handed coordinates, so we fit the (row, col)
/// to that as well (https://bevy-cheatbook.github.io/fundamentals/coords.html).
#[derive(Resource)]
pub struct TileGrid {
    bounds: GridBounds,
    tiles: Vec<Tile>,
}

impl TileGrid {
    fn new(bounds: GridBounds) -> Self {
        let len = (bounds.rows() as usize) * (bounds.columns() as usize);
        return TileGrid {
            tiles: vec![Tile { kind: TileKind::Floor }; len],
            bounds,
        };
    }

    fn bounds(&self) -> GridBounds {
        return self.bounds;
    }

    /// Index of a `GridPos` in `tiles`, or `None` if not in the grid.
    fn index(&self, pos: GridPos) -> Option<usize> {
        if !self.bounds.contains(pos) {
            return None;
        }
        let row = (pos.row - self.bounds.bottom) as usize;
        let col = (pos.col - self.bounds.left) as usize;
        return Some(row * self.bounds.columns() as usize + col);
    }

    fn set(&mut self, pos: GridPos, tile: Tile) -> Result<(), GridError> {
        let idx = self.index(pos).ok_or(GridError::OutOfBounds { pos, bounds: self.bounds })?;
        self.tiles[idx] = tile;
        return Ok(());
    }

    fn at(&self, pos: GridPos) -> Option<Tile> {
        match self.index(pos) {
            None => return None,
            Some(idx) => return Some(self.tiles[idx]),
        }
    }

    // pub fn is_walkable(&self, pos: GridPos) -> bool {
    //     matches!(
    //         self.get_tile(pos).map(|tile| tile.kind),
    //         Some(TileKind::Floor) | Some(TileKind::Exit)
    //     )
    // }

    // fn grid_to_world(&self, row: i32, col: i32) -> Vec2 {
    //     Vec2::new(col as f32 * TILE_SIZE, row as f32 * TILE_SIZE)
    // }
}

// TODO: tests for GridBounds, TileGrid
