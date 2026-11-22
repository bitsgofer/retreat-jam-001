use bevy::prelude::{Vec2, Vec3};

pub const TILE_SIZE_PIXELS: u32 = 16;
pub const LEVEL_SPRITESHEET_PATH: &str = "spritesheets/level.png";
pub const LEVEL_Z: f32 = 0.0;
pub const PLAYER_SPRITESHEET_PATH: &str = "spritesheets/player.png";
pub const PLAYER_Z: f32 = 1.0;
pub const PLAYER_ANIMATION_FRAME_DURATION_SECONDS: f32 = 0.02;
pub const PLAYER_ANIMATION_FRAMES: usize = 16;
pub const SIMULATION_INTERVAL_SECONDS: f64 = 0.2;
pub const TRANSFORM_GLOBAL_OFFSET: Vec3 = Vec2::new(
    TILE_SIZE_PIXELS as f32 / 2.0 * -1.0,
    TILE_SIZE_PIXELS as f32 / 2.0 * -1.0,
)
.extend(0.0);
