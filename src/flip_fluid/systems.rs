use crate::flip_fluid::components::{FlipFluid, LiquidParticle};
use bevy::prelude::*;

const WIDTH: f32 = 500.;
const HEIGHT: f32 = 600.;

pub fn spawn_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let sim_height = 3.0;
    // let c_scale = height / sim_height;
    // let sim_width = width / c_scale;
    // let res = 100.;
    // let tank_height = 1.0 * sim_height;
    // let tank_width = 1.0 * sim_width;
    // let h = tank_height / res;
    // let rel_water_height = 0.8;
    // let rel_water_width = 0.6;
    //
    // // Compute number of particles
    // let r = 0.3 * h; // particle radius w.r.t. cell size
    // let dx = 2.0 * r;
    // let dy = 3_f32.sqrt() / 2.0 * dx;
    //
    // let num_x = ((rel_water_width * tank_width - 2.0 * h - 2.0 * r) / dx).floor() as usize;
    // let num_y = ((rel_water_height * tank_height - 2.0 * h - 2.0 * r) / dy).floor() as usize;
    // let max_particles = num_x * num_y;
    //
    // let point_size = 2.0 * r / sim_width * width;

    let density = 1000.;
    let num_x = 50;
    let num_y = 50;
    let max_particles = num_x * num_y;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
            FlipFluid::new(density, WIDTH, HEIGHT, 10., 2., max_particles)
                .with_particles(num_x, num_y)
                .with_solid_border(),
        ))
        .with_children(|parent| {
            for _ in 0..max_particles {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(4.)))),
                    // Mesh2d(meshes.add(Circle::new(2.))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
                    LiquidParticle,
                ));
            }
        });
}

pub fn move_particles(
    fluid_query: Query<(&FlipFluid, &Children)>,
    mut particle_query: Query<&mut Transform>,
) {
    let offset = Vec2::new(WIDTH * -0.5, HEIGHT * -0.5);
    for (fluid, children) in &fluid_query {
        for (i, child) in children.iter().enumerate() {
            if let Ok(mut transform) = particle_query.get_mut(*child) {
                transform.translation = (fluid.position(i) + offset).extend(1.);
            }
        }
    }
}

pub fn simulate_liquid(mut fluid_query: Query<&mut FlipFluid>, time: Res<Time>) {
    for mut fluid in &mut fluid_query {
        fluid.simulate(time.delta_secs(), -500., 0.9, 100, 2, 1.9, true, true);
    }
}
