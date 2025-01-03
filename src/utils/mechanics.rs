use bevy::prelude::*;
use std::ops::Neg;

// https://en.wikipedia.org/wiki/Instant_centre_of_rotation#Pole_of_a_planar_displacement
pub fn pole_of_planar_displacement(
    point_a_prev: Vec2,
    point_a_next: Vec2,
    point_b_prev: Vec2,
    point_b_next: Vec2,
) -> Vec2 {
    let to_a_next = point_a_next - point_a_prev;
    let to_b_next = point_b_next - point_b_prev;

    let slope_a = -to_a_next.x / to_a_next.y;
    let slope_b = -to_b_next.x / to_b_next.y;

    let mid_point_a = 0.5 * (point_a_prev + point_a_next);
    let mid_point_b = 0.5 * (point_b_prev + point_b_next);

    let px = (slope_a * mid_point_a.x - mid_point_a.y - slope_b * mid_point_b.x + mid_point_b.y)
        / (slope_a - slope_b);
    let py = slope_b * (px - mid_point_b.x) + mid_point_b.y;

    Vec2::new(px, py)
}

pub fn center_of_rotation(
    point_a: Vec2,
    velocity_a: Vec2,
    point_b: Vec2,
    velocity_b: Vec2,
) -> Vec2 {
    let slope_a = (velocity_a.x / velocity_a.y).neg();
    let slope_b = (velocity_b.x / velocity_b.y).neg();

    let midpoint_a = point_a + velocity_a * 0.5;
    let midpoint_b = point_b + velocity_b * 0.5;

    let px = (slope_a * midpoint_a.x - midpoint_a.y - slope_b * midpoint_b.x + midpoint_b.y)
        / (slope_a - slope_b);
    let py = slope_b * (px - midpoint_b.x) + midpoint_b.y;

    let center = Vec2::new(px, py);

    if center.is_finite() {
        center
    } else {
        Vec2::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pole_of_planar_displacement_works() {
        let a_prev = Vec2::new(2., 9.);
        let a_next = Vec2::new(1., 4.);
        let b_prev = Vec2::new(6., 1.);
        let b_next = Vec2::new(9., 8.);

        let pole = pole_of_planar_displacement(a_prev, a_next, b_prev, b_next);

        assert!((pole.x - 4.).abs() < 0.0001);
        assert!((pole.y - 6.).abs() < 0.0001);
    }

    #[test]
    fn center_of_rotation_works() {
        let point_a = Vec2::new(2., 9.);
        let velocity_a = Vec2::new(-1., -5.);
        let point_b = Vec2::new(6., 1.);
        let velocity_b = Vec2::new(3., 7.);

        let center = center_of_rotation(point_a, velocity_a, point_b, velocity_b);

        assert!((center.x - 4.).abs() < 0.0001);
        assert!((center.y - 6.).abs() < 0.0001);
    }
}
