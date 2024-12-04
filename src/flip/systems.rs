use crate::flip::components::{LiquidContainer, SizedParticle, StaggeredGrid};
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
