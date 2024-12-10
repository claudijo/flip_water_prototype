use crate::pic_flip::resources::Gravity;
use bevy::prelude::*;
use crate::pic_flip::systems::spawn_fluid_container;

mod components;
mod grid;
mod resources;
mod staggered_grid;
mod systems;

pub struct PicFlipPlugin;

impl Plugin for PicFlipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
        app.add_systems(Startup, spawn_fluid_container);
    }
}
