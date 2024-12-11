use crate::pic_flip::components::{FluidSimulator, Velocity};
use crate::pic_flip::resources::Gravity;
use crate::pic_flip::staggered_grid::{CellType, StaggeredGrid};
use bevy::color::palettes::basic::{AQUA, BLACK, MAROON, SILVER};
use bevy::prelude::*;

pub fn spawn_fluid_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cols = 10;
    let rows = 8;
    let cell_spacing = 50.;

    let width = cols as f32 * cell_spacing;
    let height = rows as f32 * cell_spacing;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            FluidSimulator(StaggeredGrid::new(
                cols,
                rows,
                cell_spacing,
                Vec2::new(-width / 2., -height / 2.),
            )),
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

pub fn debug_simulation(sim_query: Query<(&FluidSimulator, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (sim, global_transform) in &sim_query {
        let cell_spacing = sim.0.cell_spacing;
        let half_cell_spacing = cell_spacing / 2.;

        let offset = global_transform.translation().xy()
            - Vec2::new(
                sim.0.cols() as f32 * half_cell_spacing,
                sim.0.rows() as f32 * half_cell_spacing,
            );

        for i in 0..=sim.0.cols() as i32 {
            for j in 0..=sim.0.rows() as i32 {

                // Cell type
                if let Some(cell_type) = sim.0.cell_type(i, j) {
                    let (color, z) = match cell_type {
                        CellType::EMPTY => (SILVER, 4.),
                        CellType::FLUID => (AQUA, 6.),
                        CellType::SOLID => (MAROON, 5.),
                    };

                    gizmos.rect(
                        Isometry3d::from_xyz(
                            i as f32 * cell_spacing + offset.x + half_cell_spacing,
                            j as f32 * cell_spacing + offset.y + half_cell_spacing,
                            z,
                        ),
                        Vec2::splat(cell_spacing),
                        color,
                    );
                }
            }
        }
    }
}
