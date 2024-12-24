use crate::flip_fluid::components::{FlipFluid, LiquidParticle};
use bevy::prelude::*;

pub fn spawn_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let width = 500.;
    let height = 400.;

    let density = 1000.;
    let sim_height = 3.0;
    let c_scale = height / sim_height;
    let sim_width = width / c_scale;
    let res = 100.;
    let tank_height = 1.0 * sim_height;
    let tank_width = 1.0 * sim_width;
    let h = tank_height / res;
    let rel_water_height = 0.8;
    let rel_water_width = 0.6;

    // Compute number of particles
    let r = 0.3 * h; // particle radius w.r.t. cell size
    let dx = 2.0 * r;
    let dy = 3_f32.sqrt() / 2.0 * dx;

    let num_x = ((rel_water_width * tank_width - 2.0 * h - 2.0 * r) / dx).floor() as usize;
    let num_y = ((rel_water_height * tank_height - 2.0 * h - 2.0 * r) / dy).floor() as usize;
    let max_particles = num_x * num_y;

    println!("num_x {:?} num_y {:?}", num_x, num_y);

    let point_size = 2.0 * r / sim_width * width;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
            // FlipFluid::new(density, tank_width, tank_height, h, r., max_particles)
            FlipFluid::new(density, tank_width, tank_height, h, 4., max_particles)
                // .with_particles(num_x, num_y)
                .with_particles(10, 10)
                .with_solid_border(),
        ))
        .with_children(|parent| {
            // for _ in 0..(num_x * num_y) {
            for _ in 0..100 {
                parent.spawn((
                    // Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(point_size)))),
                    Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(4.)))),
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
    for (fluid, children) in &fluid_query {
        for (i, child) in children.iter().enumerate() {
            let Ok(mut transform) = particle_query.get_mut(*child) else {
                continue;
            };
            transform.translation = fluid.position(i).extend(1.);
        }
    }
}

pub fn simulate_liquid(
    mut fluid_query: Query<&mut FlipFluid>,
    time: Res<Time>,
) {
    for mut fluid in &mut fluid_query {
        fluid.simulate(time.delta_secs(), -10., 0.9, 50, 2., true);
    }
}
