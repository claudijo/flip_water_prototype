// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod flip_fluid;
mod liquid_simulator;
mod pic_flip;

use crate::flip_fluid::FlipFluidPlugin;
use crate::liquid_simulator::{LiquidSimulationDebugPlugin, LiquidSimulatorPlugin};
use crate::pic_flip::PicFlipPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        // .add_plugins(PicFlipPlugin)
        // .add_plugins(LiquidSimulatorPlugin)
        // .add_plugins(LiquidSimulationDebugPlugin)
        .add_plugins(FlipFluidPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, OrthographicProjection {
        scale: 0.4,
        near: -1000.0,
        far: 1000.0,
        viewport_origin: Vec2::new(0.5, 0.5),
        scaling_mode: ScalingMode::WindowSize,
        area: Rect::new(-1.0, -1.0, 1.0, 1.0),
    }) );
}
