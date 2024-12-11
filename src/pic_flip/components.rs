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
        self.0.cell_spacing
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

    pub fn reset(&mut self) {
        self.0.pressures.reset();
        self.0.horizontal_velocities.reset();
        self.0.vertical_velocities.reset();
        self.0.sum_horizontal_weights.reset();
        self.0.sum_vertical_weights.reset();
    }

    pub fn splat_velocity(&mut self, velocity: Vec2, point: Vec2) {
        self.0.splat_velocity(velocity, point);
    }

    pub fn normalize_velocities(&mut self) {
        self.0.normalize_velocities();
    }
}
