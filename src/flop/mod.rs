mod components;
mod resources;
mod systems;

use crate::flop::resources::Gravity;
use crate::flop::systems::{
    debug_cells, integrate_particles, simulate_fluid, spawn_liquid_container,
};
use bevy::prelude::*;

pub struct FlopPlugin;

impl Plugin for FlopPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
        app.add_systems(Startup, spawn_liquid_container);
        app.add_systems(
            Update,
            (integrate_particles, simulate_fluid, debug_cells).chain(),
        );
    }
}
