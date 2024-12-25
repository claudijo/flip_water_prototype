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
    cell_particle_ids: Vec<usize>,
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
            cell_particle_ids: vec![usize::default(); max_particles],
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
        num_particle_iters: usize,
        over_relaxation: f32,
        compensate_drift: bool,
        separate_particles: bool,
    ) {
        let num_sub_steps = 1;
        let std = dt / num_sub_steps as f32;

        for _ in 0..num_sub_steps {
            self.integrate_particles(std, gravity);
            if separate_particles {
                self.push_particles_apart(num_particle_iters);
            }
            self.handle_particle_collision();
        }
    }

    pub fn position(&self, i: usize) -> Vec2 {
        Vec2::new(self.particle_pos[2 * i], self.particle_pos[2 * i + 1])
    }

    fn integrate_particles(&mut self, dt: f32, gravity: f32) {
        for i in 0..self.num_particles {
            self.particle_vel[2 * i + 1] += dt * gravity;
            self.particle_pos[2 * i] += self.particle_vel[2 * i] * dt;
            self.particle_pos[2 * i + 1] += self.particle_vel[2 * i + 1] * dt;
        }
    }

    fn push_particles_apart(&mut self, num_iters: usize) {
        let color_diffusion_coeff = 0.001;

        // count particles per cell

        self.num_cell_particles.fill(0);

        for i in 0..self.num_particles {
            let x = self.particle_pos[2 * i];
            let y = self.particle_pos[2 * i + 1];

            let xi = ((x * self.p_inv_spacing).floor() as usize).clamp(0, self.p_num_x - 1);
            let yi = ((y * self.p_inv_spacing).floor() as usize).clamp(0, self.p_num_y - 1);
            let cell_nr = xi * self.p_num_y + yi;
            self.num_cell_particles[cell_nr] += 1;
        }

        // partial sums

        let mut first = 0;

        for i in 0..self.p_num_cells {
            first += self.num_cell_particles[i];
            self.first_cell_particle[i] = first;
        }
        self.first_cell_particle[self.p_num_cells] = first; // guard

        // fill particles into cells

        for i in 0..self.num_particles {
            let x = self.particle_pos[2 * i];
            let y = self.particle_pos[2 * i + 1];

            let xi = ((x * self.p_inv_spacing).floor() as usize).clamp(0, self.p_num_x - 1);
            let yi = ((y * self.p_inv_spacing).floor() as usize).clamp(0, self.p_num_y - 1);
            let cell_nr = xi * self.p_num_y + yi;
            self.first_cell_particle[cell_nr] -= 1;
            self.cell_particle_ids[self.first_cell_particle[cell_nr]] = i;
        }

        // push particles apart

        // let min_dist = 2. * self.particle_radius;
        let min_dist = 4. * self.particle_radius;
        let min_dist_2 = min_dist * min_dist;

        for _ in 0..num_iters {
            for i in 0..self.num_particles {
                let px = self.particle_pos[2 * i];
                let py = self.particle_pos[2 * i + 1];

                let pxi = (px * self.p_inv_spacing).floor() as i32;
                let pyi = (py * self.p_inv_spacing).floor() as i32;
                let x0 = (pxi - 1).max(0) as usize;
                let y0 = (pyi - 1).max(0) as usize;
                let x1 = ((pxi + 1) as usize).min(self.p_num_x - 1);
                let y1 = ((pyi + 1) as usize).min(self.p_num_y - 1);

                for xi in x0..=x1 {
                    for yi in y0..=y1 {
                        let cell_nr = xi * self.p_num_y + yi;
                        let first = self.first_cell_particle[cell_nr];
                        let last = self.first_cell_particle[cell_nr + 1];
                        for j in first..last {
                            let id = self.cell_particle_ids[j];
                            if id == i {
                                continue;
                            }

                            let qx = self.particle_pos[2 * id];
                            let qy = self.particle_pos[2 * id + 1];

                            let mut dx = qx - px;
                            let mut dy = qy - py;
                            let d2 = dx * dx + dy * dy;
                            if d2 > min_dist_2 || d2 == 0. {
                                continue;
                            }
                            let d = d2.sqrt();
                            let s = 0.5 * (min_dist - d) / d;
                            dx *= s;
                            dy *= s;
                            self.particle_pos[2 * i] -= dx;
                            self.particle_pos[2 * i + 1] -= dy;
                            self.particle_pos[2 * id] += dx;
                            self.particle_pos[2 * id + 1] += dy;

                            // diffuse colors

                            for k in 0..3 {
                                let color0 = self.particle_color[3 * i + k];
                                let color1 = self.particle_color[3 * id + k];
                                let color = (color0 + color1) * 0.5;
                                self.particle_color[3 * i + k] =
                                    color0 + (color - color0) * color_diffusion_coeff;
                                self.particle_color[3 * id + k] =
                                    color1 + (color - color1) * color_diffusion_coeff;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_particle_collision(&mut self) {
        let h = 1. / self.f_inv_spacing;
        let r = self.particle_radius;
        let min_x = h + r;
        let max_x = (self.f_num_x - 1) as f32 * h - r;
        let min_y = h + r;
        let max_y = (self.f_num_y - 1) as f32 * h - r;


        for i in 0.. self.num_particles {
            let mut x = self.particle_pos[2 * i];
            let mut y = self.particle_pos[2 * i + 1];

            // wall collisions
            if x < min_x {
                x = min_x;
                self.particle_vel[2 * i] = 0.;
            }

            if x > max_x {
                x = max_x;
                self.particle_vel[2 * i] = 0.;
            }

            if y < min_y {
                y = min_y;
                self.particle_vel[2 * i + 1] = 0.;
            }

            if y > max_y {
                y = max_y;
                self.particle_vel[2 * i + 1] = 0.;
            }

            self.particle_pos[2 * i] = x;
            self.particle_pos[2 * i + 1] = y;
        }


    }
}
