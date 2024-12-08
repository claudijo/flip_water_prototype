use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Velocity(pub Vec2);

#[derive(Copy, Clone)]
pub struct Cell {
    pub vertical_velocity: Option<f32>,
    pub horizontal_velocity: Option<f32>,
    pub sum_of_weights: Vec2,
    pub divergence_scale: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            vertical_velocity: None,
            horizontal_velocity: None,
            sum_of_weights: Vec2::ZERO,
            divergence_scale: 1.0,
        }
    }
}

#[derive(Component)]
pub struct StaggeredGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f32,
    pub cells: Vec<Cell>,
}

impl StaggeredGrid {
    pub fn new(cols: usize, rows: usize, cell_size: f32) -> Self {
        Self {
            cols,
            rows,
            cell_size,
            cells: vec![Cell::default(); cols * rows],
        }
    }

    pub fn with_border_cells(mut self) -> Self {
        for row in 0..self.rows {
            for col in 0..self.cols {
                if row == 0 || row == self.rows - 1 || col == 0 || col == self.cols - 1 {
                    self.cell_at_mut(col, row).divergence_scale = 0.;
                }
            }
        }

        self
    }

    pub fn cell_at(&self, col: usize, row: usize) -> &Cell {
        &self.cells[row * self.cols + col]
    }

    pub fn cell_at_mut(&mut self, col: usize, row: usize) -> &mut Cell {
        &mut self.cells[row * self.cols + col]
    }
}
