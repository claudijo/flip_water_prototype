use crate::liquid_simulator::components::LiquidSimulator;
use bevy::prelude::*;

pub const GRAVITY: Vec2 = Vec2::new(0., -10.);
pub const RESOLVE_OVERLAP_ITERATIONS: usize = 4;

pub fn simulate_liquid(mut simulator_query: Query<&mut LiquidSimulator>, time: Res<Time>) {
    for mut simulator in &mut simulator_query {
        simulator.integrate_particles(time.delta_secs(), GRAVITY);
        simulator.push_particles_apart(RESOLVE_OVERLAP_ITERATIONS);
    }
}
