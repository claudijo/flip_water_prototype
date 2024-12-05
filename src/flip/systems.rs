use crate::flip::components::{LiquidContainer, SizedParticle, StaggeredGrid, Velocity};
use crate::flip::resources::Gravity;
use bevy::prelude::*;

pub fn spawn_liquid_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            LiquidContainer {
                width: 200.,
                height: 100.,
            },
            StaggeredGrid::new(10, 5).with_cell_size(20.),
            Mesh2d(meshes.add(Rectangle::new(200., 100.))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        ))
        .with_children(|parent| {
            let rows = 3;
            let cols = 5;
            let particle_size = 4.;

            for i in 0..rows * cols {
                let col = i % cols;
                let row = i / cols;

                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(particle_size, particle_size))),
                    MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
                    Transform::from_xyz(
                        (col - cols / 2) as f32 * particle_size * 2.,
                        (row - rows / 2) as f32 * particle_size * 2.,
                        1.,
                    ),
                    SizedParticle(particle_size),
                ));
            }
        });
}

pub fn integrate_particles(
    gravity: Res<Gravity>,
    mut particle_query: Query<(&mut Velocity, &mut Transform), With<SizedParticle>>,
    time: Res<Time>,
) {
    for (mut velocity, mut transform) in &mut particle_query {
        velocity.0 += gravity.0 * time.delta_secs();
        let delta_translation = velocity.0 * time.delta_secs();
        transform.translation.x += delta_translation.x;
        transform.translation.y += delta_translation.y;

        // TODO: Push particles out of obstacles
    }
}

pub fn transfer_particle_velocity_to_grid(
    mut grid_query: Query<(&mut StaggeredGrid, &LiquidContainer, &Children)>,
    particle_query: Query<(&Transform, &Velocity), With<SizedParticle>>,
) {
    for (mut grid, container, children) in &mut grid_query {
        grid.clear_cells();

        let offset = Vec3::new(container.width / 2., container.height / 2., 0.);

        for &child in children {
            let Ok((transform, velocity)) = particle_query.get(child) else {
                continue;
            };

            let adjusted_transform = transform.translation + offset;

            let col = (adjusted_transform.x / grid.cell_size) as usize;
            let row = (adjusted_transform.y / grid.cell_size) as usize;

            let x = (adjusted_transform.x / grid.cell_size).floor();
            let y = (adjusted_transform.y / grid.cell_size).floor();

            let dx = adjusted_transform.x % grid.cell_size;
            let dy = adjusted_transform.y % grid.cell_size;

            let half_cell_size = grid.cell_size / 2.;

            // Transfer x component of particle velocity to grid
            let (top, bottom) = if dy < half_cell_size {
                (y + half_cell_size - dy, y - half_cell_size + dy)
            } else {
                (y + grid.cell_size - dy, y - dy)
            };

            let left = x - dx;
            let right = x + grid.cell_size - dx;

            // println!("{:?}, Cell at col {:?} row {:?} {:?}", a, cell_col, cell_row, grid.cells[cell_col][cell_row])
        }
    }
}

pub fn make_grid_copy() {}

pub fn make_grid_velocities_incompressible() {}

pub fn transfer_grid_velocity_to_particles() {}

pub fn add_velocity_changes_to_particles() {}
