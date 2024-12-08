mod components;
mod systems;
mod resources;

use crate::flop::systems::{debug_cells, integrate_particles, spawn_liquid_container};
use bevy::prelude::*;
use crate::flop::resources::Gravity;

pub struct FlopPlugin;

impl Plugin for FlopPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
        app.add_systems(Startup, spawn_liquid_container);
        app.add_systems(Update, integrate_particles);
        app.add_systems(Update, debug_cells);
    }
}
