use bevy::prelude::*;

#[derive(Component)]
pub struct LiquidParticle;

#[derive(Component)]
pub struct LiquidSimulator {
    pub offset: Vec2,
    pub particle_positions: Vec<Vec2>,
    pub particle_velocities: Vec<Vec2>,
}

impl LiquidSimulator {
    pub fn new(particle_positions: Vec<Vec2>, offset: Vec2) -> Self {
        let particle_count = particle_positions.len();

        Self {
            offset,
            particle_positions,
            particle_velocities: vec![Vec2::default(); particle_count],
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
}
