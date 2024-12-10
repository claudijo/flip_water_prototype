use bevy::prelude::*;
use crate::pic_flip::components::FluidSimulator;
use crate::pic_flip::staggered_grid::StaggeredGrid;

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

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        FluidSimulator(StaggeredGrid::new(cols, rows, cell_spacing, Vec2::new(-width / 2., -height / 2.))),
        Transform::from_xyz(0., 0., -1.),
        Visibility::default(),
        )).with_children(|parent| {

    });
}