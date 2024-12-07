use crate::flip::utils::corner_weight;
use bevy::prelude::*;
use bevy::render::render_resource::encase::private::RuntimeSizedArray;

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
    pub divergence_scale: f32, // 0 for solid cells, 1 for fluid cells.
}

impl Cell {
    pub fn clear(&mut self) {
        self.horizontal_flow = None;
        self.vertical_flow = None;
        self.sum_of_weights = Vec2::ZERO;
    }

    pub fn even_out_flow(&mut self) {
        if let Some(flow) = self.horizontal_flow {
            self.horizontal_flow = Some(flow / self.sum_of_weights.x);
        }

        if let Some(flow) = self.vertical_flow {
            self.vertical_flow = Some(flow / self.sum_of_weights.y);
        }
    }
}

pub enum Corner {
    BottomLeft,
    BottomRight,
    TopRight,
    TopLeft,
}

const ALL_CORNERS: [Corner; 4] = [
    Corner::BottomLeft,
    Corner::BottomRight,
    Corner::TopRight,
    Corner::TopLeft,
];

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

    pub fn corner_weight(&self, corner: &Corner, offset: Vec2) -> f32 {
        match corner {
            Corner::BottomLeft => {
                (1. - offset.x / self.cell_size) * (1. - offset.y / self.cell_size)
            }
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
            return None;
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

    pub fn weighted_sum(&self, corner_values: Vec<Option<f32>>, point_offset: Vec2) -> f32 {
        let mut numerator = 0.;
        let mut denominator = 0.;

        for (i, corner_value) in corner_values.iter().enumerate() {
            if let Some(value) = corner_value {
                let weight = self.corner_weight(&ALL_CORNERS[i], point_offset);
                numerator += weight * value;
                denominator += weight;
            }
        }

        if denominator == 0. {
            return 0.;
        }

        numerator / denominator
    }

    pub fn weighted_horizontal_velocity_at_point(&self, point: Vec2) -> f32 {
        let shifted_down_point = point + self.step_down() / 2.;
        let local_offset = self.local_offset(shifted_down_point);

        let corner_values = [
            self.cell_at(shifted_down_point),
            self.cell_at(shifted_down_point + self.step_right()),
            self.cell_at(shifted_down_point + self.step_right() + self.step_up()),
            self.cell_at(shifted_down_point + self.step_up()),
        ]
        .iter()
        .map(|cell| (*cell)?.horizontal_flow)
        .collect::<Vec<_>>();

        self.weighted_sum(corner_values, local_offset)
    }

    pub fn weighted_vertical_velocity_at_point(&self, point: Vec2) -> f32 {
        let shifted_left_point = point + self.step_left() / 2.;
        let local_offset = self.local_offset(shifted_left_point);

        let corner_values = [
            self.cell_at(shifted_left_point + self.step_down()),
            self.cell_at(shifted_left_point + self.step_right() + self.step_down()),
            self.cell_at(shifted_left_point + self.step_right()),
            self.cell_at(shifted_left_point),
        ]
        .iter()
        .map(|cell| (*cell)?.vertical_flow)
        .collect::<Vec<_>>();

        self.weighted_sum(corner_values, local_offset)
    }

    fn accumulate_horizontal_flow_from_particle(&mut self, point: Vec2, flow_component: f32) {
        let shifted_down_point = point + self.step_down() / 2.;
        let local_offset = self.local_offset(shifted_down_point);

        let corner_weights = ALL_CORNERS.map(|corner| self.corner_weight(&corner, local_offset));

        for (i, point) in [
            shifted_down_point,
            shifted_down_point + self.step_right(),
            shifted_down_point + self.step_right() + self.step_up(),
            shifted_down_point + self.step_up(),
        ]
        .iter()
        .enumerate()
        {
            if let Some(mut cell) = self.cell_at_mut(*point) {
                let weight = corner_weights[i];
                cell.sum_of_weights += weight;
                if let Some(mut directional_flow) = cell.horizontal_flow {
                    cell.horizontal_flow = Some(directional_flow + flow_component * weight);
                } else {
                    cell.horizontal_flow = Some(flow_component * weight);
                }
            }
        }
    }

    fn accumulate_vertical_flow_from_particle(&mut self, point: Vec2, flow_component: f32) {
        let shifted_left_point = point + self.step_left() / 2.;
        let local_offset = self.local_offset(shifted_left_point);

        let corner_weights = ALL_CORNERS.map(|corner| self.corner_weight(&corner, local_offset));

        for (i, point) in [
            shifted_left_point + self.step_down(),
            shifted_left_point + self.step_down() + self.step_right(),
            shifted_left_point + self.step_right(),
            shifted_left_point,
        ]
        .iter()
        .enumerate()
        {
            if let Some(mut cell) = self.cell_at_mut(*point) {
                let weight = corner_weights[i];
                cell.sum_of_weights += weight;
                if let Some(mut directional_flow) = cell.vertical_flow {
                    cell.vertical_flow = Some(directional_flow + flow_component * weight);
                } else {
                    cell.vertical_flow = Some(flow_component * weight);
                }
            }
        }
    }

    pub fn accumulate_flow_from_particle(&mut self, point: Vec2, flow: Vec2) {
        self.accumulate_horizontal_flow_from_particle(point, flow.x);
        self.accumulate_vertical_flow_from_particle(point, flow.y);
    }

    pub fn even_out_flow_for_cell(&mut self, col: usize, row: usize) {
        if let Some(rows) = self.cells.get_mut(col) {
            if let Some(cell) = rows.get_mut(row) {
                cell.even_out_flow();
            }
        }
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

    pub fn divergence_for_cell(&self, col: usize, row: usize) -> Option<f32> {
        if col >= self.cols() {
            return None;
        }

        if row >= self.rows() {
            return None;
        }

        let cell = self.cells[col][row];

        // Solid cell
        if cell.divergence_scale == 0. {
            return None;
        }

        let left = cell.horizontal_flow.unwrap_or(0.);
        let up = cell.vertical_flow.unwrap_or(0.);

        let down = if row > 0 {
            self.cells[col][row - 1].vertical_flow.unwrap_or(0.)
        } else {
            0.
        };

        let right = if col + 1 < self.cols() {
            self.cells[col + 1][row].horizontal_flow.unwrap_or(0.)
        } else {
            0.
        };

        Some(right - left + down - up)
    }

    fn divergence_scale_for_cell(&self, col: usize, row: usize) -> f32 {
        let Some(rows) = self.cells.get(col) else {
            return 0.;
        };

        let Some(cell) = rows.get(row) else {
            return 0.;
        };

        cell.divergence_scale
    }

    pub fn sum_divergence_scale_for_neighboring_cells(&self, col: usize, row: usize) -> f32 {
        self.divergence_scale_for_cell(col - 1, row)
            + self.divergence_scale_for_cell(col + 1, row)
            + self.divergence_scale_for_cell(col, row - 1)
            + self.divergence_scale_for_cell(col, row + 1)
    }

    pub fn project_flow_for_cell(
        &mut self,
        col: usize,
        row: usize,
        iterations: usize,
        over_relaxation: f32,
    ) {
        for _ in 0..iterations {
            let Some(mut divergence) = self.divergence_for_cell(col, row) else {
                return;
            };

            divergence *= over_relaxation;

            let divergence_scale_left = self.divergence_scale_for_cell(col - 1, row);
            let divergence_scale_right = self.divergence_scale_for_cell(col + 1, row);
            let divergence_scale_up = self.divergence_scale_for_cell(col, row + 1);
            let divergence_scale_down = self.divergence_scale_for_cell(col, row - 1);

            let sum_divergence_scale_for_neighboring_cells = divergence_scale_left
                + divergence_scale_right
                + divergence_scale_up
                + divergence_scale_down;

            if sum_divergence_scale_for_neighboring_cells == 0. {
                return;
            }

            if let Some(rows) = self.cells.get_mut(col) {
                if let Some(cell) = rows.get_mut(row) {
                    // Flow to cell from left
                    let horizontal_flow = cell.horizontal_flow.unwrap_or(0.);
                    cell.horizontal_flow = Some(
                        horizontal_flow
                            + divergence * divergence_scale_left
                                / sum_divergence_scale_for_neighboring_cells,
                    );

                    // Flow to cell from up
                    let vertical_flow = cell.vertical_flow.unwrap_or(0.);
                    cell.vertical_flow = Some(
                        vertical_flow
                            + divergence * divergence_scale_up
                                / sum_divergence_scale_for_neighboring_cells,
                    );
                }

                if let Some(cell_down) = rows.get_mut(row - 1) {
                    // Flow to cell from down
                    let vertical_flow = cell_down.vertical_flow.unwrap_or(0.);
                    cell_down.vertical_flow = Some(
                        vertical_flow
                            - divergence * divergence_scale_down
                                / sum_divergence_scale_for_neighboring_cells,
                    );
                }
            }

            if let Some(rows) = self.cells.get_mut(col + 1) {
                if let Some(cell) = rows.get_mut(row) {
                    // Flow to cell from right
                    let horizontal_flow = cell.horizontal_flow.unwrap_or(0.);
                    cell.horizontal_flow = Some(
                        horizontal_flow
                            - divergence * divergence_scale_right
                                / sum_divergence_scale_for_neighboring_cells,
                    );
                }
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

        assert_eq!(
            grid.col_row_from_cell_position(Vec2::new(-1., 0.))
                .is_none(),
            true
        );
        assert_eq!(
            grid.col_row_from_cell_position(Vec2::new(0., -1.))
                .is_none(),
            true
        );
        assert_eq!(
            grid.col_row_from_cell_position(Vec2::new(3., 0.)).is_none(),
            true
        );
        assert_eq!(
            grid.col_row_from_cell_position(Vec2::new(0., 2.)).is_none(),
            true
        );
        assert_eq!(
            grid.col_row_from_cell_position(Vec2::new(2., 1.)),
            Some((2, 1))
        );
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
        grid.cell_at_mut(Vec2::new(15., 5.))
            .unwrap()
            .horizontal_flow = Some(20.);
        grid.cell_at_mut(Vec2::new(15., 15.))
            .unwrap()
            .horizontal_flow = Some(30.);
        grid.cell_at_mut(Vec2::new(5., 15.))
            .unwrap()
            .horizontal_flow = Some(40.);

        assert_eq!(
            grid.weighted_horizontal_velocity_at_point(Vec2::new(2.5, 12.5)),
            31.25
        );
        assert_eq!(
            grid.weighted_horizontal_velocity_at_point(Vec2::new(2.5, 7.5)),
            18.75
        );
        assert_eq!(
            grid.weighted_horizontal_velocity_at_point(Vec2::new(2.5, 2.5)),
            12.5
        );
    }

    #[test]
    fn weighted_vertical_sum_for_point_works() {
        let mut grid = StaggeredGrid::new(2, 2).with_cell_size(10.);

        grid.cell_at_mut(Vec2::new(5., 5.)).unwrap().vertical_flow = Some(10.);
        grid.cell_at_mut(Vec2::new(15., 5.)).unwrap().vertical_flow = Some(20.);
        grid.cell_at_mut(Vec2::new(15., 15.)).unwrap().vertical_flow = Some(30.);
        grid.cell_at_mut(Vec2::new(5., 15.)).unwrap().vertical_flow = Some(40.);

        assert_eq!(
            grid.weighted_vertical_velocity_at_point(Vec2::new(12.5, 17.5)),
            28.75
        );
        assert_eq!(
            grid.weighted_vertical_velocity_at_point(Vec2::new(7.5, 17.5)),
            31.25
        );
        assert_eq!(
            grid.weighted_vertical_velocity_at_point(Vec2::new(2.5, 17.5)),
            32.5
        );
    }

    #[test]
    fn accumulate_horizontal_flow_from_particle_works() {
        let mut grid = StaggeredGrid::new(2, 2).with_cell_size(10.);
        grid.accumulate_horizontal_flow_from_particle(Vec2::new(7.5, 12.5), 2.);

        assert_eq!(
            grid.cell_at(Vec2::new(5., 5.)).unwrap().horizontal_flow,
            Some(0.125)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(5., 5.)).unwrap().sum_of_weights.x,
            0.0625
        );

        assert_eq!(
            grid.cell_at(Vec2::new(15., 5.)).unwrap().horizontal_flow,
            Some(0.375)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(15., 5.)).unwrap().sum_of_weights.x,
            0.1875
        );

        assert_eq!(
            grid.cell_at(Vec2::new(15., 15.)).unwrap().horizontal_flow,
            Some(1.125)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(15., 15.)).unwrap().sum_of_weights.x,
            0.5625
        );

        assert_eq!(
            grid.cell_at(Vec2::new(5., 15.)).unwrap().horizontal_flow,
            Some(0.375)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(5., 15.)).unwrap().sum_of_weights.x,
            0.1875
        );
    }

    #[test]
    fn accumulate_vertical_flow_from_particle_works() {
        let mut grid = StaggeredGrid::new(2, 2).with_cell_size(10.);
        grid.accumulate_vertical_flow_from_particle(Vec2::new(12.5, 17.5), 2.);

        assert_eq!(
            grid.cell_at(Vec2::new(5., 5.)).unwrap().vertical_flow,
            Some(0.125)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(5., 5.)).unwrap().sum_of_weights.x,
            0.0625
        );

        assert_eq!(
            grid.cell_at(Vec2::new(15., 5.)).unwrap().vertical_flow,
            Some(0.375)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(15., 5.)).unwrap().sum_of_weights.x,
            0.1875
        );

        assert_eq!(
            grid.cell_at(Vec2::new(15., 15.)).unwrap().vertical_flow,
            Some(1.125)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(15., 15.)).unwrap().sum_of_weights.x,
            0.5625
        );

        assert_eq!(
            grid.cell_at(Vec2::new(5., 15.)).unwrap().vertical_flow,
            Some(0.375)
        );
        assert_eq!(
            grid.cell_at(Vec2::new(5., 15.)).unwrap().sum_of_weights.x,
            0.1875
        );
    }
}
