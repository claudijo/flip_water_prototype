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

    pub fn total_velocity_in_system(&self) -> Vec2 {
        let mut total_velocity = Vec2::ZERO;
        for col in 0..self.cols {
            for row in 0..self.rows {
                if let Some(cell) = self.cell_at(col as i32, row as i32) {
                    total_velocity.x += cell.horizontal_velocity.unwrap_or(0.);
                    total_velocity.y += cell.vertical_velocity.unwrap_or(0.);
                }
            }
        }

        total_velocity
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
        if point.x < 0.
            || point.y < 0.
            || point.x > self.cols as f32 * self.cell_size
            || point.y > self.rows as f32 * self.cell_size
        {
            return;
        }

        self.transfer_horizontal_velocity(point, velocity);
        self.transfer_vertical_velocity(point, velocity);
    }

    pub fn solve_incompressibility(&mut self, iterations: usize, over_relaxation: f32) {
        for _ in 0..iterations {
            for col in 0..self.cols {
                for row in 0..self.rows {
                    let i = col as i32;
                    let j = row as i32;

                    let divergence_scale_sum = self.divergence_scale_sum_for_cell_neighbours(i, j);

                    if divergence_scale_sum == 0. {
                        continue;
                    }

                    let divergence = over_relaxation * self.divergence_for_cell(i, j);

                    let a = self.divergence_scale_for_cell(i - 1, j).unwrap_or(0.);
                    let b = self.divergence_scale_for_cell(i, j - 1).unwrap_or(0.);
                    let c = self.divergence_scale_for_cell(i + 1, j).unwrap_or(0.);
                    let d = self.divergence_scale_for_cell(i, j + 1).unwrap_or(0.);

                    if let Some(mut cell) = self.cell_at_mut(i, j) {
                        let horizontal_velocity = cell.horizontal_velocity.unwrap_or(0.);
                        cell.horizontal_velocity = Some(
                            horizontal_velocity
                                + divergence * a
                                / divergence_scale_sum,
                        );

                        let vertical_velocity = cell.vertical_velocity.unwrap_or(0.);
                        cell.vertical_velocity = Some(
                            vertical_velocity
                                + divergence * b
                                / divergence_scale_sum,
                        );
                    }

                    if let Some(cell) = self.cell_at_mut(i + 1, j) {
                        let horizontal_velocity = cell.horizontal_velocity.unwrap_or(0.);
                        cell.horizontal_velocity = Some(
                            horizontal_velocity
                                - divergence * c
                                / divergence_scale_sum,
                        );
                    }

                    if let Some(cell) = self.cell_at_mut(i, j + 1) {
                        let vertical_velocity = cell.vertical_velocity.unwrap_or(0.);
                        cell.vertical_velocity = Some(
                            vertical_velocity
                                - divergence * d
                                / divergence_scale_sum,
                        );
                    }
                }
            }
        }
    }

    pub fn even_out_flow(&mut self) {
        for col in 0..self.cols {
            for row in 0..self.rows {

                if let Some(cell) = self.cell_at_mut(col as i32, row as i32) {
                    if let Some(flow) = cell.horizontal_velocity {
                        cell.horizontal_velocity = Some(flow / cell.sum_horizontal_velocity_weights);
                    }

                    if let Some(flow) = cell.vertical_velocity {
                        cell.vertical_velocity = Some(flow / cell.sum_vertical_velocity_weights);
                    }
                }
            }
        }
    }

    pub fn weighted_velocity_at_point(&self, point: Vec2) -> Option<Vec2> {
        // Ignore points outside grid
        if point.x < 0.
            || point.y < 0.
            || point.x > self.cols as f32 * self.cell_size
            || point.y > self.rows as f32 * self.cell_size
        {
            return None;
        }

        let horizontal_velocity = self.weighted_horizontal_velocity_at_point(point);
        let vertical_velocity = self.weighted_vertical_velocity_at_point(point);

        if horizontal_velocity.is_none() && vertical_velocity.is_none() {
            return None;
        }

        Some(Vec2::new(
            horizontal_velocity.unwrap_or(0.),
            vertical_velocity.unwrap_or(0.)
        ))
    }

    fn weighted_horizontal_velocity_at_point(&self, point: Vec2) -> Option<f32> {
        let adjusted_point = point - Vec2::new(0., self.cell_size / 2.);
        let weights = self.weights_for_point(adjusted_point);
        let (col, row) = self.col_row_for_point(adjusted_point);

        let mut nominator = 0.;
        let mut denominator = 0.;

        if let Some(cell) = self.cell_at(col, row) {
            if let Some(flow) = cell.horizontal_velocity {
                nominator += weights[0] * flow;
                denominator += weights[0];
            }
        };

        if let Some(cell) = self.cell_at(col + 1, row) {
            if let Some(flow) = cell.horizontal_velocity {
                nominator += weights[1] * flow;
                denominator += weights[1];
            }
        }

        if let Some(cell) = self.cell_at(col + 1, row + 1) {
            if let Some(flow) = cell.horizontal_velocity {
                nominator += weights[2] * flow;
                denominator += weights[2];
            }
        }

        if let Some(cell) = self.cell_at(col, row + 1) {
            if let Some(flow) = cell.horizontal_velocity {
                nominator += weights[3] * flow;
                denominator += weights[3];
            }
        }

        if denominator == 0. {
            return None;
        }

        Some(nominator / denominator)
    }

    fn weighted_vertical_velocity_at_point(&self, point: Vec2) -> Option<f32> {
        let adjusted_point = point - Vec2::new(self.cell_size / 2., 0.);
        let weights = self.weights_for_point(adjusted_point);
        let (col, row) = self.col_row_for_point(adjusted_point);

        let mut nominator = 0.;
        let mut denominator = 0.;

        if let Some(cell) = self.cell_at(col, row) {
            if let Some(flow) = cell.vertical_velocity {
                nominator += weights[0] * flow;
                denominator += weights[0];
            }
        };

        if let Some(cell) = self.cell_at(col + 1, row) {
            if let Some(flow) = cell.vertical_velocity {
                nominator += weights[1] * flow;
                denominator += weights[1];
            }
        };

        if let Some(cell) = self.cell_at(col + 1, row + 1) {
            if let Some(flow) = cell.vertical_velocity {
                nominator += weights[2] * flow;
                denominator += weights[2];
            }
        };

        if let Some(cell) = self.cell_at(col, row + 1) {
            if let Some(flow) = cell.vertical_velocity {
                nominator += weights[3] * flow;
                denominator += weights[3];
            }
        };

        if denominator == 0. {
            return None;
        }

        Some(nominator / denominator)
    }

    fn divergence_for_cell(&self, col: i32, row: i32) -> f32 {
        let mut divergence = 0.;

        if let Some(cell) = self.cell_at(col, row) {
            divergence -= cell.horizontal_velocity.unwrap_or(0.);
            divergence -= cell.vertical_velocity.unwrap_or(0.);
        }

        if let Some(cell) = self.cell_at(col + 1, row) {
            divergence += cell.horizontal_velocity.unwrap_or(0.);
        }

        if let Some(cell) = self.cell_at(col, row + 1) {
            divergence += cell.vertical_velocity.unwrap_or(0.);
        }

        divergence
    }

    fn divergence_scale_for_cell(&self, col: i32, row: i32) -> Option<f32> {
        let cell = self.cell_at(col, row)?;
        Some(cell.divergence_scale)
    }

    fn divergence_scale_sum_for_cell_neighbours(&self, col: i32, row: i32) -> f32 {
        self.divergence_scale_for_cell(col + 1, row).unwrap_or(0.)
            + self.divergence_scale_for_cell(col - 1, row).unwrap_or(0.)
            + self.divergence_scale_for_cell(col, row + 1).unwrap_or(0.)
            + self.divergence_scale_for_cell(col, row - 1).unwrap_or(0.)
    }

    pub fn col_row_for_point(&self, point: Vec2) -> (i32, i32) {
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
