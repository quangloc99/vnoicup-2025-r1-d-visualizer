use super::cell::Cell;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct PartBoard {
    k: usize,
    top_left: Cell,
}

impl PartBoard {
    fn size(&self) -> usize {
        1 << self.k
    }

    fn half(&self) -> usize {
        1 << (self.k - 1)
    }

    fn bottom_right(&self) -> Cell {
        self.top_left + Cell(1 << self.k, 1 << self.k) - Cell(1, 1)
    }

    fn get_quad(&self, cell: Cell) -> Cell {
        assert!(cell.inside(self.top_left, self.bottom_right()));
        let half = self.half();
        let is_bottom = cell.0 - self.top_left.0 >= half;
        let is_right = cell.1 - self.top_left.1 >= half;
        Cell(is_bottom as usize, is_right as usize)
    }

    fn part_of(&self, quad: Cell) -> Self {
        Self { k: self.k - 1, top_left: self.top_left_of(quad) }
    }

    fn joint_of(&self, quad: Cell) -> Cell {
        self.top_left + Cell(self.half() - 1, self.half() - 1) + quad
    }

    fn top_left_of(&self, quad: Cell) -> Cell {
        self.top_left + quad.mul(self.half())
    }
}

pub fn dist(k: usize, u: Cell, v: Cell) -> usize {
    fn _dist(pb: PartBoard, u: Cell, v: Cell) -> usize {
        if pb.k == 0 {
            return 0;
        }
        let diag_dist = (pb.size() - 1) * 2;
        if u.mahattan_dist(v) == diag_dist {
            return diag_dist;
        }

        let quad_u = pb.get_quad(u);
        let quad_v = pb.get_quad(v);
        if quad_u == quad_v {
            _dist(pb.part_of(quad_u), u, v)
        } else {
            let u_joint = pb.joint_of(quad_u);
            let v_joint = pb.joint_of(quad_v);

            u_joint.mahattan_dist(v_joint)
                + _dist(pb.part_of(quad_u), u, u_joint)
                + _dist(pb.part_of(quad_v), v, v_joint)
        }
    }
    _dist(PartBoard { k, top_left: Cell(0, 0) }, u, v)
}
