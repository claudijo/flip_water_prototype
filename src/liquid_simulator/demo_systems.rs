use crate::liquid_simulator::components::{LiquidParticle, LiquidSimulator};
use bevy::prelude::*;

const COLS: usize = 50;
const ROWS: usize = 40;
const CELL_SPACING: f32 = 10.;
const PARTICLE_COUNT: usize = 1600;
const PARTICLE_PER_ROW: usize = 40;
const PARTICLE_RADIUS: f32 = 2.;
const PARTICLE_SPACING: f32 = 3.;

pub fn spawn_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let width = COLS as f32 * CELL_SPACING;
    let height = ROWS as f32 * CELL_SPACING;

    let particle_positions = (0..PARTICLE_COUNT)
        .map(|i| {
            let x = (i % PARTICLE_PER_ROW) as f32 * PARTICLE_SPACING
                - PARTICLE_PER_ROW as f32 * PARTICLE_SPACING / 2.;

            let y = (i / PARTICLE_PER_ROW) as f32 * PARTICLE_SPACING
                - (PARTICLE_COUNT / PARTICLE_PER_ROW) as f32 * PARTICLE_SPACING / 2.;

            Vec2::new(x, y)
        })
        .collect::<Vec<_>>();

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
            LiquidSimulator::new(
                particle_positions,
                PARTICLE_RADIUS,
                Vec2::new(-width / 2., -height / 2.),
                COLS,
                ROWS,
                CELL_SPACING,
            )
            .with_solid_border_cells(),
        ))
        .with_children(|parent| {
            for _ in 0..PARTICLE_COUNT {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(PARTICLE_RADIUS)))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
                    LiquidParticle,
                ));
            }
        });
}

pub fn position_liquid_particles(
    simulator_query: Query<(&LiquidSimulator, &Children)>,
    mut particle_query: Query<&mut Transform, With<LiquidParticle>>,
) {
    for (simulator, children) in &simulator_query {
        for (i, child) in children.iter().enumerate() {
            let Ok(mut transform) = particle_query.get_mut(*child) else {
                continue;
            };

            let Some(position) = simulator.particle_positions.get(i) else {
                continue;
            };

            transform.translation = position.extend(1.);
        }
    }
}
