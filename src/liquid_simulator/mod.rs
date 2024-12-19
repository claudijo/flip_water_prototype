mod components;
mod demo_systems;
mod grid;
mod spatial_hash;
mod systems;

use crate::liquid_simulator::demo_systems::{position_liquid_particles, spawn_tank};
use crate::liquid_simulator::systems::simulate_liquid;
use bevy::prelude::*;

pub struct LiquidSimulatorPlugin;

impl Plugin for LiquidSimulatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tank);
        app.add_systems(Update, position_liquid_particles);

        app.add_systems(PreUpdate, simulate_liquid);
    }
}
