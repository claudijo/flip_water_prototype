use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
#[require(Velocity, Transform)]
pub struct SizedParticle(pub f32);

impl Default for SizedParticle {
    fn default() -> Self {
        Self(1.)
    }
}

#[derive(Default, Copy, Clone)]
pub struct Cell {
    pub flow_velocity: Option<Vec2>,
    pub s: f32, // 0 for solid cells, 1 for fluid cells. TODO: Check if we should change type for this field.
}

#[derive(Component, Default)]
pub struct StaggeredGrid {
    pub cells: Vec<Vec<Cell>>,
    pub cell_size: f32,
}

impl StaggeredGrid {
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            cells: vec![vec![Cell::default(); rows as usize]; cols as usize],
            ..default()
        }
    }

    pub fn with_cell_size(mut self, value: f32) -> Self {
        self.cell_size = value;
        self
    }
}

#[derive(Component, Default)]
#[require(Transform, StaggeredGrid)]
pub struct LiquidContainer {
    pub width: f32,
    pub height: f32,
}
