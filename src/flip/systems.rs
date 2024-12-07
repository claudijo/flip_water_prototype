use crate::flip::components::{Cell, LiquidContainer, SizedParticle, StaggeredGrid, Velocity};
use crate::flip::resources::Gravity;
use bevy::prelude::*;
use bevy::utils::HashSet;

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
            StaggeredGrid::new(20, 10).with_cell_size(10.),
            Mesh2d(meshes.add(Rectangle::new(200., 100.))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        ))
        .with_children(|parent| {
            let rows = 9;
            let cols = 21;
            let particle_size = 4.;

            for i in 0..rows * cols {
                let col = i % cols;
                let row = i / cols;

                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(particle_size, particle_size))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
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

pub fn simulate_fluid(
    mut grid_query: Query<(&mut StaggeredGrid, &LiquidContainer, &Children)>,
    mut particle_query: Query<(&Transform, &mut Velocity), With<SizedParticle>>,
) {
    for (mut grid, container, children) in &mut grid_query {
        let offset = Vec3::new(container.width / 2., container.height / 2., 0.);

        grid.clear_cells();

        let mut populated_cols_rows = HashSet::new();

        // Transfer particle velocity to grid

        for &child in children {
            let Ok((transform, velocity)) = particle_query.get(child) else {
                continue;
            };
            let point = (transform.translation + offset).xy();

            if let Some(col_row) =
                grid.col_row_from_cell_position(grid.cell_position_from_point(point))
            {
                populated_cols_rows.insert(col_row);
            }

            grid.accumulate_flow_from_particle(point, velocity.0);
        }

        for &(col, row) in populated_cols_rows.iter() {
            grid.even_out_flow_for_cell(col, row);
        }

        // Make water particles incompressible

        for &(col, row) in populated_cols_rows.iter() {
            grid.project_flow_for_cell(col, row, 40, 1.9);
        }

        // Transfer grid velocity to particles

        for &child in children {
            let Ok((transform, mut velocity)) = particle_query.get_mut(child) else {
                continue;
            };
            let point = (transform.translation + offset).xy();

            velocity.0.x = grid.weighted_horizontal_velocity_at_point(point);
            velocity.0.y = grid.weighted_vertical_velocity_at_point(point);
        }
    }
}

