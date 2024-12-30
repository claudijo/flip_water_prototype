mod components;
mod systems;

use crate::flip_fluid::systems::{color_particles, integrate_position, integrate_rotation, move_particles, simulate_liquid, spawn_tank, update_angular_velocity, update_linear_velocity};
use bevy::prelude::*;

pub struct FlipFluidPlugin;

impl Plugin for FlipFluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tank);
        app.add_systems(Update, (move_particles, color_particles));
        app.add_systems(
            PreUpdate,
            (
                update_linear_velocity,
                update_angular_velocity,
                integrate_position,
                integrate_rotation,
                simulate_liquid,
            )
                .chain(),
        );
    }
}
