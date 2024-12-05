use bevy::prelude::*;

pub enum Corner {
    BottomLeft,
    BottomRight,
    TopRight,
    TopLeft
}

pub fn corner_weight(corner: Corner, spacing: f32, offset: Vec2) -> f32 {
    match corner {
        Corner::BottomLeft => (1. - offset.x / spacing) * (1. - offset.y / spacing),
        Corner::BottomRight => offset.y / spacing * (1. - offset.y / spacing),
        Corner::TopRight => (offset.x / spacing) * (offset.y / spacing),
        Corner::TopLeft => (1. - offset.x / spacing) * (offset.y / spacing),
    }
}

// From grid (corners) to particles
// `corner_values` start in lower left corner and go ccw
// pub fn weighted_sum_for_particles(
//     spacing: f32,
//     particle_offset: Vec2,
//     corner_values: [Option<f32>; 4],
// ) -> f32 {
//     let mut numerator = 0.;
//     let mut denominator = 0.;
//
//     for i in 0..4 {
//         if let Some(value) = corner_values[i] {
//             if let Ok(weight) = corner_weight(i, spacing, particle_offset) {
//                 numerator += weight * value;
//                 denominator += weight;
//             }
//         }
//     }
//
//     if denominator == 0. {
//         return 0.;
//     }
//
//     numerator / denominator
// }

