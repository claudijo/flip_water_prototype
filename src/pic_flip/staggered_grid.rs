use bevy::prelude::*;

use crate::pic_flip::grid::Grid;

#[derive(Clone, Default)]
pub enum CellType {
    #[default]
    EMPTY,
    FLUID,
    SOLID,
}

pub struct StaggeredGrid {
    cols: usize,
    rows: usize,
    lower_corner: Vec2,
    pressures: Grid<f32>,
    horizontal_velocities: Grid<f32>,
    vertical_velocities: Grid<f32>,
    cell_type: Grid<CellType>,
    cell_spacing: f32,
}

impl StaggeredGrid {
    pub fn new(cols: usize, rows: usize, cell_spacing: f32, lower_corner: Vec2) -> Self {
        Self {
            cols,
            rows,
            cell_type: Grid::new(cols, rows),
            pressures: Grid::new(cols, rows),
            horizontal_velocities: Grid::new(cols + 1, rows),
            vertical_velocities: Grid::new(cols, rows + 1),
            cell_spacing,
            lower_corner,
        }
    }

    fn floor(&self, point: Vec2) -> Option<(usize, usize)> {
        let Vec2 { x, y } = (point - self.lower_corner) / self.cell_spacing;
        let col = x as usize;
        let row = y as usize;
        if x < 0. || y < 0. || col > self.cols - 1 || row > self.rows - 1 {
            return None;
        }

        Some((col, row))
    }

    fn weights(&self, point: Vec2) -> Option<Vec2> {
        let (col, row) = self.floor(point)?;
        Some((point - self.lower_corner) - Vec2::new(col as f32, row as f32) * self.cell_spacing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_indices_for_point() {
        let grid = StaggeredGrid::new(4, 4, 10., Vec2::splat(-20.));
        assert_eq!(grid.floor(Vec2::new(-20., 10.)), Some((0, 3)));
        assert_eq!(grid.floor(Vec2::new(-20., 20.)), None);
        assert_eq!(grid.floor(Vec2::new(-21., 10.)), None);
    }

    fn barycentric_weights_for_point() {
        let grid = StaggeredGrid::new(4, 4, 10., Vec2::splat(-20.));
        assert_eq!(grid.weights(Vec2::new(-20., 10.)), Some(Vec2::new(0., 0.)));
        assert_eq!(grid.weights(Vec2::new(-18., 14.)), Some(Vec2::new(2., 4.)));
    }
}
