use bevy::prelude::*;

use crate::pic_flip::grid::Grid;

#[derive(Debug, Clone, Default)]
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

    pub fn splat_velocity(&mut self, velocity: Vec2, point: Vec2) {
        self.splat_horizontal_velocity(velocity.x, point);
        self.splat_vertical_velocity(velocity.y, point);
    }

    fn update_horizontal_velocity(&mut self, i: i32, j: i32, velocity_component: f32, weight: f32) {
        if let Some(mut horizontal_velocity) = self.horizontal_velocities.get_at_mut(i, j) {
            *horizontal_velocity += velocity_component * weight;
        }

        if let Some(mut sum_of_weights) = self.sum_horizontal_weights.get_at_mut(i, j) {
            *sum_of_weights += weight;
        }
    }

    fn update_vertical_velocity(&mut self, i: i32, j: i32, velocity_component: f32, weight: f32) {
        if let Some(mut vertical_velocity) = self.vertical_velocities.get_at_mut(i, j) {
            *vertical_velocity += velocity_component * weight;
        }

        if let Some(mut sum_of_weights) = self.sum_vertical_weights.get_at_mut(i, j) {
            *sum_of_weights += weight;
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

    fn splat_horizontal_velocity(&mut self, velocity_component: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(0., self.cell_spacing / 2.);
        let weights = self.corner_weights(shifted_point);

        let (i, j) = self.floor(shifted_point);

        self.update_horizontal_velocity(i, j, velocity_component, weights[0]);
        self.update_horizontal_velocity(i + 1, j, velocity_component, weights[1]);
        self.update_horizontal_velocity(i + 1, j + 1, velocity_component, weights[2]);
        self.update_horizontal_velocity(i, j + 1, velocity_component, weights[3]);
    }

    fn splat_vertical_velocity(&mut self, velocity_component: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(self.cell_spacing / 2., 0.);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        self.update_vertical_velocity(i, j, velocity_component, weights[0]);
        self.update_vertical_velocity(i + 1, j, velocity_component, weights[1]);
        self.update_vertical_velocity(i + 1, j + 1, velocity_component, weights[2]);
        self.update_vertical_velocity(i, j + 1, velocity_component, weights[3]);
    }

    fn floor(&self, point: Vec2) -> (i32, i32) {
        let Vec2 { x: i, y: j } = ((point - self.offset) / self.cell_spacing).floor();
        (i as i32, j as i32)
    }

    fn weights(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            (point.x - self.offset.x) % self.cell_spacing,
            (point.y - self.offset.y) % self.cell_spacing,
        )
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
    fn splatting_velocity() {
        let mut grid = StaggeredGrid::new(3, 3, 10., Vec2::new(-15., -15.));
        grid.splat_velocity(Vec2::new(10., -20.), Vec2::new(2.5, -2.5));

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
