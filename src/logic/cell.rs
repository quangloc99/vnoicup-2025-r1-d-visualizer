use std::ops::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Cell(pub usize, pub usize);

impl Cell {
    pub fn encode(self, limit: Cell) -> usize {
        assert!(self.0 < limit.0 && self.1 < limit.1);
        self.0 * limit.1 + self.1
    }
    pub fn neighbor(self, lim: Cell) -> impl Iterator<Item = Cell> {
        [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .filter_map(move |(dr, dc)| Some(Cell(self.0.checked_add_signed(dr)?, self.1.checked_add_signed(dc)?)))
            .filter(move |&cell| cell.0 < lim.0 && cell.1 < lim.1)
    }

    pub fn dim(&self) -> usize {
        self.0 * self.1
    }

    pub fn inside(&self, top_left: Cell, bottom_right: Cell) -> bool {
        self.0 >= top_left.0 && self.0 <= bottom_right.0 && self.1 >= top_left.1 && self.1 <= bottom_right.1
    }

    pub fn mul(self, scalar: usize) -> Self {
        Self(self.0 * scalar, self.1 * scalar)
    }

    pub fn mahattan_dist(self, other: Cell) -> usize {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

impl Add<Cell> for Cell {
    type Output = Cell;

    fn add(self, other: Cell) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub<Cell> for Cell {
    type Output = Cell;
    fn sub(self, other: Cell) -> Self::Output {
        Self(self.0 - other.0, self.1 - other.1)
    }
}
