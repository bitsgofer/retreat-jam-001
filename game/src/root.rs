use bevy::prelude::{Entity, FromWorld, Name, Resource, Transform, Visibility, World};

use crate::global;

/// Game's root node, mainly for others to inherrit its offset vector:
/// TRANSFORM_GLOBAL_OFFSET. This will center the graphics aagain,
/// negating the effect of Anchor::BOTTOM_LEFT.
#[derive(Resource)]
pub struct GameRoot(pub Entity);

impl FromWorld for GameRoot {
    fn from_world(world: &mut World) -> Self {
        let root = world
            .spawn((
                Name::new("GameRoot"),
                Transform::from_translation(global::TRANSFORM_GLOBAL_OFFSET),
                Visibility::default(),
            ))
            .id();
        return GameRoot(root);
    }
}
