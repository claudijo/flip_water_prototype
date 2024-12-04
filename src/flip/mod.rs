mod components;
mod resources;
mod systems;

use crate::flip::resources::Gravity;
use crate::flip::systems::spawn_liquid_container;
use bevy::prelude::*;

pub struct FlipPlugin;

impl Plugin for FlipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
        app.add_systems(Startup, spawn_liquid_container);
    }
}
