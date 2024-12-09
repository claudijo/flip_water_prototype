use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Velocity(pub Vec2);

#[derive(Clone, Debug)]
pub struct Cell {
    pub vertical_velocity: Option<f32>,
    pub horizontal_velocity: Option<f32>,
    pub sum_vertical_velocity_weights: f32,
    pub sum_horizontal_velocity_weights: f32,
    pub divergence_scale: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            vertical_velocity: None,
            horizontal_velocity: None,
            sum_vertical_velocity_weights: 0.,
            sum_horizontal_velocity_weights: 0.,
            divergence_scale: 1.0,
        }
    }
}

#[derive(Component, Debug)]
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
                    self.cell_at_mut(col as i32, row as i32)
                        .unwrap()
                        .divergence_scale = 0.;
                }
            }
        }

        self
    }

    pub fn cell_at(&self, col: i32, row: i32) -> Option<&Cell> {
        if col < 0 || row < 0 || col > self.cols as i32 - 1 || row > self.rows as i32 - 1 {
            return None;
        }

        self.cells.get(row as usize * self.cols + col as usize)
    }

    pub fn cell_at_mut(&mut self, col: i32, row: i32) -> Option<&mut Cell> {
        if col < 0 || row < 0 || col > self.cols as i32 - 1 || row > self.rows as i32 - 1 {
            return None;
        }

        self.cells.get_mut(row as usize * self.cols + col as usize)
    }

    fn weights_for_point(&self, point: Vec2) -> [f32; 4] {
        let local_point = self.grid_local_point(point);

        let delta_x_by_size = local_point.x / self.cell_size;
        let delta_y_by_size = local_point.y / self.cell_size;
        let one_minus_delta_x_by_size = 1. - delta_x_by_size;
        let one_minus_delta_y_by_size = 1. - delta_y_by_size;

        [
            (one_minus_delta_x_by_size * one_minus_delta_y_by_size),
            (delta_x_by_size * one_minus_delta_y_by_size),
            (delta_x_by_size * delta_y_by_size),
            (one_minus_delta_x_by_size * delta_y_by_size),
        ]
    }

    pub fn clear_cells(&mut self) {
        self.cells.iter_mut().for_each(|cell| {
            cell.horizontal_velocity = None;
            cell.vertical_velocity = None;
            cell.sum_vertical_velocity_weights = 0.;
            cell.sum_horizontal_velocity_weights = 0.;
        });
    }

    pub fn transfer_velocities(&mut self, point: Vec2, velocity: Vec2) {
        // Ignore points outside grid
        if point.x < 0. || point.y < 0. || point.x > self.cols as f32 * self.cell_size || point.y > self.rows as f32 * self.cell_size {
            return;
        }

        self.transfer_horizontal_velocity(point, velocity);
        self.transfer_vertical_velocity(point, velocity);
    }

    pub fn solve_incompressibility(&mut self) {
        for col in 0..self.cols {
            for row in 0..self.rows {}
        }
    }

    fn divergence_for_cell(&self, col: i32, row: i32) -> Option<f32> {
        // let cell = self.cell_at(col, row)?;
        //
        // if  cell.horizontal_velocity.is_none() && cell.vertical_velocity.is_none() {
        //     return None;
        // }
        //
        // self.cell_at(col + 1, row)
        //
        // let horizontal_velocity = cell.horizontal_velocity;
        // let vertical_velocity = cell.vertical_velocity;
        //
        // let right_cell = self.cell_at(col + 1, row)
        //
        //
        // let d = self.cell_at(col + 1, row)?.horizontal_velocity.unwrap_or(0.) - self.cell_at(col, row)?.horizontal_velocity.unwrap_or(0.) + ;
        //

        None
    }

    fn col_row_for_point(&self, point: Vec2) -> (i32, i32) {
        // Note that casting from a float to an integer will round (negative) float towards zero, which
        // could be a foot gun if we accept negative point coordinates
        (
            (point.x / self.cell_size) as i32,
            (point.y / self.cell_size) as i32,
        )
    }

    fn grid_local_point(&self, point: Vec2) -> Vec2 {
        Vec2::new(point.x % self.cell_size, point.y % self.cell_size)
    }

    fn update_horizontal_velocity(
        &mut self,
        col: i32,
        row: i32,
        horizontal_velocity: f32,
        weight: f32,
    ) {
        if let Some(mut cell) = self.cell_at_mut(col, row) {
            let flow = cell.horizontal_velocity.unwrap_or(0.);
            cell.horizontal_velocity = Some(flow + horizontal_velocity * weight);
            cell.sum_horizontal_velocity_weights += weight;
        }
    }

    fn update_vertical_velocity(
        &mut self,
        col: i32,
        row: i32,
        vertical_velocity: f32,
        weight: f32,
    ) {
        if let Some(mut cell) = self.cell_at_mut(col, row) {
            let flow = cell.vertical_velocity.unwrap_or(0.);
            cell.vertical_velocity = Some(flow + vertical_velocity * weight);
            cell.sum_vertical_velocity_weights += weight;
        }
    }

    fn transfer_horizontal_velocity(&mut self, point: Vec2, velocity: Vec2) {
        let adjusted_point = point - Vec2::new(0., self.cell_size / 2.);

        let weights = self.weights_for_point(adjusted_point);
        let (col, row) = self.col_row_for_point(adjusted_point);

        self.update_horizontal_velocity(col, row, velocity.x, weights[0]);
        self.update_horizontal_velocity(col + 1, row, velocity.x, weights[1]);
        self.update_horizontal_velocity(col + 1, row + 1, velocity.x, weights[2]);
        self.update_horizontal_velocity(col, row + 1, velocity.x, weights[3]);
    }

    fn transfer_vertical_velocity(&mut self, point: Vec2, velocity: Vec2) {
        let adjusted_point = point - Vec2::new(self.cell_size / 2., 0.);
        let weight = self.weights_for_point(adjusted_point);
        let (col, row) = self.col_row_for_point(adjusted_point);

        self.update_vertical_velocity(col, row, velocity.y, weight[0]);
        self.update_vertical_velocity(col + 1, row, velocity.y, weight[1]);
        self.update_vertical_velocity(col + 1, row + 1, velocity.y, weight[2]);
        self.update_vertical_velocity(col, row + 1, velocity.y, weight[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::vec2;

    #[test]
    fn weights_for_point_works() {
        let grid = StaggeredGrid::new(2, 2, 10.);
        assert_eq!(
            grid.weights_for_point(vec2(7.5, 2.5)),
            [0.25 * 0.75, 0.75 * 0.75, 0.75 * 0.25, 0.25 * 0.25]
        );

        assert_eq!(
            grid.weights_for_point(vec2(12.5, 17.5)),
            [0.75 * 0.25, 0.25 * 0.25, 0.25 * 0.75, 0.75 * 0.75,]
        );
    }

    #[test]
    fn col_row_for_point_works() {
        let grid = StaggeredGrid::new(2, 2, 10.);
        assert_eq!(grid.col_row_for_point(vec2(7.5, 2.5)), (0, 0));
        assert_eq!(grid.col_row_for_point(vec2(12.5, 17.5)), (1, 1))
    }
}
