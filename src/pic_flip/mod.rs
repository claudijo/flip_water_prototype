use crate::pic_flip::resources::Gravity;
use bevy::prelude::*;

mod components;
mod grid;
mod resources;
mod staggered_grid;

pub struct PicFlipPlugin;

impl Plugin for PicFlipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0., -10.)));
    }
}
