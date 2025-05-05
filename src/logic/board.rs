use super::cell::Cell;
use std::ops::*;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Board(pub Vec<Vec<usize>>);
impl Board {
    pub fn new(k: usize, missing: Cell) -> Self {
        let n = 1 << k;
        let board = vec![vec![0; n]; n];
        let mut res = Self(board);
        res._tile_fill(k, Cell(0, 0), missing, &mut 1);
        res
    }

    pub fn limit(&self) -> Cell {
        Cell(self.0.len(), self.0[0].len())
    }

    pub fn k(&self) -> usize {
        self.limit().0.trailing_zeros() as usize
    }

    fn _tile_fill(&mut self, k: usize, top_left: Cell, missing: Cell, pc: &mut usize) {
        if k == 0 {
            return;
        }
        let half = 1 << (k - 1);
        let missing_is_bottom = missing.0 - top_left.0 >= half;
        let missing_is_right = missing.1 - top_left.1 >= half;

        let cur_piece = *pc;
        *pc += 1;
        for cen_r in 0..2 {
            for cen_c in 0..2 {
                let new_top_left = Cell(top_left.0 + half * cen_r, top_left.1 + half * cen_c);
                let center = Cell(top_left.0 + half - 1 + cen_r, top_left.1 + half - 1 + cen_c);
                if cen_r == missing_is_bottom as usize && cen_c == missing_is_right as usize {
                    self._tile_fill(k - 1, new_top_left, missing, pc);
                } else {
                    self[center] = cur_piece;
                    self._tile_fill(k - 1, new_top_left, center, pc);
                }
            }
        }
    }

    pub fn make_move(&mut self, from: Cell, to: Cell) -> Option<usize> {
        assert!(self[from] == 0 || self[to] == 0);
        let moved_piece = self[from] + self[to];
        self._swap(from, to);
        if self._check_valid_move(from) && self._check_valid_move(to) {
            Some(moved_piece)
        } else {
            self._swap(from, to);
            None
        }
    }

    fn _swap(&mut self, from: Cell, to: Cell) {
        let piece = self[from];
        self[from] = self[to];
        self[to] = piece;
    }

    fn _check_valid_move(&self, cell: Cell) -> bool {
        let value = self[cell];
        if value == 0 {
            return true;
        }
        cell.neighbor(self.limit()).any(|nei| self[nei] == value)
    }
}

impl Index<Cell> for Board {
    type Output = usize;

    fn index(&self, index: Cell) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl IndexMut<Cell> for Board {
    fn index_mut(&mut self, index: Cell) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

pub fn build_move_graph(board: &mut Board, empty_cell: Cell) -> Vec<Vec<Cell>> {
    let lim = board.limit();
    let mut graph: Vec<Vec<Cell>> = vec![vec![]; lim.dim()];
    fn build_graph(board: &mut Board, graph: &mut Vec<Vec<Cell>>, visited: &mut Vec<bool>, u: Cell) {
        if visited[u.encode(board.limit())] {
            return;
        }
        visited[u.encode(board.limit())] = true;
        let lim = board.limit();
        for v in u.neighbor(board.limit()) {
            if board.make_move(u, v).is_some() {
                graph[u.encode(lim)].push(v);
                graph[v.encode(lim)].push(u);
                // eprintln!("{} {}", u.encode(lim), v.encode(lim));
                build_graph(board, graph, visited, v);
                board.make_move(v, u).unwrap();
            }
        }
    }

    build_graph(board, &mut graph, &mut vec![false; lim.dim()], empty_cell);
    graph
}
