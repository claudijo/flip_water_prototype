mod components;
mod resources;
mod systems;
mod utils;

use crate::flip::resources::Gravity;
use crate::flip::systems::{integrate_particles, simulate_fluid, spawn_liquid_container};
use bevy::prelude::*;

pub struct FlipPlugin;

impl Plugin for FlipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
        app.add_systems(Startup, spawn_liquid_container);
        app.add_systems(Update, (integrate_particles, simulate_fluid).chain());
    }
}

// PIC method (particle in cell)
// PIC method introduces unwanted numerical viscosity
// 1. simulate_particles
// 2. transfer_particle_velocity_to_grid
// 3. make_grid_velocities_incompressible
// 4. transfer_grid_velocity_to_particles
// Since particles carry velocity we can skip the grid advection step

// FLIP method (fluid implicit particle)
// FLIP method reduces problem with lost velocity information, ie reduces smoothing of particle velocities
// FLIP introduces noise
// 1. simulate_particles
// 2a. transfer_particle_velocity_to_grid
// 2b. make_grid_copy
// 3. make_grid_velocities_incompressible
// 4. add_velocity_changes_to_particles

// Best result mix PIC and FLIP
// 0.1 * PIC + 0.9 * FLIP
