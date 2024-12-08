use crate::flop::components::{StaggeredGrid, Velocity};
use bevy::color::palettes::basic::{BLUE, YELLOW};
use bevy::prelude::*;
use crate::flop::resources::Gravity;

pub fn spawn_liquid_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cols = 10;
    let rows = 8;
    let cell_size = 50.;

    let width = cols as f32 * cell_size;
    let height = rows as f32 * cell_size;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            StaggeredGrid::new(cols, rows, cell_size).with_border_cells(),
        ))
        .with_children(|parent| {
            let particle_count = 120;
            let particle_per_row = 15;
            let particle_size = 4.;
            let particle_spacing = particle_size * 4.;

            for i in 0..particle_count {
                let x = (i % particle_per_row) as f32 * particle_spacing
                    - particle_per_row as f32 * particle_spacing / 2.;
                let y = (i / particle_per_row) as f32 * particle_spacing;

                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(particle_size, particle_size))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
                    Transform::from_xyz(x, y, 5.),
                    Velocity::default(),
                ));
            }
        });
}

pub fn integrate_particles(
    mut particle_query: Query<(&mut Velocity, &mut Transform)>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    for (mut velocity, mut transform) in &mut particle_query {
        velocity.0 += gravity.0 * time.delta_secs();
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}

pub fn debug_cells(mut grid_query: Query<(&StaggeredGrid, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (grid, global_transform) in &grid_query {
        let offset = global_transform.translation().xy()
            - Vec2::new(
                grid.cols as f32 * grid.cell_size / 2.,
                grid.rows as f32 * grid.cell_size / 2.,
            );

        for row in 0..grid.rows {
            for col in 0..grid.cols {
                let (color, z) = if grid.cell_at(col, row).divergence_scale == 0. {
                    (YELLOW, 5.)
                } else {
                    (BLUE, 4.)
                };

                gizmos.rect(
                    Isometry3d::from_xyz(
                        col as f32 * grid.cell_size + offset.x + grid.cell_size / 2.,
                        row as f32 * grid.cell_size + offset.y + grid.cell_size / 2.,
                        z,
                    ),
                    Vec2::splat(grid.cell_size),
                    color,
                );
            }
        }
    }
}

