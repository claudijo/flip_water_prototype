use crate::pic_flip::staggered_grid::{CellType, StaggeredGrid};
use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct FluidSimulator(StaggeredGrid);

impl FluidSimulator {
    pub fn new(staggered_grid: StaggeredGrid) -> Self {
        Self(staggered_grid)
    }

    pub fn cols(&self) -> usize {
        self.0.cols
    }

    pub fn rows(&self) -> usize {
        self.0.rows
    }

    pub fn cell_spacing(&self) -> f32 {
        self.0.spacing
    }

    pub fn cell_type(&self, i: i32, j: i32) -> Option<&CellType> {
        self.0.cell_types.get_at(i, j)
    }

    pub fn horizontal_velocity(&self, i: i32, j: i32) -> Option<&f32> {
        self.0.horizontal_velocities.get_at(i, j)
    }

    pub fn vertical_velocity(&self, i: i32, j: i32) -> Option<&f32> {
        self.0.vertical_velocities.get_at(i, j)
    }

    fn reset(&mut self) {
        self.0.pressures.reset();
        self.0.horizontal_velocities.reset();
        self.0.vertical_velocities.reset();
        self.0.sum_horizontal_weights.reset();
        self.0.sum_vertical_weights.reset();

        // Reset fluid cells
        for mut cell_type in self.0.cell_types.iter_mut() {
            if *cell_type == CellType::FLUID {
                *cell_type = CellType::EMPTY;
            }
        }
    }

    pub fn particles_to_grid(&mut self, velocities_points: Vec<(Vec2, Vec2)>) {
        self.reset();

        if self.0.border {
            self.0.set_boundary_cells_to_solid();
        }

        for (velocity, point) in velocities_points {
            let point = point - self.0.offset;
            self.0.set_particle_cell_to_fluid(point);
            self.0.splat_velocities(velocity, point);
        }

        self.0.normalize_velocities();

        self.0.store_normalized_velocities();

        self.0.set_boundary_velocities();
    }

    pub fn project_pressure(&mut self) {
        self.0.project_pressure(100, 1.9);

        /*

        // Cache which neighbors are non-SOLID and which ones are FLUID.
        MakeNeighborMaterialInfo(cell_labels_, &neighbors_);

        // Determine fluid pressures that make fluid velocity as divergence-free as
        // we reasonably can.
        pressure_solver_.ProjectPressure(cell_labels_, neighbors_, u_, v_, w_, &p_);

        // Update grid fluid velocity values based on the fluid pressure gradient.
        SubtractPressureGradientFromVelocity():

         */
    }

    pub fn grid_to_particle(&self, point: Vec2) -> Option<Vec2> {
        let point = point - self.0.offset;
        self.0.interpolate_velocity(point)
    }

    pub fn offset(&self) -> Vec2 {
        self.0.offset
    }
}
