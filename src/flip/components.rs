use bevy::prelude::*;
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use crate::flip::utils::corner_weight;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
#[require(Velocity, Transform)]
pub struct SizedParticle(pub f32);

impl Default for SizedParticle {
    fn default() -> Self {
        Self(1.)
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Cell {
    pub horizontal_flow: Option<f32>,
    pub vertical_flow: Option<f32>,
    pub sum_of_weights: Vec2,
    pub s: f32, // 0 for solid cells, 1 for fluid cells. TODO: Check if we should change type for this field.
}

impl Cell {
    pub fn clear(&mut self) {
        self.horizontal_flow = None;
        self.vertical_flow = None;
        self.sum_of_weights = Vec2::ZERO;
    }
}

pub enum Corner {
    BottomLeft,
    BottomRight,
    TopRight,
    TopLeft
}

#[derive(Component, Default, Debug)]
pub struct StaggeredGrid {
    pub cells: Vec<Vec<Cell>>,
    pub cell_size: f32,
}

impl StaggeredGrid {
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            cells: vec![vec![Cell::default(); rows as usize]; cols as usize],
            cell_size: 1.,
        }
    }

    pub fn with_cell_size(mut self, value: f32) -> Self {
        self.cell_size = value;
        self
    }

    pub fn corner_weight(&self, corner: Corner, offset: Vec2) -> f32 {
        match corner {
            Corner::BottomLeft => (1. - offset.x / self.cell_size) * (1. - offset.y / self.cell_size),
            Corner::BottomRight => offset.x / self.cell_size * (1. - offset.y / self.cell_size),
            Corner::TopRight => (offset.x / self.cell_size) * (offset.y / self.cell_size),
            Corner::TopLeft => (1. - offset.x / self.cell_size) * (offset.y / self.cell_size),
        }
    }

    pub fn cols(&self) -> usize {
        self.cells.len()
    }

    pub fn rows(&self) -> usize {
        if self.cols() == 0 {
            0
        } else {
            self.cells[0].len()
        }
    }

    pub fn col_row_from_cell_position(&self, cell_position: Vec2) -> Option<(usize, usize)> {
        if cell_position.x < 0. || cell_position.y < 0. {
            return None
        }

        let col = cell_position.x as usize;
        if col + 1 > self.cols() {
            return None;
        }

        let row = cell_position.y as usize;
        if row + 1 > self.rows() {
            return None;
        }

        Some((col, row))
    }

    pub fn cell_at(&self, point: Vec2) -> Option<&Cell> {
        let cell_position = self.cell_position_from_point(point);
        let (col, row) = self.col_row_from_cell_position(cell_position)?;
        Some(&self.cells[col][row])
    }

    pub fn cell_at_mut(&mut self, point: Vec2) -> Option<&mut Cell> {
        let cell_position = self.cell_position_from_point(point);
        let (col, row) = self.col_row_from_cell_position(cell_position)?;
        Some(&mut self.cells[col][row])
    }

    pub fn cell_position_from_point(&self, point: Vec2) -> Vec2 {
        (point / self.cell_size).floor()
    }

    pub fn local_offset(&self, point: Vec2) -> Vec2 {
        point - self.cell_position_from_point(point) * self.cell_size
    }

    const fn step_right(&self) -> Vec2 {
        Vec2::new(self.cell_size, 0.)
    }

    const fn step_left(&self) -> Vec2 {
        Vec2::new(-self.cell_size, 0.)
    }

    const fn step_down(&self) -> Vec2 {
        Vec2::new(0., -self.cell_size)
    }

    const fn step_up(&self) -> Vec2 {
        Vec2::new(0., self.cell_size)
    }

    pub fn weighted_horizontal_sum_for_point(&self, point: Vec2) -> f32 {
        let shifted_down_point = point + self.step_down() / 2.;
        let local_offset = self.local_offset(shifted_down_point);

        let mut numerator = 0.;
        let mut denominator = 0.;

        for (cell, corner) in [
            (self.cell_at(shifted_down_point), Corner::BottomLeft),
            (self.cell_at(shifted_down_point + self.step_right()), Corner::BottomRight),
            (self.cell_at(shifted_down_point + self.step_right() + self.step_up()), Corner::TopRight),
            (self.cell_at(shifted_down_point + self.step_up()), Corner::TopLeft),
        ] {
            if let Some(cell) = cell {
                if let Some(velocity) = cell.horizontal_flow {
                    let weight =  self.corner_weight(corner, local_offset);
                    numerator += weight * velocity;
                    denominator += weight;
                }
            }
        }

        if denominator == 0. {
            return 0.;
        }

        numerator / denominator
    }

    pub fn weighted_vertical_sum_for_point(&self, point: Vec2) -> f32 {
        let shifted_left_point = point + self.step_left() / 2.;
        let local_offset = self.local_offset(shifted_left_point);

        let mut numerator = 0.;
        let mut denominator = 0.;

        for (cell, corner) in [
            (self.cell_at(shifted_left_point), Corner::TopLeft),
            (self.cell_at(shifted_left_point + self.step_right()), Corner::TopRight),
            (self.cell_at(shifted_left_point + self.step_right() + self.step_down()), Corner::BottomRight),
            (self.cell_at(shifted_left_point + self.step_down()), Corner::BottomLeft),
        ] {
            if let Some(cell) = cell {
                if let Some(velocity) = cell.vertical_flow {
                    let weight =  self.corner_weight(corner, local_offset);
                    numerator += weight * velocity;
                    denominator += weight;
                }
            }
        }

        if denominator == 0. {
            return 0.;
        }

        numerator / denominator
    }

    pub fn clear_cells(&mut self) {
        let cols = self.cols();
        let rows = self.rows();

        for col in 0..cols {
            for row in 0..rows {
                self.cells[col][row].clear();
            }
        }
    }
}

#[derive(Component, Default)]
#[require(Transform, StaggeredGrid)]
pub struct LiquidContainer {
    pub width: f32,
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_coordinates_works() {
        let grid = StaggeredGrid::new(3, 2).with_cell_size(10.);
        let point = Vec2::new(26., 14.);
        let col_row = grid.cell_position_from_point(point);
        assert_eq!(col_row, Vec2::new(2., 1.));
    }

    #[test]
    fn local_offset_work() {
        let grid = StaggeredGrid::new(3, 2).with_cell_size(10.);
        let point = Vec2::new(26., 14.);
        let offset = grid.local_offset(point);
        assert_eq!(offset, Vec2::new(6., 4.));
    }

    #[test]
    fn cols_work() {
        let grid = StaggeredGrid::default();
        assert_eq!(grid.cols(), 0);

        let grid = StaggeredGrid::new(3, 2);
        assert_eq!(grid.cols(), 3);
    }

    #[test]
    fn rows_work() {
        let grid = StaggeredGrid::default();
        assert_eq!(grid.rows(), 0);

        let grid = StaggeredGrid::new(3, 2);
        assert_eq!(grid.rows(), 2);
    }

    #[test]
    fn col_row_from_cell_position_works() {
        let grid = StaggeredGrid::new(3, 2);

        assert_eq!(grid.col_row_from_cell_position(Vec2::new(-1., 0.)).is_none(), true);
        assert_eq!(grid.col_row_from_cell_position(Vec2::new(0., -1.)).is_none(), true);
        assert_eq!(grid.col_row_from_cell_position(Vec2::new(3., 0.)).is_none(), true);
        assert_eq!(grid.col_row_from_cell_position(Vec2::new(0., 2.)).is_none(), true);
        assert_eq!(grid.col_row_from_cell_position(Vec2::new(2., 1.)), Some((2, 1)));
    }

    #[test]
    fn mutate_and_access_cell() {
        let mut grid = StaggeredGrid::new(3, 2);
        let point = Vec2::new(2., 1.);
        let cell = grid.cell_at_mut(point).unwrap();
        cell.horizontal_flow = Some(10.);

        assert_eq!(grid.cell_at(point).unwrap().horizontal_flow, Some(10.));
    }

    #[test]
    fn weighted_horizontal_sum_for_point_works() {
        let mut grid = StaggeredGrid::new(2, 2).with_cell_size(10.);

        grid.cell_at_mut(Vec2::new(5., 5.)).unwrap().horizontal_flow = Some(10.);
        grid.cell_at_mut(Vec2::new(15., 5.)).unwrap().horizontal_flow = Some(20.);
        grid.cell_at_mut(Vec2::new(15., 15.)).unwrap().horizontal_flow = Some(30.);
        grid.cell_at_mut(Vec2::new(5., 15.)).unwrap().horizontal_flow = Some(40.);

        assert_eq!(grid.weighted_horizontal_sum_for_point(Vec2::new(2.5, 12.5)), 31.25);
        assert_eq!(grid.weighted_horizontal_sum_for_point(Vec2::new(2.5, 7.5)), 18.75);
        assert_eq!(grid.weighted_horizontal_sum_for_point(Vec2::new(2.5, 2.5)), 12.5);
    }

    #[test]
    fn weighted_vertical_sum_for_point_works() {
        let mut grid = StaggeredGrid::new(2, 2).with_cell_size(10.);

        grid.cell_at_mut(Vec2::new(5., 5.)).unwrap().vertical_flow = Some(10.);
        grid.cell_at_mut(Vec2::new(15., 5.)).unwrap().vertical_flow = Some(20.);
        grid.cell_at_mut(Vec2::new(15., 15.)).unwrap().vertical_flow = Some(30.);
        grid.cell_at_mut(Vec2::new(5., 15.)).unwrap().vertical_flow = Some(40.);

        assert_eq!(grid.weighted_vertical_sum_for_point(Vec2::new(12.5, 17.5)), 28.75);
        assert_eq!(grid.weighted_vertical_sum_for_point(Vec2::new(7.5, 17.5)), 31.25);
        assert_eq!(grid.weighted_vertical_sum_for_point(Vec2::new(2.5, 17.5)), 32.5);
    }
}