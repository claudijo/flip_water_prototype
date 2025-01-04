use crate::flip_fluid::components::{
    AngularVelocity, FlipFluid, LinearVelocity, LiquidParticle, PrevAngularVelocity,
    PrevGlobalTransform, PrevLinearVelocity, Tank,
};
use crate::utils::mechanics::center_of_rotation;
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::input::mouse::MouseMotion;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::ops::Neg;

const WIDTH: f32 = 30.;
const HEIGHT: f32 = 50.;

pub fn spawn_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let density = 1000.;
    let num_x = 30;
    let num_y = 30;
    let max_particles = num_x * num_y;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
            MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.4))),
            Transform::from_xyz(0., 0., -1.),
            Visibility::default(),
            FlipFluid::new(density, WIDTH, HEIGHT, 2., 0.2, max_particles)
                .with_solid_border()
                .with_particles(num_x, num_y),
            Tank,
            LinearVelocity(Vec2::default()),
            PrevLinearVelocity(Vec2::default()),
            PrevGlobalTransform(Affine3A::default()),
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
    mut particle_query: Query<&mut Transform, With<LiquidParticle>>,
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

pub fn color_particles(
    fluid_query: Query<(&FlipFluid, &Children)>,
    particle_query: Query<&MeshMaterial2d<ColorMaterial>, With<LiquidParticle>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    for (fluid, children) in &fluid_query {
        for (i, child) in children.iter().enumerate() {
            if let Ok(color_material) = particle_query.get(*child) {
                if let Some(material) = colors.get_mut(color_material.id()) {
                    material.color = fluid.color(i);
                }
            }
        }
    }
}

pub fn simulate_liquid(
    mut fluid_query: Query<(
        &mut FlipFluid,
        &GlobalTransform,
        &Transform,
        &mut PrevLinearVelocity,
        &LinearVelocity,
        &AngularVelocity,
        &mut PrevAngularVelocity,
        &mut PrevGlobalTransform,
    )>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (
        mut fluid,
        global_transform,
        transform,
        mut prev_linear_velocity,
        linear_velocity,
        angular_velocity,
        mut prev_angular_velocity,
        mut prev_global_transform,
    ) in &mut fluid_query
    {
        let gravity_angle = (transform.rotation * Vec3::NEG_Y)
            .xy()
            .angle_to(Vec2::NEG_X);
        let gravity_vec = Vec2::from_angle(gravity_angle);
        let gravity = gravity_vec * 400.;

        let linear_velocity_delta = linear_velocity.0 - prev_linear_velocity.0;
        let tank_acceleration = linear_velocity_delta / time.delta_secs();
        prev_linear_velocity.0 = linear_velocity.0;

        let angular_velocity_delta = angular_velocity.0 - prev_angular_velocity.0;
        let angular_acceleration = angular_velocity_delta / time.delta_secs();
        prev_angular_velocity.0 = angular_velocity.0;

        let linear_acceleration = if tank_acceleration.is_finite() {
            gravity + tank_acceleration.neg()
        } else {
            gravity
        };

        let point_a = prev_global_transform.0.translation.xy();
        let velocity_a = global_transform.translation().xy() - point_a;
        let point_b = prev_global_transform.0.transform_point3(Vec3::Y * 1.).xy();
        let velocity_b = global_transform.transform_point(Vec3::Y * 1.).xy() - point_b;
        prev_global_transform.0 = global_transform.affine();

        let pole = center_of_rotation(point_a, velocity_a, point_b, velocity_b);
        let tank_offset = Vec2::new(WIDTH * 0.5, HEIGHT * 0.5);
        let rotation_center = tank_offset
            + global_transform
                .affine()
                .inverse()
                .transform_point(pole.extend(0.))
                .xy();

        println!("local rotation center {:.2}", rotation_center);
        gizmos.circle_2d(Isometry2d::from(pole), 4., YELLOW);

        fluid.simulate(
            time.delta_secs(),
            linear_acceleration.x,
            linear_acceleration.y,
            angular_acceleration,
            angular_velocity.0,
            rotation_center.x,
            rotation_center.y,
            0.9,
            100,
            2,
            1.9,
            true,
            true,
            &mut gizmos,
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
        angular_velocity.0 = (time.elapsed_secs() * 0.2).sin() * 8.;
        // angular_velocity.0 += 0.001;
        // angular_velocity.0 = 4.;
    }
}

pub fn integrate_rotation(
    mut physics_query: Query<(&mut Transform, &AngularVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, angular_velocity) in &mut physics_query {
        transform.rotate_around(
            Vec3::new(0., 0., 0.),
            Quat::from_rotation_z(angular_velocity.0 * time.delta_secs()),
        );

        // transform.rotate(Quat::from_rotation_z(
        //     angular_velocity.0 * time.delta_secs(),
        // ));
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
