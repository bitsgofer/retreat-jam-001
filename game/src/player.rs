use bevy::prelude::{App, Commands, Component, Name, Plugin, Res, Startup, Transform};
use bevy::sprite::Anchor;

pub struct PlayerPlugin;

use crate::global;
use crate::level::GridPos;
use crate::spritesheet::SpriteSheet;
use crate::spritesheet::SpriteSheets;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
    }
}

fn spawn_player(mut commands: Commands, sheets: Res<SpriteSheets>) {
    let sheet = sheets.get(SpriteSheet::Player);
    let pos = GridPos::new(0, 0);
    commands.spawn((
        pos,
        sheet.sprite(0 as usize),
        Transform::from_translation(pos.to_xy().extend(global::PLAYER_Z)),
        Anchor::BOTTOM_LEFT,
        Player,
        Name::new(format!("Player({}, {})", pos.row, pos.col)),
    ));
}

#[derive(Component)]
struct Player;
