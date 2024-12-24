use bevy::prelude::*;

#[derive(Component)]
pub struct LiquidParticle;

#[derive(Component)]
pub struct FlipFluid {
    density: f32,

    // Number of cols and rows in staggered grid
    f_num_x: usize,
    f_num_y: usize,

    // Spacing
    h: f32,
    f_inv_spacing: f32,
    f_num_cells: usize,

    // Horizontal velocity
    u: Vec<f32>,

    // Vertical velocity
    v: Vec<f32>,

    // Sum of weights(?)
    du: Vec<f32>,
    dv: Vec<f32>,

    pre_u: Vec<f32>,
    pre_v: Vec<f32>,

    // Density
    p: Vec<f32>,

    s: Vec<f32>,

    cell_type: Vec<i32>,
    cell_color: Vec<f32>,

    max_particles: usize,
    particle_pos: Vec<f32>,
    particle_color: Vec<f32>,
    particle_vel: Vec<f32>,
    particle_density: Vec<f32>,
    particle_rest_density: f32,
    particle_radius: f32,
    p_inv_spacing: f32,
    p_num_x: usize,
    p_num_y: usize,
    p_num_cells: usize,
    num_cell_particles: Vec<usize>,
    first_cell_particle: Vec<usize>,
    cell_particle_ids: Vec<i32>,
    num_particles: usize,
}

impl FlipFluid {
    pub fn new(
        density: f32,
        width: f32,
        height: f32,
        spacing: f32,
        particle_radius: f32,
        max_particles: usize,
    ) -> Self {
        let f_num_x = (width / spacing).floor() as usize + 1;
        let f_num_y = (height / spacing).floor() as usize + 1;

        println!("f_num_x {:?} f_num_y {:?}", f_num_x, f_num_y);

        let h = (width / f_num_x as f32).max(height / f_num_y as f32);
        let f_num_cells = f_num_x * f_num_y;
        let p_inv_spacing = 1. / (2.2 * particle_radius);
        let p_num_x = (width * p_inv_spacing).floor() as usize + 1;
        let p_num_y = (height * p_inv_spacing).floor() as usize + 1;
        let p_num_cells = p_num_x * p_num_y;

        Self {
            density,
            f_num_x,
            f_num_y,
            h,
            f_inv_spacing: 1. / h,
            f_num_cells,
            u: vec![f32::default(); f_num_cells],
            v: vec![f32::default(); f_num_cells],
            du: vec![f32::default(); f_num_cells],
            dv: vec![f32::default(); f_num_cells],
            pre_u: vec![f32::default(); f_num_cells],
            pre_v: vec![f32::default(); f_num_cells],
            p: vec![f32::default(); f_num_cells],
            s: vec![1.; f_num_cells], // 1 = fluid (liquid or empty), 0 = solid
            cell_type: vec![i32::default(); f_num_cells],
            cell_color: vec![f32::default(); f_num_cells * 3],
            max_particles,

            particle_pos: vec![f32::default(); max_particles * 2],
            particle_color: vec![1.; max_particles * 3],
            particle_vel: vec![f32::default(); max_particles * 2],
            particle_density: vec![f32::default(); f_num_cells],
            particle_rest_density: 0.,
            particle_radius,
            p_inv_spacing,
            p_num_x,
            p_num_y,
            p_num_cells,
            num_cell_particles: vec![usize::default(); p_num_cells],
            first_cell_particle: vec![usize::default(); p_num_cells + 1],
            cell_particle_ids: vec![i32::default(); max_particles],
            num_particles: 0,
        }
    }

    pub fn with_particles(mut self, num_x: usize, num_y: usize) -> Self {
        self.num_particles = num_y * num_x;

        let h = self.h;
        let r = self.particle_radius;
        let dx = 2. * r;
        let dy = 3_f32.sqrt() / 2.0 * dx;

        let mut p = 0;
        for i in 0..num_x {
            for j in 0..num_y {
                self.particle_pos[p] = h + r + dx * i as f32 + if j % 2 == 0 { 0. } else { r };
                p += 1;
                self.particle_pos[p] = h + r + dy * j as f32;
                p += 1;
            }
        }

        self
    }

    pub fn with_solid_border(mut self) -> Self {
        let n = self.f_num_y;

        for i in 0..self.f_num_x {
            for j in 0..self.f_num_y {
                if i == 0 || i == self.f_num_x - 1 || j == 0 {
                    self.s[i * n + j] = 0.;
                }
            }
        }

        self
    }

    pub fn simulate(
        &mut self,
        dt: f32,
        gravity: f32,
        flip_ratio: f32,
        num_pressure_iters: usize,
        over_relaxation: f32,
        compensate_drift: bool,
    ) {
        let num_sub_steps = 1;
        let std = dt / num_sub_steps as f32;

        for _ in 0..num_sub_steps {
            self.integrate_particles(std, gravity);
        }
    }

    pub fn position(&self, i: usize) -> Vec2 {
        Vec2::new(
            self.particle_pos[2 * i],
            self.particle_pos[2 * i + 1],
        )
    }

    fn integrate_particles(&mut self, dt: f32, gravity: f32) {
        for i in 0..self.num_particles {
            self.particle_vel[2 * i + 1] += dt * gravity;
            self.particle_pos[2 * i] += self.particle_vel[2 * i] * dt;
            self.particle_pos[2 * i + 1] += self.particle_vel[2 * i + 1] * dt;
        }
    }
}
