use bevy::prelude::*;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug)]
pub struct SpatialHash<T> {
    spacing: f32,
    table_size: usize,
    starts: Vec<usize>,
    entries: Vec<T>,
}

impl<T: Debug + Default + Copy + Clone + Hash + Eq + PartialEq> SpatialHash<T> {
    pub fn new(spacing: f32, max_num_objects: usize) -> Self {
        let table_size = 2 * max_num_objects;

        Self {
            table_size,
            spacing,
            starts: vec![usize::default(); table_size + 1],
            entries: vec![T::default(); max_num_objects],
        }
    }

    fn hash_coordinates(&self, i: usize, j: usize) -> usize {
        let hashed = (i * 92837111) ^ (j * 689287499);
        hashed % self.table_size
    }

    fn index(&self, point: Vec2) -> usize {
        let (i, j) = self.coords(point);
        self.hash_coordinates(i, j)
    }

    fn coords(&self, point: Vec2) -> (usize, usize) {
        let i = (point.x / self.spacing) as usize;
        let j = (point.y / self.spacing) as usize;
        (i, j)
    }

    pub fn populate(&mut self, entries: &Vec<(Vec2, T)>) {
        let num_object = entries.len().min(self.entries.len());

        self.starts.fill(usize::default());

        for (point, _) in &entries[0..num_object] {
            let i = self.index(*point);
            self.starts[i] += 1;
        }

        let mut start = 0;
        for i in 0..self.starts.len() {
            start += self.starts[i];
            self.starts[i] = start;
        }

        for (point, entry) in &entries[0..num_object] {
            let i = self.index(*point);
            self.starts[i] -= 1;
            self.entries[self.starts[i]] = *entry;
        }
    }

    // Will contain false positives
    pub fn query(&self, point: Vec2, max_distance: f32) -> HashSet<T> {
        let (i_low, j_low) = self.coords(point - max_distance);
        let (i_high, j_high) = self.coords(point + max_distance);

        let mut result = HashSet::new();

        for i in i_low..=i_high {
            for j in j_low..=j_high {
                let starts_index = self.hash_coordinates(i, j);
                let start = self.starts[starts_index];
                let end = self.starts[starts_index + 1];

                for entries_index in start..end {
                    result.insert(self.entries[entries_index]);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices() {
        let mut hash: SpatialHash<i32> = SpatialHash::new(10., 5);

        let one = Vec2::new(15., 25.);
        let two = Vec2::new(25., 5.);
        let three = Vec2::new(35., 25.);
        let four = Vec2::new(24., 6.);
        let five = Vec2::new(16., 24.);

        let entities = vec![(one, 1), (two, 2), (three, 3), (four, 4), (five, 5)];

        hash.populate(&entities);

        assert_eq!(hash.index(four), 2);
        assert_eq!(hash.index(two), 2);
        assert_eq!(hash.index(one), 3);
        assert_eq!(hash.index(five), 3);
        assert_eq!(hash.index(three), 9);
    }

    #[test]
    fn local_coords() {
        let hash: SpatialHash<i32> = SpatialHash::new(10., 5);

        let one = Vec2::new(15., 25.);
        let two = Vec2::new(25., 5.);
        let three = Vec2::new(35., 25.);
        let four = Vec2::new(24., 6.);
        let five = Vec2::new(16., 24.);

        assert_eq!(hash.coords(one), (1, 2));
        assert_eq!(hash.coords(two), (2, 0));
        assert_eq!(hash.coords(three), (3, 2));
        assert_eq!(hash.coords(four), (2, 0));
        assert_eq!(hash.coords(five), (1, 2));
    }

    #[test]
    fn query_point() {
        let mut hash: SpatialHash<i32> = SpatialHash::new(10., 20);

        let one = Vec2::new(15., 25.);
        let two = Vec2::new(25., 5.);
        let three = Vec2::new(35., 25.);
        let four = Vec2::new(24., 6.);
        let five = Vec2::new(16., 24.);

        let entities = vec![(one, 1), (two, 2), (three, 3), (four, 4), (five, 5)];

        hash.populate(&entities);

        // (15, 15)
        assert!(hash.query(Vec2::new(15., 5.), 10.).contains(&4));

        // (25, 5)
        assert!(hash.query(Vec2::new(25., 5.), 10.).contains(&2));
        assert!(hash.query(Vec2::new(25., 5.), 10.).contains(&4));

        // (15, 15)
        assert!(hash.query(Vec2::new(15., 15.), 10.).contains(&4));
        assert!(hash.query(Vec2::new(15., 15.), 10.).contains(&5));

        // (25, 15)
        assert!(hash.query(Vec2::new(25., 15.), 10.).contains(&4));
        assert!(hash.query(Vec2::new(25., 15.), 10.).contains(&5));

        // (15, 25)
        assert!(hash.query(Vec2::new(15., 25.), 10.).contains(&1));
        assert!(hash.query(Vec2::new(15., 25.), 10.).contains(&5));

        // (25, 25)
        assert!(hash.query(Vec2::new(25., 25.), 10.).contains(&5));

        // (35, 25)
        assert!(hash.query(Vec2::new(35., 25.), 10.).contains(&3));
    }
}
