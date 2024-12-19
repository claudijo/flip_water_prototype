use crate::liquid_simulator::spatial_hash::SpatialHash;
use bevy::prelude::*;

#[derive(Component)]
pub struct LiquidParticle;

#[derive(Component)]
pub struct LiquidSimulator {
    pub offset: Vec2,
    pub particle_positions: Vec<Vec2>,
    pub particle_velocities: Vec<Vec2>,
    pub particle_radius: f32,
    pub spacial_hash: SpatialHash,
}

impl LiquidSimulator {
    pub fn new(
        width: f32,
        height: f32,
        particle_positions: Vec<Vec2>,
        particle_radius: f32,
        offset: Vec2,
    ) -> Self {
        let particle_count = particle_positions.len();

        Self {
            spacial_hash: SpatialHash::from_sizes(width, height, particle_radius)
                .with_offset(offset),
            offset,
            particle_positions,
            particle_velocities: vec![Vec2::default(); particle_count],
            particle_radius,
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

            let mut collisions = vec![];

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

                    // self.separate_particles(a, b, collision_normal, collision_depth);
                    collisions.push((a, b, collision_normal, collision_depth));
                }
            }

            for collision in collisions {
                let (a, b, collision_normal, collision_depth) = collision;
                self.separate_particles(a, b, collision_normal, collision_depth);
            }
        }
    }
}
