use bevy::prelude::*;
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use std::fmt::Debug;

pub struct SpatialHash {
    cols: usize,
    rows: usize,
    spacing: f32,
    offset: Vec2,
    starts: Vec<usize>,
    entries: Vec<usize>,
}

impl SpatialHash {
    pub fn from_sizes(width: f32, height: f32, particle_radius: f32) -> Self {
        let particle_size = particle_radius * 2.;
        Self::new(
            (width / particle_size) as usize + 1,
            (height / particle_size) as usize + 1,
            particle_size,
        )
    }

    pub fn new(cols: usize, rows: usize, spacing: f32) -> Self {
        Self {
            cols,
            rows,
            spacing,
            offset: Vec2::default(),
            starts: vec![],
            entries: vec![],
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn populate(&mut self, points: &Vec<Vec2>) {
        self.starts = vec![usize::default(); self.cols * self.rows + 1];
        self.entries = vec![usize::default(); points.len()];

        for point in points {
            let i = self.index_from_point(*point - self.offset);
            if i >= self.starts.len() {
                continue;
            }

            self.starts[i] += 1;
        }

        let mut start = 0;
        for i in 0..self.starts.len() {
            start += self.starts[i];
            self.starts[i] = start;
        }

        for (entry, point) in points.iter().enumerate() {
            let i = self.index_from_point(*point - self.offset);
            if i >= self.starts.len() {
                continue;
            }

            self.starts[i] -= 1;
            self.entries[self.starts[i]] = entry;
        }
    }

    pub fn query(&self, point: Vec2) -> Vec<usize> {
        let (i_low, j_low) = self.coords_from_point(point - self.offset - self.spacing);
        let (i_high, j_high) = self.coords_from_point(point - self.offset + self.spacing);

        let mut results = vec![];

        for i in i_low..=i_high {
            for j in j_low..=j_high {
                if i >= self.cols || j >= self.rows {
                    continue;
                }
                let start_index = self.index_from_coords(i, j);
                let start = self.starts[start_index];
                let end = self.starts[start_index + 1];
                for entry_index in start..end {
                    results.push(self.entries[entry_index]);
                }
            }
        }

        results
    }

    fn coords_from_point(&self, point: Vec2) -> (usize, usize) {
        let i = (point.x / self.spacing) as usize;
        let j = (point.y / self.spacing) as usize;
        (i, j)
    }

    fn index_from_coords(&self, i: usize, j: usize) -> usize {
        i * self.rows + j
    }

    fn index_from_point(&self, point: Vec2) -> usize {
        let (i, j) = self.coords_from_point(point);
        self.index_from_coords(i, j)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_hash_without_offset() {
        let mut hash = SpatialHash::new(4, 3, 10.);
        hash.populate(&vec![
            Vec2::new(15., 25.),
            Vec2::new(25., 5.),
            Vec2::new(35., 25.),
            Vec2::new(24., 6.),
            Vec2::new(16., 24.),
        ]);

        assert!(hash.query(Vec2::new(5., 5.)).is_empty());
        assert_eq!(hash.query(Vec2::new(15., 5.)), vec![3, 1]);
        assert_eq!(hash.query(Vec2::new(25., 5.)), vec![3, 1]);
        assert_eq!(hash.query(Vec2::new(35., 5.)), vec![3, 1]);

        assert_eq!(hash.query(Vec2::new(5., 15.)), vec![4, 0]);
        assert_eq!(hash.query(Vec2::new(15., 15.)), vec![4, 0, 3, 1]);
        assert_eq!(hash.query(Vec2::new(25., 15.)), vec![4, 0, 3, 1, 2]);
        assert_eq!(hash.query(Vec2::new(35., 15.)), vec![3, 1, 2]);

        assert_eq!(hash.query(Vec2::new(5., 25.)), vec![4, 0]);
        assert_eq!(hash.query(Vec2::new(15., 25.)), vec![4, 0]);
        assert_eq!(hash.query(Vec2::new(25., 25.)), vec![4, 0, 2]);
        assert_eq!(hash.query(Vec2::new(35., 25.)), vec![2]);
    }

    #[test]
    fn query_hash_with_offset() {
        let mut hash = SpatialHash::from_sizes(40., 30., 5.).with_offset(Vec2::new(-20., -15.));
        hash.populate(&vec![
            Vec2::new(-7.5, -10.),
            Vec2::new(-15., 10.),
            Vec2::new(5., 10.),
            Vec2::new(12.5, 10.),
        ]);

        assert_eq!(hash.query(Vec2::new(-15., -10.)), vec![0]);
        assert_eq!(hash.query(Vec2::new(-5., -10.)), vec![0]);
        assert_eq!(hash.query(Vec2::new(5., -10.)), vec![0]);
        assert_eq!(hash.query(Vec2::new(15., -10.)), vec![] as Vec<usize>);

        assert_eq!(hash.query(Vec2::new(-15., 0.)), vec![1, 0]);
        assert_eq!(hash.query(Vec2::new(-5., 0.)), vec![1, 0, 2]);
        assert_eq!(hash.query(Vec2::new(5., 0.)), vec![0, 2, 3]);
        assert_eq!(hash.query(Vec2::new(15., 0.)), vec![2, 3]);

        assert_eq!(hash.query(Vec2::new(-15., 10.)), vec![1]);
        assert_eq!(hash.query(Vec2::new(-5., 10.)), vec![1, 2]);
        assert_eq!(hash.query(Vec2::new(5., 10.)), vec![2, 3]);
        assert_eq!(hash.query(Vec2::new(15., 10.)), vec![2, 3]);
    }
}
