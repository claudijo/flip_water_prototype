use crate::flop::components::{CellType, StaggeredGrid, Velocity};
use crate::flop::resources::Gravity;
use bevy::color::palettes::basic::{BLUE, GREEN, RED, YELLOW};
use bevy::prelude::*;

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
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
        ))
        .with_children(|parent| {
            let particle_count = 1;
            let particle_per_row = 5;
            let particle_size = 4.;
            let particle_spacing = particle_size * 5.;

            for i in 0..particle_count {
                let x = (i % particle_per_row) as f32 * particle_spacing
                    - particle_per_row as f32 * particle_spacing / 2.;
                let y = (i / particle_per_row) as f32 * particle_spacing;

                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(particle_size, particle_size))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
                    Transform::from_xyz(x, y, 1.),
                    Velocity(Vec2::new(50., 0.)),
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

pub fn simulate_fluid(
    mut grid_query: Query<(&mut StaggeredGrid, &GlobalTransform, &Children)>,
    mut particles_qeury: Query<(&mut Velocity, &mut Transform)>,
) {
    for (mut grid, global_transform, children) in &mut grid_query {
        let offset = Vec2::new(
            grid.cols as f32 * grid.cell_size / 2.,
            grid.rows as f32 * grid.cell_size / 2.,
        );

        grid.clear_cells();

        for child in children {
            if let Ok((mut velocity, mut transform)) = particles_qeury.get_mut(*child) {
                let point = transform.translation.xy() + offset;
                grid.transfer_velocities(point, velocity.0);
                println!("Velocity length {:?}", velocity.0.length());
            }
        }



        // grid.even_out_flow();

        // grid.solve_incompressibility(100, 1.9);

        // for child in children {
        //     if let Ok((mut velocity, transform)) = particles_qeury.get_mut(*child) {
        //         let point = transform.translation.xy() + offset;
        //         if let Some(weighted_velocity) = grid.weighted_velocity_at_point(point) {
        //             velocity.0 = weighted_velocity;
        //         }
        //     }
        // }

        println!("Total velocity in system {:?}", grid.total_velocity_in_system().length());

    }
}

pub fn debug_cells(grid_query: Query<(&StaggeredGrid, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (grid, global_transform) in &grid_query {
        let offset = global_transform.translation().xy()
            - Vec2::new(
                grid.cols as f32 * grid.cell_size / 2.,
                grid.rows as f32 * grid.cell_size / 2.,
            );

        for row in 0..grid.rows {
            for col in 0..grid.cols {
                let Some(cell) = grid.cell_at(col as i32, row as i32) else {
                    continue;
                };

                // Cell type

                let (color, z) = match cell.cell_type {
                    CellType::Air =>  (YELLOW, 4.),
                    CellType::Water =>  (BLUE, 6.),
                    CellType::Solid =>  (RED, 5.),
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

                // Flow velocity

                let flow_scale = 2.;

                if let Some(flow) = cell.horizontal_velocity {
                    let x = col as f32 * grid.cell_size + offset.x;
                    let y = row as f32 * grid.cell_size + offset.y + grid.cell_size / 2.;
                    let start = Vec2::new(x, y);
                    gizmos.arrow_2d(start, start + Vec2::X * flow * flow_scale, GREEN);
                }

                if let Some(flow) = cell.vertical_velocity {
                    let x = col as f32 * grid.cell_size + offset.x + grid.cell_size / 2.;
                    let y = row as f32 * grid.cell_size + offset.y;
                    let start = Vec2::new(x, y);
                    gizmos.arrow_2d(start, start + Vec2::Y * flow * flow_scale, GREEN);
                }
            }
        }
    }
}
