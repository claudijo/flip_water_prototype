use crate::liquid_simulator::components::{CellType, LiquidSimulator};
use bevy::color::palettes::basic::{AQUA, GREEN, MAROON, RED};
use bevy::prelude::*;

pub const GRAVITY: Vec2 = Vec2::new(0., -10.);
pub const RESOLVE_OVERLAP_ITERATIONS: usize = 2;

pub const PRESSURE_ITERATIONS: usize = 100;

pub const OVER_RELAXATION: f32 = 1.9;

pub const WATER_CELL_AVERAGE_DENSITY: f32 = 3.;

pub const STIFFNESS_COEFFICIENT: f32 = 1.;

pub fn simulate_liquid(mut simulator_query: Query<&mut LiquidSimulator>, time: Res<Time>) {
    for mut simulator in &mut simulator_query {
        simulator.integrate_particles(time.delta_secs(), GRAVITY);
        simulator.push_particles_apart(RESOLVE_OVERLAP_ITERATIONS);
        simulator.handle_particle_collisions();
        simulator.transfer_velocities(None);
        simulator.update_particle_density();
        simulator.solve_incompressibility(
            PRESSURE_ITERATIONS,
            OVER_RELAXATION,
            STIFFNESS_COEFFICIENT,
            WATER_CELL_AVERAGE_DENSITY,
        );
        simulator.transfer_velocities(Some(0.9));
    }
}

pub fn debug(simulator_query: Query<(&LiquidSimulator, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (simulator, global_transform) in &simulator_query {
        let cell_spacing = simulator.spacing;
        let half_cell_spacing = cell_spacing / 2.;
        let offset = global_transform.translation().xy() + simulator.offset;

        for i in 0..=simulator.cols as i32 {
            for j in 0..=simulator.rows as i32 {
                // Cell type
                if let Some(cell_type) = simulator.cell_types.get(i, j) {
                    let (color, z) = match cell_type {
                        CellType::Empty => (Srgba::rgb(0.6, 0.6, 0.6), 4.),
                        CellType::Fluid => (AQUA, 6.),
                        CellType::Solid => (MAROON, 5.),
                        CellType::OutOfBounds => (RED, 5.),
                    };

                    gizmos.rect(
                        Isometry3d::from_xyz(
                            i as f32 * cell_spacing + offset.x + half_cell_spacing,
                            j as f32 * cell_spacing + offset.y + half_cell_spacing,
                            z,
                        ),
                        Vec2::splat(cell_spacing),
                        color,
                    );
                }

                // Horizontal flow velocity
                if let Some(horizontal_velocity) = simulator.horizontal_velocities.get(i, j) {
                    let x = i as f32 * cell_spacing + offset.x;
                    let y = j as f32 * cell_spacing + offset.y + cell_spacing / 2.;
                    let start = Vec2::new(x, y);
                    gizmos.arrow_2d(start, start + Vec2::X * horizontal_velocity, GREEN);
                }

                // Vertical flow velocity
                if let Some(vertical_velocity) = simulator.vertical_velocities.get(i, j) {
                    let x = i as f32 * cell_spacing + offset.x + cell_spacing / 2.;
                    let y = j as f32 * cell_spacing + offset.y;
                    let start = Vec2::new(x, y);
                    gizmos.arrow_2d(start, start + Vec2::Y * vertical_velocity, GREEN);
                }
            }
        }
    }
}
