# Foreword

> Here be Dragons!

This is a beginner guide, but **NOT** a "gamedev for dummy" one.

My motivation is to layout a learning path with a lot more breadth and depth.
Especially, if you are a solo developers who want to fully understand the
process of designing, making and releaseing a game (using Bevy), this book is
for you.

There will be more detours and rabbit holes than you actually wanted,
as gamedev covers an incredible breadth and depth. At times you might want to
give up. But persist and the fog will eventually lift, showing you amazing
scenery.

From there, I hope this note will help you choose your own adventure!

# Index

- [Chapter 1: Setting up toolings](#)
- [Chapter 2: A blank window](#)
- [Chapter 3: Global constants and hot-reloaded configs](#)
- [Chapter 4: Draw a level](#)

## Chapter 1: Setitng up toolings


## Chapter 2:

## Chapter 3:

## Chapter 4:

- Add a LevelPlugin that do nothing
- Think about what goes into a level:
  - Favor building from scratch, instead of using crates (e.g. ecs_tilemap).
    It is harder to understand a blackbox => build from 1st principle first.
  - Try to avoid marco where possible, as it also hide things.
  - A tile grid that can be drawn onto the screen.
    - GridPos
    - quirk: coordinates in the game are `(x, y)` vec.
      meanwhile the traditional way to think about 2D grid is `(row, col)`,
      which is `(y, x)` -> need to pay attention of the switch.
      - Q: should we just think in terms of map coordinate? maybe, but not natural for me.
    - Tile:
      - TileKind
        - TileKind might makes it harder to expand the visual part, e.g. walls
          in different environment (rock wall, tree wall, brick wall, etc).
          At that point might need to make it a component/property.
          Even defining Start/Exit seems "wrong"
    - TileGrid, many choices for memory layout:
      - HashMap<GridPos, Tile>
      - Vec<Vec<Tile>>
      - Vec<Tile> with row*col elements
      - Started with a HashMap (not sure me or the AI), probably because it
        handles negative row/col index easily.
      - Generally a flat Vec is best, but need to deal with negative index offset.
        Can use an offset to convert negative row/col -> integer index.
      - Let's leave integer overflow for later @_@
      - Probably need unit test at this point.
    - Add image to Tile
  - New approach: define the hierarchy first: Level -> TileGrid -> Tile(GridPos, Sprite, TileKind)

        ┌─────────────────────────────────┐
        │ Level                           │
        └────────────────┬────────────────┘
                         │ parent of
                         ▼
        ┌─────────────────────────────────┐
        │ TileGrid                        │
        └────────────────┬────────────────┘
                         │ parent of
                         ▼
        ┌─────────────────────────────────┐
        │ Tile(s) { kind: TileKind }      │
        └─────────────────────────────────┘

  - This top-down approach (after doing bottom-up learning about the pieces)
    seems to yield better structure and easier to get started.
    - Quite similar to how we construct a scene in other enigne (e.g. GoDot).
    - Afterwards, can start splitting these into their own modules.
    - Hierarchy of data/struct != hierachy of entities (later can be more compressed).
  - Player start position
  - Maze
  - Player exit position
  - Constraint: there must always be a path from start -> exit
  - Nice to have: maze shape, no secret room (or should there be one?)
  - Queue for later: bigger maze, camera following the player.
  - Queue for later: level is auto-generated (from a seed?)

# Rust things

- Each Rust file is ~=1 modules (or more). Everything inside is private
  unless otherwise stated. Start private, and expose as needed.
- Feeling storngly against `use [crate]::prelude::*`, as it makes it hard
  to read w/o IDE support.
  - Explicit `use` helps understanding what we are getting from Bevy.

# Others

- https://en.wikipedia.org/wiki/Spiral_approach
- https://google.github.io/styleguide/rustguide.html
- https://google.github.io/styleguide/rustguide.html#apply-non_exhaustive-to-publicly-exposed-enums
- https://doc.rust-lang.org/std/primitive.unit.html
- https://doc.rust-lang.org/stable/std/vec
- https://bevy-cheatbook.github.io/fundamentals/coords.html
- https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#creating-idiomatic-use-paths
- https://rust-lang.github.io/rust-clippy/master/index.html#absolute_paths
- https://doc.rust-lang.org/style-guide/items.html#use-items
- https://doc.rust-lang.org/book/ch11-01-writing-tests.html
- https://users.rust-lang.org/t/to-use-prelude-or-to-not-to-use-prelude-that-is-the-question/110855
- https://old.reddit.com/r/rust/comments/fg6vrn/for_your_consideration_an_alternative_to_the/?cache-bust=1793514182495
- https://github.com/bevyengine/bevy/discussions/10083
