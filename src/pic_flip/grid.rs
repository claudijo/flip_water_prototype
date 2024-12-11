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
        let (i, j) = self.validate_indices(i,j)?;
        self.data.get(j * self.cols + i)
    }

    pub fn get_at_mut(&mut self, i: i32, j: i32) -> Option<&mut T> {
        let (i, j) = self.validate_indices(i,j)?;
        self.data.get_mut(j * self.cols + i)
    }

    fn validate_indices(&self, i: i32, j: i32) -> Option<(usize, usize)> {
        if i < 0 || j < 0 || i > self.cols as i32 - 1 || j > self.rows as i32 - 1{
            return None;
        }

        Some((i as usize, j as usize))
    }
}
