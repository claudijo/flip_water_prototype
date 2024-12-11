use crate::pic_flip::staggered_grid::StaggeredGrid;
use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct FluidSimulator(pub StaggeredGrid);
