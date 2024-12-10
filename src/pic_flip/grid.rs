pub struct Grid<T> {
    cols: usize,
    rows: usize,
    data: Vec<T>,
}

impl<T: Default + Clone> Grid<T> {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            data: vec![T::default(); cols * rows],
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn get_at(&self, i: i32, j: i32) -> Option<&T> {
        if i < 0 || j < 0 {
            return None;
        }

        self.data.get(j as usize * self.cols + i as usize)
    }

    pub fn get_at_mut(&mut self, i: i32, j: i32) -> Option<&mut T> {
        if i < 0 || j < 0 {
            return None;
        }

        self.data.get_mut(j as usize * self.cols + i as usize)
    }
}
