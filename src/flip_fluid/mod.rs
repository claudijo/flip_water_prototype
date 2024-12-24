mod components;
mod systems;

use crate::flip_fluid::systems::{move_particles, simulate_liquid, spawn_tank};
use bevy::prelude::*;

pub struct FlipFluidPlugin;

impl Plugin for FlipFluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tank);
        app.add_systems(Update, move_particles);

        app.add_systems(PreUpdate, simulate_liquid);
    }
}
