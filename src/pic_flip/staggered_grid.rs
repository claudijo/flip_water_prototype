use bevy::prelude::*;

use crate::pic_flip::grid::Grid;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum CellType {
    #[default]
    EMPTY,
    FLUID,
    SOLID,
}

pub struct StaggeredGrid {
    pub cols: usize,
    pub rows: usize,
    pub offset: Vec2,
    pub pressures: Grid<f32>,
    pub horizontal_velocities: Grid<f32>,
    pub vertical_velocities: Grid<f32>,
    pub sum_vertical_weights: Grid<f32>,
    pub sum_horizontal_weights: Grid<f32>,
    pub cell_types: Grid<CellType>,
    pub cell_spacing: f32,
    pub with_boundary_cells: bool,
}

impl StaggeredGrid {
    pub fn new(cols: usize, rows: usize, spacing: f32, offset: Vec2) -> Self {
        Self {
            cols,
            rows,
            cell_types: Grid::new(cols, rows),
            pressures: Grid::new(cols, rows),
            horizontal_velocities: Grid::new(cols + 1, rows),
            sum_horizontal_weights: Grid::new(cols + 1, rows),
            vertical_velocities: Grid::new(cols, rows + 1),
            sum_vertical_weights: Grid::new(cols, rows + 1),
            cell_spacing: spacing,
            offset,
            with_boundary_cells: false,
        }
    }

    pub fn with_boundary_cells(mut self) -> Self {
        self.with_boundary_cells = true;

        self
    }

    pub fn set_boundary_cells_to_solid(&mut self) {
        for i in 0..self.cols {
            for j in 0..self.rows {
                if i == 0 || i == self.cols - 1 || j == 0 || j == self.rows - 1 {
                    if let Some(mut cell_type) = self.cell_types.get_at_mut(i as i32, j as i32) {
                        *cell_type = CellType::SOLID;
                    }
                }
            }
        }
    }

    pub fn horizontal_velocity(&self, i: i32, j: i32) -> Option<&f32> {
        self.horizontal_velocities.get_at(i, j)
    }

    pub fn vertical_velocity(&self, i: i32, j: i32) -> Option<&f32> {
        self.vertical_velocities.get_at(i, j)
    }

    pub fn cell_type(&self, i: i32, j: i32) -> Option<&CellType> {
        self.cell_types.get_at(i, j)
    }

    pub fn set_particle_cell_to_fluid(&mut self, point: Vec2) {
        let (i, j) = self.floor(point);
        if let Some(mut cell_type) = self.cell_types.get_at_mut(i, j) {
            *cell_type = CellType::FLUID;
        };
    }

    pub fn splat_velocities(&mut self, velocity: Vec2, point: Vec2) {
        self.splat_horizontal_velocity(velocity.x, point);
        self.splat_vertical_velocity(velocity.y, point);
    }

    pub fn normalize_velocities(&mut self) {
        StaggeredGrid::normalize_velocity_components(
            &mut self.horizontal_velocities,
            &self.sum_horizontal_weights,
        );
        StaggeredGrid::normalize_velocity_components(
            &mut self.vertical_velocities,
            &self.sum_vertical_weights,
        );
    }

    pub fn advect(&self, point: Vec2) -> Option<Vec2> {
        let (i, j) = self.floor(point);
        if i < 0 || i >= self.cols as i32 || j < 0 || j >= self.rows as i32 {
            return None;
        }

        Some(Vec2::new(
            self.advect_horizontal_velocity(point),
            self.advect_vertical_velocity(point),
        ))
    }

    fn advect_horizontal_velocity(&self, point: Vec2) -> f32 {
        let shifted_point = point - Vec2::new(0., self.cell_spacing / 2.);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        Self::get_velocity_component(i, j, &self.horizontal_velocities, weights[0]).unwrap_or(0.)
        + Self::get_velocity_component(i + 1, j, &self.horizontal_velocities, weights[1]).unwrap_or(0.)
        + Self::get_velocity_component(i +1, j+1, &self.horizontal_velocities, weights[2]).unwrap_or(0.)
        + Self::get_velocity_component(i, j+1, &self.horizontal_velocities, weights[3]).unwrap_or(0.)
    }

    fn advect_vertical_velocity(&self, point: Vec2) -> f32 {
        let shifted_point = point - Vec2::new( self.cell_spacing / 2., 0.);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        Self::get_velocity_component(i, j, &self.vertical_velocities, weights[0]).unwrap_or(0.)
            + Self::get_velocity_component(i + 1, j, &self.vertical_velocities, weights[1]).unwrap_or(0.)
            + Self::get_velocity_component(i +1, j+1, &self.vertical_velocities, weights[2]).unwrap_or(0.)
            + Self::get_velocity_component(i, j+1, &self.vertical_velocities, weights[3]).unwrap_or(0.)
    }

    fn set_velocity_component_to_zero(mut grid: &mut Grid<f32>, i: i32, j: i32) {
        if let Some(mut velocoity) = grid.get_at_mut(i, j) {
            *velocoity = 0.;
        }
    }

    fn copy_velocity_component(
        mut velocities: &mut Grid<f32>,
        src_i: i32,
        src_j: i32,
        dest_i: i32,
        dest_j: i32,
    ) {
        if let Some(&src_velocity) = velocities.get_at(src_i, src_j) {
            if let Some(mut dest_velocity) = velocities.get_at_mut(dest_i, dest_j) {
                *dest_velocity = src_velocity;
            }
        }
    }

    pub fn set_boundary_velocities(&mut self) {
        let cols = self.cols as i32;
        let rows = self.rows as i32;

        for i in 0..cols {
            for j in 0..rows {
                if i == 0 {
                    Self::set_velocity_component_to_zero(&mut self.horizontal_velocities, i, j);
                    Self::set_velocity_component_to_zero(&mut self.horizontal_velocities, i + 1, j);
                    Self::copy_velocity_component(&mut self.vertical_velocities, i + 1, j, i, j);
                }

                if i == cols - 1 {
                    Self::set_velocity_component_to_zero(&mut self.horizontal_velocities, i, j);
                    Self::set_velocity_component_to_zero(&mut self.horizontal_velocities, i - 1, j);
                    Self::copy_velocity_component(&mut self.vertical_velocities, i - 1, j, i, j);
                }

                if j == 0 {
                    Self::set_velocity_component_to_zero(&mut self.vertical_velocities, i, j);
                    Self::set_velocity_component_to_zero(&mut self.vertical_velocities, i, j + 1);
                    Self::copy_velocity_component(&mut self.horizontal_velocities, i, j + 1, i, j);
                }

                if j == rows - 1 {
                    Self::set_velocity_component_to_zero(&mut self.vertical_velocities, i, j);
                    Self::set_velocity_component_to_zero(&mut self.vertical_velocities, i, j - 1);
                    Self::copy_velocity_component(&mut self.horizontal_velocities, i, j - 1, i, j);
                }
            }
        }
    }

    fn normalize_velocity_components(
        mut velocity_components: &mut Grid<f32>,
        weight_sums: &Grid<f32>,
    ) {
        // Check in reference source. https://github.com/unusualinsights/flip_pic_examples/blob/main/incremental7/StaggeredGrid.cpp#L390
        // Should be doing: Set boundary velocities to zero.
        // Should be doing:  Normalize the non-boundary velocities (unless the corresponding velocity-weight is small).
        for (weight_sum, velocity_component) in
            weight_sums.iter().zip(velocity_components.iter_mut())
        {
            if *weight_sum <= f32::EPSILON {
                *velocity_component = 0.;
                continue;
            }

            *velocity_component /= *weight_sum;
        }
    }

    fn get_velocity_component(i: i32, j: i32, velocity_components: &Grid<f32>, weight: f32) -> Option<f32> {
        let magnitude = velocity_components.get_at(i, j)?;
        Some(magnitude * weight)
    }

    fn update_velocity_component(
        i: i32,
        j: i32,
        mut velocity_components: &mut Grid<f32>,
        weight_sums: &mut Grid<f32>,
        magnitude: f32,
        weight: f32,
    ) {
        if let Some(mut velocity_component) = velocity_components.get_at_mut(i, j) {
            *velocity_component += magnitude * weight;
        }

        if let Some(mut weight_sum) = weight_sums.get_at_mut(i, j) {
            *weight_sum += weight;
        }
    }

    fn corner_weights(&self, point: Vec2) -> [f32; 4] {
        let local_point = self.weights(point);

        let x_over_spacing = local_point.x / self.cell_spacing;
        let y_over_spacing = local_point.y / self.cell_spacing;
        let one_minus_x_over_spacing = 1. - x_over_spacing;
        let one_minus_y_over_spacing = 1. - y_over_spacing;

        [
            one_minus_x_over_spacing * one_minus_y_over_spacing,
            x_over_spacing * one_minus_y_over_spacing,
            x_over_spacing * y_over_spacing,
            one_minus_x_over_spacing * y_over_spacing,
        ]
    }

    fn splat_horizontal_velocity(&mut self, magnitude: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(0., self.cell_spacing / 2.);
        let weights = self.corner_weights(shifted_point);

        let (i, j) = self.floor(shifted_point);

        Self::update_velocity_component(
            i,
            j,
            &mut self.horizontal_velocities,
            &mut self.sum_horizontal_weights,
            magnitude,
            weights[0],
        );
        Self::update_velocity_component(
            i + 1,
            j,
            &mut self.horizontal_velocities,
            &mut self.sum_horizontal_weights,
            magnitude,
            weights[1],
        );
        Self::update_velocity_component(
            i + 1,
            j + 1,
            &mut self.horizontal_velocities,
            &mut self.sum_horizontal_weights,
            magnitude,
            weights[2],
        );
        Self::update_velocity_component(
            i,
            j + 1,
            &mut self.horizontal_velocities,
            &mut self.sum_horizontal_weights,
            magnitude,
            weights[3],
        );
    }

    fn splat_vertical_velocity(&mut self, magnitude: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(self.cell_spacing / 2., 0.);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        Self::update_velocity_component(
            i,
            j,
            &mut self.vertical_velocities,
            &mut self.sum_vertical_weights,
            magnitude,
            weights[0],
        );
        Self::update_velocity_component(
            i + 1,
            j,
            &mut self.vertical_velocities,
            &mut self.sum_vertical_weights,
            magnitude,
            weights[1],
        );
        Self::update_velocity_component(
            i + 1,
            j + 1,
            &mut self.vertical_velocities,
            &mut self.sum_vertical_weights,
            magnitude,
            weights[2],
        );
        Self::update_velocity_component(
            i,
            j + 1,
            &mut self.vertical_velocities,
            &mut self.sum_vertical_weights,
            magnitude,
            weights[3],
        );
    }

    fn floor(&self, point: Vec2) -> (i32, i32) {
        let Vec2 { x: i, y: j } = (point / self.cell_spacing).floor();
        (i as i32, j as i32)
    }

    fn weights(&self, point: Vec2) -> Vec2 {
        Vec2::new(point.x % self.cell_spacing, point.y % self.cell_spacing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::vec2;

    #[test]
    fn grid_indices_for_point() {
        let grid = StaggeredGrid::new(4, 4, 10., Vec2::splat(-20.));
        assert_eq!(grid.floor(Vec2::new(-20., 10.)), (0, 3));
        assert_eq!(grid.floor(Vec2::new(-20., 20.)), (0, 4));
        assert_eq!(grid.floor(Vec2::new(-21., 10.)), (-1, 3));
    }

    fn barycentric_weights_for_point() {
        let grid = StaggeredGrid::new(4, 4, 10., Vec2::splat(-20.));
        assert_eq!(grid.weights(Vec2::new(-20., 10.)), Vec2::new(0., 0.));
        assert_eq!(grid.weights(Vec2::new(-18., 14.)), Vec2::new(2., 4.));
    }

    #[test]
    fn corner_weights_for_point() {
        let grid = StaggeredGrid::new(2, 2, 10., Vec2::ZERO);
        assert_eq!(
            grid.corner_weights(vec2(7.5, 2.5)),
            [0.1875, 0.5625, 0.1875, 0.0625]
        );

        assert_eq!(
            grid.corner_weights(vec2(12.5, 17.5)),
            [0.1875, 0.0625, 0.1875, 0.5625,]
        );
    }

    #[test]
    fn splat_velocity_onto_grid() {
        let mut grid = StaggeredGrid::new(3, 3, 10., Vec2::new(-15., -15.));
        grid.splat_velocities(Vec2::new(10., -20.), Vec2::new(2.5, -2.5));

        assert_eq!(
            *grid.horizontal_velocities.get_at(1, 0).unwrap(),
            0.0625 * 10.
        );
        assert_eq!(
            *grid.horizontal_velocities.get_at(2, 0).unwrap(),
            0.1875 * 10.
        );
        assert_eq!(
            *grid.horizontal_velocities.get_at(2, 1).unwrap(),
            0.5625 * 10.
        );
        assert_eq!(
            *grid.horizontal_velocities.get_at(1, 1).unwrap(),
            0.1875 * 10.
        );

        assert_eq!(
            *grid.vertical_velocities.get_at(1, 1).unwrap(),
            0.5625 * -20.
        );
        assert_eq!(
            *grid.vertical_velocities.get_at(2, 1).unwrap(),
            0.1875 * -20.
        );
        assert_eq!(
            *grid.vertical_velocities.get_at(2, 2).unwrap(),
            0.0625 * -20.
        );
        assert_eq!(
            *grid.vertical_velocities.get_at(1, 2).unwrap(),
            0.1875 * -20.
        );
    }
}
