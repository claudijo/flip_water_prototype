use bevy::prelude::*;
use crate::pic_flip::staggered_grid::StaggeredGrid;

#[derive(Component)]
pub struct FluidSimulator(pub StaggeredGrid);
