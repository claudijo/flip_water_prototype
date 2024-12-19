use std::fmt::Debug;
use std::slice::{Iter, IterMut};

#[derive(Debug)]
pub struct Grid<T> {
    cols: usize,
    rows: usize,
    data: Vec<T>,
}

impl<T: Debug + Default + Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        Self {
            cols: self.cols,
            rows: self.rows,
            data: self.data.clone(),
        }
    }
}

impl<T: Debug + Default + Clone> Grid<T> {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            data: vec![T::default(); cols * rows],
        }
    }

    pub fn with_default_value(mut self, value: T) -> Self {
        self.data.fill(value);
        self
    }

    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn get(&self, i: i32, j: i32) -> Option<&T> {
        let (i, j) = self.validate_indices(i, j)?;
        self.data.get(j * self.cols + i)
    }

    pub fn get_mut(&mut self, i: i32, j: i32) -> Option<&mut T> {
        let (i, j) = self.validate_indices(i, j)?;
        self.data.get_mut(j * self.cols + i)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }

    fn validate_indices(&self, i: i32, j: i32) -> Option<(usize, usize)> {
        if i < 0 || j < 0 || i >= self.cols as i32 || j >= self.rows as i32 {
            return None;
        }

        Some((i as usize, j as usize))
    }
}
