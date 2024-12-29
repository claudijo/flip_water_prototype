use std::f32::consts::{FRAC_PI_2, PI};
use crate::flip_fluid::components::{
    AngularVelocity, FlipFluid, LinearVelocity, LiquidParticle, PrevAngularVelocity,
    PrevLinearVelocity, Tank,
};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use std::ops::Neg;
use bevy::color::palettes::basic::{GREEN, RED, YELLOW};
use bevy::transform;

const WIDTH: f32 = 100.;
const HEIGHT: f32 = 200.;

pub fn spawn_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let density = 1000.;
    let num_x = 50;
    let num_y = 50;
    let max_particles = num_x * num_y;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
            FlipFluid::new(density, WIDTH, HEIGHT, 4., 0.5, max_particles)
                .with_particles(num_x, num_y)
                .with_solid_border(),
            Tank,
            LinearVelocity(Vec2::ZERO),
            PrevLinearVelocity(Vec2::ZERO),
            AngularVelocity(0.),
            PrevAngularVelocity(0.),
        ))
        .with_children(|parent| {
            for _ in 0..max_particles {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(2.)))),
                    MeshMaterial2d(materials.add(Color::srgb(1., 1., 1.))),
                    LiquidParticle,
                ));
            }
        });
}

pub fn move_particles(
    fluid_query: Query<(&FlipFluid, &Children)>,
    mut particle_query: Query<&mut Transform>,
) {
    let offset = Vec2::new(WIDTH * -0.5, HEIGHT * -0.5);
    for (fluid, children) in &fluid_query {
        for (i, child) in children.iter().enumerate() {
            if let Ok(mut transform) = particle_query.get_mut(*child) {
                transform.translation = (fluid.position(i) + offset).extend(1.);
            }
        }
    }
}

pub fn simulate_liquid(
    mut fluid_query: Query<(&mut FlipFluid, &Transform, &mut PrevLinearVelocity, &LinearVelocity)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut fluid, transform, mut prev_linear_velocity, linear_velocity) in &mut fluid_query {
        let gravity_angle = (transform.rotation * Vec3::NEG_Y).xy().angle_to(Vec2::NEG_X) ;
        let gravity_vec = Vec2::from_angle(gravity_angle);
        let gravity = gravity_vec * 500.;

        let linear_velocity_delta = linear_velocity.0 - prev_linear_velocity.0;
        let tank_acceleration = linear_velocity_delta / time.delta_secs();
        prev_linear_velocity.0 = linear_velocity.0;

        let total_accel =
            if tank_acceleration.is_finite() {
                gravity + tank_acceleration.neg()
            } else {
                gravity
            };

        fluid.simulate(
            time.delta_secs(),
            total_accel.x,
            total_accel.y,
            0.9,
            100,
            2,
            1.9,
            true,
            true,
        );

    }

}

pub fn update_linear_velocity(
    mut evr_motion: EventReader<MouseMotion>,
    mut physics_query: Query<&mut LinearVelocity>,
    time: Res<Time>,
) {
    for mut linear_velocity in &mut physics_query {
        linear_velocity.0 = Vec2::ZERO;
        for ev in evr_motion.read() {
            linear_velocity.0 = ev.delta * Vec2::new(1., -1.) / time.delta_secs();
        }
    }
}

pub fn update_angular_velocity(mut physics_query: Query<&mut AngularVelocity>, time: Res<Time>) {
    for mut angular_velocity in &mut physics_query {
        angular_velocity.0 = (time.elapsed_secs() * 0.5).sin() * 0.02;
    }
}

pub fn integrate_rotation(
    mut physics_query: Query<(&mut Transform, &AngularVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, angular_velocity) in &mut physics_query {
        transform.rotate_around(Vec3::new(0., 0., 0.), Quat::from_rotation_z(angular_velocity.0));
        // transform.rotation *= Quat::from_rotation_z(angular_velocity.0);
    }
}

pub fn integrate_position(
    mut physics_query: Query<(&mut Transform, &LinearVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, linear_velocity) in &mut physics_query {
        transform.translation += linear_velocity.0.extend(0.) * time.delta_secs();
    }
}
