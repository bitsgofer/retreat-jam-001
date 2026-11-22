use std::fmt;

use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::Xoshiro128PlusPlus;

use bevy::log::info;
use bevy::prelude::{
    App, ButtonInput, ChildOf, Commands, Component, Fixed, FixedUpdate, IntoScheduleConfigs,
    KeyCode, Name, NextState, Plugin, Query, Res, ResMut, Resource, RunFixedMainLoop,
    RunFixedMainLoopSystems, Sprite, Startup, State, Time, Timer, TimerMode, Transform, Update,
    With,
};

use bevy::sprite::Anchor;

use crate::global;
use crate::level::{GridDelta, GridPos, Level, Levels, Tile, TileGrid};
use crate::root::GameRoot;
use crate::spritesheet::{SpriteSheet, SpriteSheets};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .insert_resource(SimStepNo::new(1))
            .insert_resource(Time::<Fixed>::from_seconds(
                global::SIMULATION_INTERVAL_SECONDS,
            ))
            .add_systems(
                RunFixedMainLoop,
                collect_input.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            ) // per render frame, but before FixedUpdate (sim) // !IMPORTANT
            .add_systems(
                FixedUpdate,
                (compute_sim_changes, simulate, check_win_condition).chain(),
            ) // per sim step
            .add_systems(Update, animate_player); // per render frame
        return;
    }
}

fn spawn_player(
    mut commands: Commands,
    root: Res<GameRoot>,
    sheets: Res<SpriteSheets>,
    level: Res<State<Level>>,
    levels: Res<Levels>,
) {
    let sheet = sheets.get(SpriteSheet::Player);
    let grid = levels.get(*level.get());
    commands.spawn((
        Player,
        ChildOf(root.0),
        sheet.sprite(0 as usize),
        Transform::default(),
        Anchor::BOTTOM_LEFT,
        Name::new(format!("Player")),
        InputBuffer::default(),
        InputCommand::default(),
        SimulationState {
            pos: grid.start(),
            prev_pos: grid.start(),
            ..Default::default()
        },
        AnimationState::default_with_anim_duration(global::PLAYER_ANIMATION_FRAME_DURATION_SECONDS),
    ));
}

/// Every render frame, look at what key(s) player pressed and when it happened
/// (by render frame sequence number) => store into InputBuffer.
///
/// If the same key is already pressed, then last write wins.
/// This happens when press + release + press very quickly.
///
/// The buffer allows the sim to see all keys pressed since the last sim step.
/// => handle player's intent/input more accurately.
///
/// !IMPORTANT: The sim will also clear the buffer after reading.
fn collect_input(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut InputBuffer, With<Player>>,
) {
    let Ok(mut buffer) = query.single_mut() else {
        return;
    };

    for key in Facing::ALL.map(Facing::key_codes).concat() {
        if input.just_pressed(key) {
            buffer.record(key);
        }
    }
}

/// Calculate the player's input towards the simulation.
///
/// In a client-server setup, this will be in the client code.
fn compute_sim_changes(
    sim_step: Res<SimStepNo>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut InputBuffer, &mut InputCommand), With<Player>>,
) {
    let Ok((mut input_buffer, mut cmd)) = query.single_mut() else {
        return;
    };

    let (delta, facing) = player_intent(&input_buffer, input);
    // info!("compute_sim_changes: delta={}, facing={:?}", delta, facing);

    input_buffer.clear();

    let command = InputCommand {
        sim_step: *sim_step,
        delta,
        facing,
    };
    *cmd = command;
    // info!("compute_sim_changes: send cmd={}", *cmd);
}

/// Called every RenderFrame that runs together (and before) a SimStep,
/// this fn computes the next delta(row, col) && Facing direction,
/// which we will need for the next simulation step.
///
/// It use the buffered input (since the last SimStep) + the current input
/// seen this RenderFrame for this calculation.
fn player_intent(
    buffer: &InputBuffer,
    input: Res<ButtonInput<KeyCode>>,
) -> (GridDelta, Option<Facing>) {
    // keys pressed in this RenderFrame
    let pressed_down = if input.pressed(KeyCode::ArrowDown)
        || input.pressed(KeyCode::KeyS)
        || input.pressed(KeyCode::KeyJ)
    {
        true
    } else {
        false
    };
    let pressed_left = if input.pressed(KeyCode::ArrowLeft)
        || input.pressed(KeyCode::KeyA)
        || input.pressed(KeyCode::KeyH)
    {
        true
    } else {
        false
    };
    let pressed_up = if input.pressed(KeyCode::ArrowUp)
        || input.pressed(KeyCode::KeyW)
        || input.pressed(KeyCode::KeyK)
    {
        true
    } else {
        false
    };
    let pressed_right = if input.pressed(KeyCode::ArrowRight)
        || input.pressed(KeyCode::KeyD)
        || input.pressed(KeyCode::KeyL)
    {
        true
    } else {
        false
    };

    // combine pressed_DIRECTION (in this RenderFrame) with the input
    // bufered since the last SimStep (it was not yet handled)
    // => direction that the player intended to move
    let down = pressed_down || buffer.pressed_down;
    let left = pressed_left || buffer.pressed_left;
    let up = pressed_up || buffer.pressed_up;
    let right = pressed_right || buffer.pressed_right;

    // translate keys pressed -> delta in row && col, i.e (-1, 0, +1)
    //
    // we also handle SOCD (Simultaneous Opposing Cardinal Directions),
    // picking only one direction between lef/right or up/down (the latest)
    let drow: Option<(GridDelta, Facing, InputSeqNo)> = if down && up {
        if buffer.pressed_down_at.0 > buffer.pressed_up_at.0 {
            Some((GridDelta::DOWN, Facing::Down, buffer.pressed_down_at))
        } else {
            Some((GridDelta::UP, Facing::Up, buffer.pressed_up_at))
        }
    } else if down {
        Some((GridDelta::DOWN, Facing::Down, buffer.pressed_down_at))
    } else if up {
        Some((GridDelta::UP, Facing::Up, buffer.pressed_up_at))
    } else {
        None
    };
    let dcol: Option<(GridDelta, Facing, InputSeqNo)> = if left && right {
        if buffer.pressed_left_at.0 > buffer.pressed_right_at.0 {
            Some((GridDelta::LEFT, Facing::Left, buffer.pressed_left_at))
        } else {
            Some((GridDelta::RIGHT, Facing::Right, buffer.pressed_right_at))
        }
    } else if left {
        Some((GridDelta::LEFT, Facing::Left, buffer.pressed_left_at))
    } else if right {
        Some((GridDelta::RIGHT, Facing::Right, buffer.pressed_right_at))
    } else {
        None
    };

    // aggregate into delta(row, col)
    // and translate the latest direction key pressed -> Facing direction
    match (drow, dcol) {
        (Some((dr, r_facing, r_pressed_at)), Some((dc, c_facing, c_pressed_at))) => {
            let facing = if r_pressed_at.0 > c_pressed_at.0 {
                r_facing
            } else {
                c_facing
            };

            (dr + dc, Some(facing))
        }
        (Some((dr, r_facing, _)), None) => (dr, Some(r_facing)),
        (None, Some((dc, c_facing, _))) => (dc, Some(c_facing)),
        (None, None) => (GridDelta::ZERO, None),
    }
}

/// Run simulation.
///
/// In a client-server setup, this will be the server code.
fn simulate(
    mut sim_step: ResMut<SimStepNo>,
    level: Res<State<Level>>,
    levels: Res<Levels>,
    mut query: Query<(&InputCommand, &mut SimulationState), With<Player>>,
) {
    let Ok((cmd, mut sim)) = query.single_mut() else {
        return;
    };
    // info!("sim: received cmd={}; state: pos={}, facing={}", cmd, sim.pos, sim.facing);

    // advance sim state (current -> prev)
    sim.prev_pos = sim.pos;
    sim.prev_facing = sim.facing;
    // advance sim step
    sim_step.0 = sim_step.0 + 1;

    // run the actual sim
    let grid = levels.get(*level.get());
    simulate_player(&mut sim, cmd, grid);

    return;
}

fn check_win_condition(
    mut rng: ResMut<GameRng>,
    level: Res<State<Level>>,
    levels: Res<Levels>,
    mut next_level: ResMut<NextState<Level>>,
    mut query: Query<&mut SimulationState, With<Player>>,
) {
    let Ok(mut sim) = query.single_mut() else {
        return;
    };

    let lvl = *level.get();
    if sim.pos != levels.get(lvl).end() {
        return;
    }

    let mut next: Level = lvl;
    while next == lvl {
        next = rng
            .choose(&Level::PLAYABLE)
            .expect("Level::PLAYABLE is never empty");
    }
    info!(
        "maze end reached: change level from={:?} -> to={:?}",
        lvl, next
    );

    next_level.set(next);

    // Reset player's position.
    // NOTE: Need to set both pos and prev_pos, so animate_player() doesn't
    // animate the player fro last Tile::MazeEnd to the new Tile::MazeStart.
    let start = levels.get(next).start();
    sim.pos = start;
    sim.prev_pos = start;
    return;
}

fn simulate_player(sim: &mut SimulationState, cmd: &InputCommand, grid: &TileGrid) {
    let changes = player_changes(sim.pos, cmd, grid);
    match changes {
        Some((pos, facing)) => {
            sim.pos = pos;
            sim.facing = facing;
        }
        None => {
            if let Some(facing) = cmd.facing {
                sim.facing = facing;
            };
        }
    }
}

/// Determine the player's new position & facing,
/// or None if there's no delta or the move is blocked.
///
/// From pos, validate if we can use InputCommand.delta to move to next_pos,
/// which is either:
/// - [Vert]: moved Down/Up
/// - [Horz]: moved Left/Right
/// - [Diag]: moved Down + Left/Down + Right/Up + Left/Up + Right
///
/// ```text
/// ┌──────┬──────┐
/// │      │      │
/// │ Vert │ Diag │
/// │      │      │
/// ├──────┼──────┤
/// │      │      │
/// │ pos  │ Horz │
/// │      │      │
/// └──────┴──────┘
/// ```
///
/// For next_pos to be a valid move:
/// - next_pos must be within TileGrid's bounds.
/// - next_pos must NOT be a wall.
/// - Reaching next_pos takes one or two 4-directional moves (Up/Down/Left/Right):
///   - If the next_pos is Vert/Horz, it costs 1 move.
///   - If next_pos is a Diag, can take either a Vert/Horz move first
///     before making another move to Diag, for the total of 2 moves.
///   => This allows for smoother movements in large empty space
///   => It also allows player to effectively go around valid corners.
///   => However, they cannot squeeze between diagonal gaps
///      or run diagonally into a wall, which is logically sound.
///   => In general, this seems to feel more natural for movement.
///      We are trying to replicate the same moves found in games like
///      Stardew Valley, Sea of Stars, etc.
fn player_changes(pos: GridPos, cmd: &InputCommand, grid: &TileGrid) -> Option<(GridPos, Facing)> {
    let delta = cmd.delta;
    let next_pos = pos + delta;

    if delta == GridDelta::ZERO {
        // No movement -> no change
        return None;
    }

    let is_valid_dest = |p: GridPos, d: GridDelta| -> bool {
        let next_p = p + d;
        grid.contains(next_p) && grid[next_p] != Tile::Wall
    };
    let is_within_two_grid_moves = |p: GridPos, d: GridDelta| -> bool {
        match d {
            GridDelta::DOWN | GridDelta::LEFT | GridDelta::UP | GridDelta::RIGHT => true,
            GridDelta::DOWN_LEFT => {
                is_valid_dest(p, GridDelta::DOWN) || is_valid_dest(p, GridDelta::LEFT)
            }
            GridDelta::UP_LEFT => {
                is_valid_dest(p, GridDelta::UP) || is_valid_dest(p, GridDelta::LEFT)
            }
            GridDelta::UP_RIGHT => {
                is_valid_dest(p, GridDelta::UP) || is_valid_dest(p, GridDelta::RIGHT)
            }
            GridDelta::DOWN_RIGHT => {
                is_valid_dest(p, GridDelta::DOWN) || is_valid_dest(p, GridDelta::RIGHT)
            }
            _ => {
                panic!(
                    "player delta is not in GridDelta::VALID_MOVES({:?}); delta={}",
                    GridDelta::VALID_MOVES,
                    d
                );
            }
        }
    };
    // info!("player_changes: next_pos={}, is_valid_dest={}, is_within_two_grid_moves={}", next_pos, is_valid_dest(pos, delta), is_within_two_grid_moves(pos, delta));

    if is_valid_dest(pos, delta) && is_within_two_grid_moves(pos, delta) {
        // info!("player_changes: allow move from pos={} -> next_pos={}", pos, next_pos);
        Some((next_pos, cmd.facing.unwrap_or(Facing::Down)))
    } else {
        // info!("player_changes: disallow move from pos={} -> next_pos={}", pos, next_pos);
        None
    }
}

fn animate_player(
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &mut Transform,
            &mut AnimationState,
            &mut Sprite,
            &SimulationState,
        ),
        With<Player>,
    >,
) {
    let Ok((mut transform, mut anim, mut sprite, sim)) = query.single_mut() else {
        return;
    };

    interpolate_xy(fixed_time, &mut transform, sim); // TODO: What happens if we use time instead of fixed_time?
    update_sprite(time, sim, &mut anim, &mut sprite);

    return;
}

fn interpolate_xy(fixed_time: Res<Time<Fixed>>, transform: &mut Transform, sim: &SimulationState) {
    let frac = fixed_time.overstep_fraction();
    let from = sim.prev_pos.to_xy();
    let to = sim.pos.to_xy();

    transform.translation = from.lerp(to, frac).round().extend(global::PLAYER_Z);
    // info!("animate: transform.translation(x={}, y={})", transform.translation.x, transform.translation.y);
}

fn update_sprite(
    time: Res<Time>,
    sim: &SimulationState,
    anim: &mut AnimationState,
    sprite: &mut Sprite,
) {
    // read sim states
    let next_facing: Facing = sim.facing;
    let next_moving: bool = sim.pos != sim.prev_pos;

    let sprite_col = anim.sprite_index % global::PLAYER_ANIMATION_FRAMES;

    // compare: sim state vs animation state => animation actions
    let moving_will_change = next_moving != anim.moving; // TODO: investigate Changed<>
    let facing_will_change = next_facing != anim.facing;
    let will_continue_moving = !moving_will_change && !facing_will_change && next_moving == true;
    if moving_will_change || facing_will_change {
        anim.moving = next_moving;
        anim.facing = next_facing;
        anim.sprite_index =
            anim.facing.row_index_in_player_spritesheet() * global::PLAYER_ANIMATION_FRAMES;
        anim.timer.reset();
        // info!("animate: switch to {}", anim);
    } else if will_continue_moving {
        anim.timer.tick(time.delta());
        anim.sprite_index = anim.facing.row_index_in_player_spritesheet()
            * global::PLAYER_ANIMATION_FRAMES
            + (sprite_col + anim.timer.times_finished_this_tick() as usize)
                % global::PLAYER_ANIMATION_FRAMES;
        // info!("animate: continue moving {}", anim);
    } else {
        // info!("animate: standing still {}", anim);
    };

    // update sprite to new index
    let Some(atlas) = sprite.texture_atlas.as_mut() else {
        return;
    };
    atlas.index = anim.sprite_index;
    // info!("animate: atlas.index={}", atlas.index);
}

#[derive(Component)]
struct Player;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    #[default]
    Down,
    Left,
    Up,
    Right,
}

impl Facing {
    const ALL: [Facing; 4] = [Facing::Down, Facing::Left, Facing::Up, Facing::Right];

    fn row_index_in_player_spritesheet(&self) -> usize {
        match self {
            Facing::Down => 0,
            Facing::Left => 1,
            Facing::Up => 2,
            Facing::Right => 3,
        }
    }

    fn key_codes(self) -> &'static [KeyCode] {
        // support both Arrow(s), WASD and HJKL (vim)
        match self {
            Facing::Down => &[KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::KeyJ],
            Facing::Left => &[KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::KeyH],
            Facing::Up => &[KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::KeyK],
            Facing::Right => &[KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::KeyL],
        }
    }
}

impl fmt::Display for Facing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Facing::Down => write!(f, "Down"),
            Facing::Left => write!(f, "Left"),
            Facing::Up => write!(f, "Up"),
            Facing::Right => write!(f, "Right"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct InputSeqNo(u32);

impl InputSeqNo {
    fn new(v: u32) -> Self {
        return InputSeqNo(v);
    }

    fn inc(&mut self) {
        self.0 += 1;
    }
}

impl fmt::Display for InputSeqNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug)]
struct InputBuffer {
    seq_no: InputSeqNo,
    pressed_down: bool,
    pressed_down_at: InputSeqNo,
    pressed_left: bool,
    pressed_left_at: InputSeqNo,
    pressed_up: bool,
    pressed_up_at: InputSeqNo,
    pressed_right: bool,
    pressed_right_at: InputSeqNo,
}

impl InputBuffer {
    fn default() -> Self {
        InputBuffer {
            seq_no: InputSeqNo::new(1),
            pressed_down: false,
            pressed_down_at: InputSeqNo::new(0),
            pressed_left: false,
            pressed_left_at: InputSeqNo::new(0),
            pressed_up: false,
            pressed_up_at: InputSeqNo::new(0),
            pressed_right: false,
            pressed_right_at: InputSeqNo::new(0),
        }
    }
}

impl fmt::Display for InputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InputBuffer{{seq_no:{}, down:({}, {}), left:({}, {}), up:({}, {}), right:({}, {}),}}",
            self.seq_no,
            self.pressed_down,
            self.pressed_down_at,
            self.pressed_left,
            self.pressed_left_at,
            self.pressed_up,
            self.pressed_up_at,
            self.pressed_right,
            self.pressed_right_at
        )
    }
}

impl InputBuffer {
    fn record(&mut self, key: KeyCode) {
        self.seq_no.0 += 1;
        match key {
            KeyCode::ArrowDown | KeyCode::KeyS | KeyCode::KeyJ => {
                self.pressed_down = true;
                self.pressed_down_at = InputSeqNo(self.seq_no.0);
            }
            KeyCode::ArrowLeft | KeyCode::KeyA | KeyCode::KeyH => {
                self.pressed_left = true;
                self.pressed_left_at = InputSeqNo(self.seq_no.0);
            }
            KeyCode::ArrowUp | KeyCode::KeyW | KeyCode::KeyK => {
                self.pressed_up = true;
                self.pressed_up_at = InputSeqNo(self.seq_no.0);
            }
            KeyCode::ArrowRight | KeyCode::KeyD | KeyCode::KeyL => {
                self.pressed_right = true;
                self.pressed_right_at = InputSeqNo(self.seq_no.0);
            }
            _ => {}
        }
        // info!("record: {}", self);
    }

    fn clear(&mut self) {
        self.pressed_down = false;
        self.pressed_down_at = InputSeqNo::default();
        self.pressed_left = false;
        self.pressed_left_at = InputSeqNo::default();
        self.pressed_up = false;
        self.pressed_up_at = InputSeqNo::default();
        self.pressed_right = false;
        self.pressed_right_at = InputSeqNo::default();
    }
}

#[derive(Resource, Debug, Default, Copy, Clone)]
struct SimStepNo(u32);

impl SimStepNo {
    fn new(v: u32) -> Self {
        return SimStepNo(v);
    }

    fn inc(&mut self) {
        self.0 += 1;
    }
}

impl fmt::Display for SimStepNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug, Default)]
struct InputCommand {
    sim_step: SimStepNo,
    delta: GridDelta,
    facing: Option<Facing>,
}

impl fmt::Display for InputCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{sim_step:{}, delta={}, facing={:?} }}",
            self.sim_step, self.delta, self.facing
        )
    }
}

#[derive(Component, Debug)]
struct AnimationState {
    moving: bool,
    timer: Timer,
    facing: Facing,
    sprite_index: usize,
}

impl AnimationState {
    fn default_with_anim_duration(seconds: f32) -> Self {
        return AnimationState {
            moving: false,
            facing: Facing::Down,
            sprite_index: 0 as usize,
            timer: Timer::from_seconds(seconds, TimerMode::Repeating),
        };
    }
}

impl fmt::Display for AnimationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AnimationState{{moving:{}, facing:{}, sprite_index:{}}}",
            self.moving, self.facing, self.sprite_index
        )
    }
}

#[derive(Component, Debug, Default)]
struct SimulationState {
    pos: GridPos,
    prev_pos: GridPos,
    facing: Facing,
    prev_facing: Facing,
}

#[derive(Resource)]
pub struct GameRng(Xoshiro128PlusPlus);

impl GameRng {
    pub fn new(seed: u64) -> Self {
        return GameRng(Xoshiro128PlusPlus::seed_from_u64(seed));
    }

    pub fn choose<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        return items.choose(&mut self.0).copied();
    }
}
