pub struct Grid<T> {
    cols: usize,
    rows: usize,
    data: Vec<T>
}

impl<T: Default + Clone> Grid<T> {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            data: vec![T::default(); cols * rows]
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn get_at(&self, col: usize, row: usize) -> Option<&T> {
        self.data.get(row * self.cols + col)
    }

    pub fn get_at_mut(&mut self, col: usize, row: usize) -> Option<&mut T> {
        self.data.get_mut(row * self.cols + col)
    }
}