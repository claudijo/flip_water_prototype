use crate::pic_flip::resources::Gravity;
use crate::pic_flip::systems::{
    debug_simulation, integrate_particles, simulate_fluid_mechanics, spawn_fluid_container,
};
use bevy::prelude::*;

mod components;
mod grid;
mod resources;
mod spatial_hash;
mod staggered_grid;
mod systems;

pub struct PicFlipPlugin;

impl Plugin for PicFlipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -500.)));
        app.add_systems(Startup, spawn_fluid_container);
        app.add_systems(
            Update,
            (
                integrate_particles,
                simulate_fluid_mechanics,
                // debug_simulation,
            )
                .chain(),
        );
    }
}
