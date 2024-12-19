use crate::liquid_simulator::grid::Grid;
use crate::liquid_simulator::spatial_hash::SpatialHash;
use bevy::prelude::*;

#[derive(Component)]
pub struct LiquidParticle;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum CellType {
    #[default]
    EMPTY,
    FLUID,
    SOLID,
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
    pub sum_horizontal_velocities: Grid<f32>,
    pub vertical_velocities: Grid<f32>,
    pub prev_vertical_velocities: Grid<f32>,
    pub sum_vertical_velocities: Grid<f32>,
    pub cell_types: Grid<CellType>,
    pub s: Grid<f32>, // 0 -> EMPTY or LIQUID , 1 -> Solid
}

impl LiquidSimulator {
    pub fn new(
        particle_positions: Vec<Vec2>,
        particle_radius: f32,
        offset: Vec2,
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
            spacial_hash: SpatialHash::from_sizes(width, height, particle_radius)
                .with_offset(offset),
            offset,
            particle_positions,
            particle_velocities: vec![Vec2::default(); particle_count],
            particle_radius,
            horizontal_velocities: Grid::new(cols + 1, rows),
            prev_horizontal_velocities: Grid::new(cols + 1, rows),
            sum_horizontal_velocities: Grid::new(cols + 1, rows),
            vertical_velocities: Grid::new(cols, rows + 1),
            prev_vertical_velocities: Grid::new(cols, rows + 1),
            sum_vertical_velocities: Grid::new(cols, rows + 1),
            cell_types: Grid::new(cols, rows),
            s: Grid::new(cols, rows).with_default_value(1.),
        }
    }

    pub fn with_solid_border_cells(mut self) -> Self {
        self.set_border_cells_to_solid();
        self
    }

    fn set_border_cells_to_solid(&mut self) {
        for i in 0..self.cols {
            for j in 0..self.rows {
                if i == 0 || i == self.cols - 1 || j == 0 || j == self.rows - 1 {
                    if let Some(mut value) = self.s.get_mut(i as i32, j as i32) {
                        *value = 0.; // Solid
                    }
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

    pub fn transfer_velocities(&mut self, flip_ratio: Option<f32>) {
        if flip_ratio.is_none() {
            self.prev_horizontal_velocities = self.horizontal_velocities.clone();
            self.prev_vertical_velocities = self.vertical_velocities.clone();

            self.horizontal_velocities.fill(0.);
            self.vertical_velocities.fill(0.);

            // TODO: Check that this is `this.du` and `this.dv` in original code
            self.sum_vertical_velocities.fill(0.);
            self.sum_horizontal_velocities.fill(0.);

            for (mut cell_type, s) in self.cell_types.iter_mut().zip(self.s.iter()) {
                *cell_type = if *s == 0. {
                    CellType::SOLID
                } else {
                    CellType::EMPTY
                }
            }

            for point in &self.particle_positions {
                let point = point - self.offset;
                let (i, j) = self.floor(point);
                if let Some(mut cell_type) = self.cell_types.get_mut(i, j) {
                    if *cell_type == CellType::EMPTY {
                        *cell_type = CellType::FLUID;
                    }
                };
            }
        }
    }
}
