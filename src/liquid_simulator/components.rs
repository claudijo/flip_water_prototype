use crate::liquid_simulator::grid::Grid;
use crate::liquid_simulator::spatial_hash::SpatialHash;
use bevy::prelude::*;

#[derive(Component)]
pub struct LiquidParticle;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum CellType {
    #[default]
    Empty,
    Fluid,
    Solid,
    OutOfBounds,
}

#[derive(Component)]
pub struct LiquidSimulator {
    pub width: f32,
    pub height: f32,
    pub cols: usize,
    pub rows: usize,
    pub spacing: f32,
    pub offset: Vec2,
    pub particle_positions: Vec<Vec2>,
    pub particle_velocities: Vec<Vec2>,
    pub particle_radius: f32,
    pub spacial_hash: SpatialHash,
    pub horizontal_velocities: Grid<f32>,
    pub prev_horizontal_velocities: Grid<f32>,
    pub sum_horizontal_weights: Grid<f32>,
    pub vertical_velocities: Grid<f32>,
    pub prev_vertical_velocities: Grid<f32>,
    pub sum_vertical_weights: Grid<f32>,
    pub cell_types: Grid<CellType>,
    pub densities: Grid<f32>,
    pub normalized_horizontal_velocities: Grid<f32>,
    pub normalized_vertical_velocities: Grid<f32>,
    pub s: Grid<f32>, // 0 -> EMPTY or LIQUID , 1 -> Solid
}

impl LiquidSimulator {
    pub fn new(
        particle_positions: Vec<Vec2>,
        particle_radius: f32,
        cols: usize,
        rows: usize,
        spacing: f32,
    ) -> Self {
        let particle_count = particle_positions.len();
        let width = cols as f32 * spacing;
        let height = rows as f32 * spacing;

        Self {
            width,
            height,
            cols,
            rows,
            spacing,
            spacial_hash: SpatialHash::from_sizes(width, height, particle_radius),
            offset: Vec2::ZERO,
            particle_positions,
            particle_velocities: vec![Vec2::default(); particle_count],
            particle_radius,
            horizontal_velocities: Grid::new(cols + 1, rows),
            prev_horizontal_velocities: Grid::new(cols + 1, rows),
            sum_horizontal_weights: Grid::new(cols + 1, rows),
            vertical_velocities: Grid::new(cols, rows + 1),
            prev_vertical_velocities: Grid::new(cols, rows + 1),
            sum_vertical_weights: Grid::new(cols, rows + 1),
            cell_types: Grid::new(cols, rows),
            densities: Grid::new(cols + 1, rows + 1),
            normalized_horizontal_velocities: Grid::new(cols + 1, rows),
            normalized_vertical_velocities: Grid::new(cols, rows + 1),
            s: Grid::new(cols, rows).with_default_value(1.),
        }
    }

    pub fn with_solid_border_cells(mut self) -> Self {
        self.set_border_cells_to_solid();
        self
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.set_offset(offset);
        self
    }

    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
        self.spacial_hash.set_offset(offset);
    }

    fn set_cell_to_solid(&mut self, i: i32, j: i32) {
        if let Some(mut value) = self.s.get_mut(i, j) {
            *value = 0.; // Solid
        }
    }

    fn set_border_cells_to_solid(&mut self) {
        for i in 0..self.cols {
            for j in 0..self.rows {
                if i == 0 || i == self.cols - 1 || j == 0 || j == self.rows - 1 {
                    self.set_cell_to_solid(i as i32, j as i32)
                }
            }
        }
    }

    pub fn integrate_particles(&mut self, delta_time: f32, gravity: Vec2) {
        for (velocity, position) in self
            .particle_velocities
            .iter_mut()
            .zip(self.particle_positions.iter_mut())
        {
            *velocity += gravity * delta_time;
            *position += *velocity * delta_time;
        }
    }

    fn separate_particles(
        &mut self,
        a: usize,
        b: usize,
        collision_normal: Vec2,
        collision_depth: f32,
    ) {
        self.particle_positions[a] -= collision_normal * collision_depth * 0.5;
        self.particle_positions[b] += collision_normal * collision_depth * 0.5;
    }

    pub fn push_particles_apart(&mut self, iterations: usize) {
        let min_distance = 2. * self.particle_radius;
        let min_distance_squared = min_distance * min_distance;

        for _ in 0..iterations {
            self.spacial_hash.populate(&self.particle_positions);

            for a in 0..self.particle_positions.len() {
                for b in self.spacial_hash.query(self.particle_positions[a]) {
                    if a == b {
                        continue;
                    }

                    let first_position = self.particle_positions[a];
                    let second_position = self.particle_positions[b];
                    let distance_squared = first_position.distance_squared(second_position);

                    if distance_squared >= min_distance_squared || distance_squared <= f32::EPSILON
                    {
                        continue;
                    }

                    let collision_normal = (second_position - first_position).normalize();
                    let collision_depth = min_distance - distance_squared.sqrt();

                    self.separate_particles(a, b, collision_normal, collision_depth);
                }
            }
        }
    }

    pub fn handle_particle_collisions(&mut self) {
        for (point, velocity) in self
            .particle_positions
            .iter_mut()
            .zip(self.particle_velocities.iter_mut())
        {
            // Clamp particle positions within boundaries
            let min_y = self.offset.y + self.particle_radius;
            let max_y = self.height + self.offset.y - self.particle_radius;
            let min_x = self.offset.x + self.particle_radius;
            let max_x = self.width + self.offset.x - self.particle_radius;

            if point.y < min_y {
                point.y = min_y;
                velocity.y = 0.;
            }

            if point.y > max_y {
                point.y = max_y;
                velocity.y = 0.;
            }

            if point.x < min_x {
                point.x = min_x;
                velocity.x = 0.;
            }

            if point.x > max_x {
                point.x = max_x;
                velocity.x = 0.;
            }
        }
    }

    fn floor(&self, point: Vec2) -> (i32, i32) {
        let Vec2 { x: i, y: j } = (point / self.spacing).floor();
        (i as i32, j as i32)
    }

    fn remainder(&self, point: Vec2) -> Vec2 {
        Vec2::new(point.x % self.spacing, point.y % self.spacing)
    }

    fn mark_occupied_cells_as_fluid(&mut self, point: Vec2) {
        let (i, j) = self.floor(point);
        if let Some(mut cell_type) = self.cell_types.get_mut(i, j) {
            if *cell_type == CellType::Empty {
                *cell_type = CellType::Fluid;
            }
        };
    }

    fn corner_weights(&self, point: Vec2) -> [f32; 4] {
        let point_remainder = self.remainder(point);

        let x_over_spacing = point_remainder.x / self.spacing;
        let y_over_spacing = point_remainder.y / self.spacing;

        let one_minus_x_over_spacing = 1. - x_over_spacing;
        let one_minus_y_over_spacing = 1. - y_over_spacing;

        [
            one_minus_x_over_spacing * one_minus_y_over_spacing,
            x_over_spacing * one_minus_y_over_spacing,
            x_over_spacing * y_over_spacing,
            one_minus_x_over_spacing * y_over_spacing,
        ]
    }

    fn update_velocity_component(
        i: i32,
        j: i32,
        mut velocity_components: &mut Grid<f32>,
        weight_sums: &mut Grid<f32>,
        magnitude: f32,
        weight: f32,
    ) {
        if let Some(mut velocity_component) = velocity_components.get_mut(i, j) {
            *velocity_component += magnitude * weight;
        }

        if let Some(mut weight_sum) = weight_sums.get_mut(i, j) {
            *weight_sum += weight;
        }
    }

    fn splat_vertical_velocity(&mut self, magnitude: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(self.spacing / 2., 0.);
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

    fn splat_horizontal_velocity(&mut self, magnitude: f32, point: Vec2) {
        let shifted_point = point - Vec2::new(0., self.spacing / 2.);
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

    fn splat_velocities(&mut self, velocity: Vec2, point: Vec2) {
        self.splat_horizontal_velocity(velocity.x, point);
        self.splat_vertical_velocity(velocity.y, point);
    }

    fn update_density(&mut self, i: i32, j: i32, weight: f32) {
        if let Some(density) = self.densities.get_mut(i, j) {
            *density += weight;
        }
    }

    pub fn splat_density(&mut self, point: Vec2) {
        let weights = self.corner_weights(point);
        let (i, j) = self.floor(point);

        self.update_density(i, j, weights[0]);
        self.update_density(i + 1, j, weights[1]);
        self.update_density(i + 1, j + 1, weights[2]);
        self.update_density(i, j + 1, weights[3]);
    }

    fn normalize_velocity_components(
        mut velocity_components: &mut Grid<f32>,
        weight_sums: &Grid<f32>,
    ) {
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

    fn normalize_velocities(&mut self) {
        Self::normalize_velocity_components(
            &mut self.horizontal_velocities,
            &self.sum_horizontal_weights,
        );

        Self::normalize_velocity_components(
            &mut self.vertical_velocities,
            &self.sum_vertical_weights,
        );
    }

    fn set_velocity_component_to_zero(mut grid: &mut Grid<f32>, i: i32, j: i32) {
        if let Some(mut velocoity) = grid.get_mut(i, j) {
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
        if let Some(&src_velocity) = velocities.get(src_i, src_j) {
            if let Some(mut dest_velocity) = velocities.get_mut(dest_i, dest_j) {
                *dest_velocity = src_velocity;
            }
        }
    }

    fn is_solid_boundary(first_cell_type: &CellType, second_cell_type: &CellType) -> bool {
        (*first_cell_type == CellType::Solid || *second_cell_type == CellType::Solid)
            && *first_cell_type != *second_cell_type
    }

    fn set_boundary_velocity(&mut self, i: i32, j: i32) {
        let cell_type = self.cell_types.get(i, j).unwrap_or(&CellType::Empty);
        let cell_type_west = self.cell_types.get(i - 1, j).unwrap_or(&CellType::Empty);
        let cell_type_south = self.cell_types.get(i, j - 1).unwrap_or(&CellType::Empty);
        let cell_type_south_west = self
            .cell_types
            .get(i - 1, j - 1)
            .unwrap_or(&CellType::Empty);

        if Self::is_solid_boundary(cell_type, cell_type_west) {
            Self::set_velocity_component_to_zero(&mut self.horizontal_velocities, i, j);

            if !Self::is_solid_boundary(cell_type_west, cell_type_south_west) {
                Self::copy_velocity_component(&mut self.vertical_velocities, i, j, i - 1, j);
            }
        }

        if Self::is_solid_boundary(cell_type, cell_type_south) {
            Self::set_velocity_component_to_zero(&mut self.vertical_velocities, i, j);

            if !Self::is_solid_boundary(cell_type_south, cell_type_south_west) {
                Self::copy_velocity_component(&mut self.horizontal_velocities, i, j, i, j - 1);
            }
        }
    }

    pub fn set_boundary_velocities(&mut self) {
        let cols = self.cols as i32;
        let rows = self.rows as i32;

        for i in 0..cols + 1 {
            for j in 0..rows + 1 {
                self.set_boundary_velocity(i, j);
            }
        }
    }

    fn get_weighted_velocity_component(
        i: i32,
        j: i32,
        velocity_components: &Grid<f32>,
        weight: f32,
    ) -> Option<f32> {
        let magnitude = velocity_components.get(i, j)?;
        Some(magnitude * weight)
    }

    fn interpolate_horizontal_velocity(&self, point: Vec2) -> f32 {
        let shifted_point = point - Vec2::new(0., self.spacing * 0.5);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        Self::get_weighted_velocity_component(i, j, &self.horizontal_velocities, weights[0])
            .unwrap_or(0.)
            + Self::get_weighted_velocity_component(
                i + 1,
                j,
                &self.horizontal_velocities,
                weights[1],
            )
            .unwrap_or(0.)
            + Self::get_weighted_velocity_component(
                i + 1,
                j + 1,
                &self.horizontal_velocities,
                weights[2],
            )
            .unwrap_or(0.)
            + Self::get_weighted_velocity_component(
                i,
                j + 1,
                &self.horizontal_velocities,
                weights[3],
            )
            .unwrap_or(0.)
    }

    fn interpolate_vertical_velocity(&self, point: Vec2) -> f32 {
        let shifted_point = point - Vec2::new(self.spacing * 0.5, 0.);
        let weights = self.corner_weights(shifted_point);
        let (i, j) = self.floor(shifted_point);

        Self::get_weighted_velocity_component(i, j, &self.vertical_velocities, weights[0])
            .unwrap_or(0.)
            + Self::get_weighted_velocity_component(i + 1, j, &self.vertical_velocities, weights[1])
                .unwrap_or(0.)
            + Self::get_weighted_velocity_component(
                i + 1,
                j + 1,
                &self.vertical_velocities,
                weights[2],
            )
            .unwrap_or(0.)
            + Self::get_weighted_velocity_component(i, j + 1, &self.vertical_velocities, weights[3])
                .unwrap_or(0.)
    }

    fn interpolate_velocity(&self, point: Vec2) -> Option<Vec2> {
        let (i, j) = self.floor(point);
        if i < 0 || i >= self.cols as i32 || j < 0 || j >= self.rows as i32 {
            return None;
        }

        Some(Vec2::new(
            self.interpolate_horizontal_velocity(point),
            self.interpolate_vertical_velocity(point),
        ))
    }

    pub fn transfer_velocities(&mut self, flip_ratio: Option<f32>) {
        if flip_ratio.is_none() {
            self.prev_horizontal_velocities = self.horizontal_velocities.clone();
            self.prev_vertical_velocities = self.vertical_velocities.clone();

            self.horizontal_velocities.fill(0.);
            self.vertical_velocities.fill(0.);

            self.sum_vertical_weights.fill(0.);
            self.sum_horizontal_weights.fill(0.);

            for (mut cell_type, s) in self.cell_types.iter_mut().zip(self.s.iter()) {
                *cell_type = if *s == 0. {
                    CellType::Solid
                } else {
                    CellType::Empty
                }
            }

            for i in 0..self.particle_positions.len() {
                let point = self.particle_positions[i] - self.offset;
                let velocity = self.particle_velocities[i];
                self.mark_occupied_cells_as_fluid(point);
                self.splat_velocities(velocity, point);
            }

            self.normalize_velocities();
            self.set_boundary_velocities();
        } else {
            for i in 0..self.particle_positions.len() {
                let point = self.particle_positions[i] - self.offset;
                if let Some(velocity) = self.interpolate_velocity(point) {
                    self.particle_velocities[i] = velocity;
                }
            }
        }
    }

    pub fn update_particle_density(&mut self) {
        self.densities.fill(0.);

        for i in 0..self.particle_positions.len() {
            let point = self.particle_positions[i] - self.offset;
            self.splat_density(point);
        }
    }

    fn contribute_to_solid_cell_count(&self, i: i32, j: i32) -> f32 {
        match self.cell_types.get(i, j) {
            None => 0.,
            Some(cell_type) => match cell_type {
                CellType::Solid => 0.,
                _ => 1.,
            },
        }
    }

    fn non_solid_neighbours_count(&self, i: i32, j: i32) -> f32 {
        self.contribute_to_solid_cell_count(i + 1, j)
            + self.contribute_to_solid_cell_count(i - 1, j)
            + self.contribute_to_solid_cell_count(i, j + 1)
            + self.contribute_to_solid_cell_count(i, j - 1)
    }

    fn divergence(&self, i: i32, j: i32) -> f32 {
        self.horizontal_velocities.get(i + 1, j).unwrap_or(&0.)
            - self.horizontal_velocities.get(i, j).unwrap_or(&0.)
            + self.vertical_velocities.get(i, j + 1).unwrap_or(&0.)
            - self.vertical_velocities.get(i, j).unwrap_or(&0.)
    }

    fn density(&self, i: i32, j: i32) -> f32 {
        (self.densities.get(i, j).unwrap_or(&0.)
            + self.densities.get(i + 1, j).unwrap_or(&0.)
            + self.densities.get(i + 1, j + 1).unwrap_or(&0.)
            + self.densities.get(i, j + 1).unwrap_or(&0.))
            * 0.25
    }

    pub fn solve_incompressibility(
        &mut self,
        iterations: usize,
        over_relaxation: f32,
        stiffness_coefficient: f32,
        average_density: f32,
    ) {
        let cols = self.cell_types.cols();

        for _ in 0..iterations {
            for (i, cell_type) in self.cell_types.iter().enumerate() {
                if *cell_type != CellType::Fluid {
                    continue;
                }

                let (i, j) = ((i % cols) as i32, (i / cols) as i32);
                let divergence = over_relaxation * self.divergence(i, j)
                    - stiffness_coefficient * (self.density(i, j) - average_density);

                let non_solid_neighbours_count = self.non_solid_neighbours_count(i, j);

                let s1 = self.contribute_to_solid_cell_count(i - 1, j);
                if let Some(velocity) = self.horizontal_velocities.get_mut(i, j) {
                    *velocity += divergence * s1 / non_solid_neighbours_count;
                }

                let s2 = self.contribute_to_solid_cell_count(i + 1, j);
                if let Some(velocity) = self.horizontal_velocities.get_mut(i + 1, j) {
                    *velocity -= divergence * s2 / non_solid_neighbours_count;
                }

                let s3 = self.contribute_to_solid_cell_count(i, j - 1);
                if let Some(mut velocity) = self.vertical_velocities.get_mut(i, j) {
                    *velocity += divergence * s3 / non_solid_neighbours_count;
                }

                let s4 = self.contribute_to_solid_cell_count(i, j + 1);
                if let Some(mut velocity) = self.vertical_velocities.get_mut(i, j + 1) {
                    *velocity -= divergence * s4 / non_solid_neighbours_count;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_boundary_velocities() {
        // *****
        // *   *
        // * * *
        // *   *
        // *****

        let mut simulator = LiquidSimulator::new(vec![], 1., 5, 5, 10.).with_solid_border_cells();
        simulator.set_cell_to_solid(2, 2);

        // Marks cell types
        simulator.transfer_velocities(None);

        simulator.horizontal_velocities.fill(1.);
        simulator.vertical_velocities.fill(2.);

        *simulator.horizontal_velocities.get_mut(2, 1).unwrap() = 3.;
        *simulator.vertical_velocities.get_mut(2, 1).unwrap() = 4.;

        let i = 2;
        let j = 0;

        simulator.set_boundary_velocities();

        println!(
            "horizontal_velocity {:?}",
            simulator.horizontal_velocities.get(i, j).unwrap()
        );
        println!(
            "vertical_velocity {:?}",
            simulator.vertical_velocities.get(i, j).unwrap()
        );
    }
}
