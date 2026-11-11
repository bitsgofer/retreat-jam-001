use bevy::prelude::{
    App, Commands, Component, Name, Plugin, Query, Res, Sprite, Startup, Time, Timer, TimerMode,
    Transform, Update, With,
};
use bevy::sprite::Anchor;
use core::time::Duration;

pub struct PlayerPlugin;

use crate::global;
use crate::level::GridPos;
use crate::spritesheet::SpriteSheet;
use crate::spritesheet::SpriteSheets;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, animate_player);
    }
}

fn spawn_player(mut commands: Commands, sheets: Res<SpriteSheets>) {
    let sheet = sheets.get(SpriteSheet::Player);
    let pos = GridPos::new(0, 0);
    let animation_index: usize = 0;
    commands.spawn((
        pos,
        sheet.sprite(animation_index),
        Transform::from_translation(pos.to_xy().extend(global::PLAYER_Z)),
        Anchor::BOTTOM_LEFT,
        Player,
        AnimationState {
            facing: Facing::Down,
            moving: true,
            index: animation_index,
            timer: Timer::from_seconds(
                global::PLAYER_ANIMATION_FRAME_DURATION_SECONDS,
                TimerMode::Repeating,
            ),
        },
        Name::new(format!("Player({}, {})", pos.row, pos.col)),
    ));
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut Sprite), With<Player>>,
) {
    let Ok((mut animation, mut sprite)) = query.single_mut() else {
        return;
    };

    let atlas = match sprite.texture_atlas.as_mut() {
        Some(a) => a,
        None => return,
    };

    animation.advance(time.delta());
    atlas.index = animation.index;
}

#[derive(Component)]
struct Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    Down,
    Left,
    Up,
    Right,
}

impl Facing {
    fn row_index_in_player_spritesheet(&self) -> usize {
        match self {
            Facing::Down => 0,
            Facing::Left => 1,
            Facing::Up => 2,
            Facing::Right => 3,
        }
    }
}

#[derive(Component)]
struct AnimationState {
    facing: Facing,
    moving: bool,
    index: usize,
    timer: Timer,
}

impl AnimationState {
    fn advance(&mut self, delta: Duration) {
        let current_row = self.index / global::MAX_FRAMES_PER_ANIMATION;
        let current_col = self.index % global::MAX_FRAMES_PER_ANIMATION;
        let target_row = self.facing.row_index_in_player_spritesheet();

        if current_row != target_row {
            self.index = target_row * global::MAX_FRAMES_PER_ANIMATION;
            self.timer.reset();
        } else {
            if self.moving {
                self.timer.tick(delta);
                if self.timer.just_finished() {
                    let target_col = (current_col + 1) % global::MAX_FRAMES_PER_ANIMATION;
                    self.index = target_row + target_col;
                } else {
                    // no-op
                }
            } else {
            }
        }
    }
}
